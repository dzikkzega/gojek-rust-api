use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use subtle::ConstantTimeEq;
use tracing::warn;
use uuid::Uuid;

use crate::AppState;

#[derive(Serialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorDetail,
}

#[derive(Serialize)]
pub struct ApiErrorDetail {
    pub code: &'static str,
    pub message: &'static str,
    pub request_id: String,
}

pub fn api_error(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    let body = ApiErrorEnvelope {
        error: ApiErrorDetail {
            code,
            message,
            request_id: format!("req_{}", Uuid::new_v4().simple()),
        },
    };
    (status, Json(body)).into_response()
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/auth/otp/request", post(request_otp))
        .route("/api/v1/auth/otp/verify", post(verify_otp))
        .route("/api/v1/auth/token/refresh", post(refresh_token))
        .route("/api/v1/auth/logout", post(logout))
}

#[derive(Deserialize)]
pub struct OtpRequestPayload {
    pub phone_number: String,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct OtpRequestAccepted {
    pub status: &'static str,
    pub expires_in_seconds: u64,
}

#[derive(Deserialize)]
pub struct OtpVerifyPayload {
    pub phone_number: String,
    pub code: String,
    pub device_label: Option<String>,
}

#[derive(Deserialize)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct UserProfileDto {
    pub id: Uuid,
    pub phone_number: String,
    pub display_name: String,
    pub email: Option<String>,
    pub role: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct SessionResponseDto {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub user: UserProfileDto,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
}

pub fn normalize_phone(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('+') {
        return None;
    }
    let digits = &trimmed[1..];
    if digits.len() < 8 || digits.len() > 15 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(trimmed.to_string())
}

pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
        let _ = write!(acc, "{:02x}", b);
        acc
    })
}

pub fn hash_token(raw: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hasher.finalize().to_vec()
}

pub fn hash_otp(phone: &str, code: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(phone.as_bytes());
    hasher.update(b":");
    hasher.update(code.trim().as_bytes());
    hasher.finalize().to_vec()
}

fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub async fn request_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpRequestPayload>,
) -> Response {
    let Some(phone) = normalize_phone(&payload.phone_number) else {
        return api_error(
            StatusCode::BAD_REQUEST,
            "invalid_phone_format",
            "Phone number must be valid E.164 (e.g. +6281234567890)",
        );
    };

    let Some(db) = &state.db else {
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "unavailable", "Authentication is temporarily unavailable");
    };

    // No OTP delivery provider is configured (PRD decision D-05). Never create
    // a challenge that cannot be delivered; keep response generic to avoid enumeration.
    let _ = (&phone, db);
    api_error(
        StatusCode::SERVICE_UNAVAILABLE,
        "otp_delivery_unavailable",
        "Authentication is temporarily unavailable",
    )
}

pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpVerifyPayload>,
) -> Response {
    let Some(phone) = normalize_phone(&payload.phone_number) else {
        return api_error(
            StatusCode::BAD_REQUEST,
            "invalid_phone_format",
            "Phone number must be valid E.164",
        );
    };

    let Some(db) = &state.db else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "database_unavailable",
            "Database is not ready",
        );
    };

    // Find active unexpired challenge
    let challenge = sqlx::query(
        r#"
        SELECT id, code_hash, attempt_count, max_attempts
        FROM auth_otp_challenges
        WHERE phone_e164 = $1 AND status = 'active' AND expires_at > now()
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .bind(&phone)
    .fetch_optional(db)
    .await;

    let challenge_row = match challenge {
        Ok(Some(row)) => row,
        _ => {
            return api_error(
                StatusCode::UNAUTHORIZED,
                "invalid_or_expired_otp",
                "Invalid, expired, or non-existent OTP",
            );
        }
    };

    let challenge_id: Uuid = challenge_row.get("id");
    let stored_hash: Vec<u8> = challenge_row.get("code_hash");
    let attempt_count: i16 = challenge_row.get("attempt_count");
    let max_attempts: i16 = challenge_row.get("max_attempts");

    if attempt_count >= max_attempts {
        let _ = sqlx::query("UPDATE auth_otp_challenges SET status = 'locked' WHERE id = $1")
            .bind(challenge_id)
            .execute(db)
            .await;
        return api_error(
            StatusCode::UNAUTHORIZED,
            "otp_attempts_exceeded",
            "Maximum attempts exceeded. Please request a new OTP",
        );
    }

    let input_hash = hash_otp(&phone, &payload.code);
    let is_valid = bool::from(stored_hash.ct_eq(&input_hash));

    if !is_valid {
        let _ = sqlx::query(
            "UPDATE auth_otp_challenges SET attempt_count = attempt_count + 1 WHERE id = $1"
        )
        .bind(challenge_id)
        .execute(db)
        .await;

        return api_error(
            StatusCode::UNAUTHORIZED,
            "invalid_or_expired_otp",
            "Invalid OTP code provided",
        );
    }

    // Atomic consumption and user onboarding in transaction
    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Transaction start failed",
            );
        }
    };

    let mark_consumed = sqlx::query(
        "UPDATE auth_otp_challenges SET status = 'consumed', consumed_at = now() WHERE id = $1 AND status = 'active'"
    )
    .bind(challenge_id)
    .execute(&mut *tx)
    .await;

    match mark_consumed {
        Ok(res) if res.rows_affected() == 1 => {}
        _ => {
            let _ = tx.rollback().await;
            return api_error(
                StatusCode::UNAUTHORIZED,
                "otp_already_consumed",
                "OTP has already been used",
            );
        }
    }

    // Upsert rider user account
    let user_row = match sqlx::query(
        r#"
        INSERT INTO users (phone_e164, status, role)
        VALUES ($1, 'active', 'rider')
        ON CONFLICT (phone_e164) DO UPDATE
            SET updated_at = now()
        RETURNING id, phone_e164, display_name, email, role::text, status::text,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at_utc
        "#
    )
    .bind(&phone)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(u) => u,
        Err(e) => {
            let _ = tx.rollback().await;
            warn!(error = %e, "failed to upsert user");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "User account creation failed",
            );
        }
    };

    let user_id: Uuid = user_row.get("id");
    let user_role: String = user_row.get("role");
    let user_status: String = user_row.get("status");
    let display_name: String = user_row.get("display_name");
    let email: Option<String> = user_row.get("email");
    let created_at: String = user_row.get("created_at_utc");

    if user_status != "active" {
        let _ = tx.rollback().await;
        return api_error(
            StatusCode::FORBIDDEN,
            "account_inactive",
            "Account is not active",
        );
    }

    // Generate cryptographically secure refresh token & persist hash
    let mut raw_token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut raw_token_bytes);
    let raw_refresh_token = to_hex(&raw_token_bytes);
    let refresh_hash_str = to_hex(&hash_token(&raw_refresh_token));

    let session_row = match sqlx::query(
        r#"
        INSERT INTO auth_sessions (user_id, refresh_token_hash, device_label, expires_at)
        VALUES ($1, $2, $3, now() + interval '30 days')
        RETURNING id
        "#
    )
    .bind(user_id)
    .bind(&refresh_hash_str)
    .bind(&payload.device_label)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.rollback().await;
            warn!(error = %e, "failed to create session");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Failed to issue session",
            );
        }
    };

    if let Err(_) = tx.commit().await {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Commit failed",
        );
    }

    let session_id: Uuid = session_row.get("id");

    // Sign 15-minute JWT Access Token
    let jwt_secret = state.jwt_secret.as_bytes();
    let issued_at = unix_now_secs() as usize;
    let claims = Claims {
        sub: user_id,
        session_id,
        role: user_role.clone(),
        iat: issued_at,
        iss: "gojek-rust-api".to_owned(),
        aud: "gojek-rust-api".to_owned(),
        exp: issued_at + 15 * 60,
    };

    let access_token = match jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    ) {
        Ok(t) => t,
        Err(e) => {
            warn!(error = %e, "failed to sign JWT");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Token generation failed",
            );
        }
    };

    (
        StatusCode::OK,
        Json(SessionResponseDto {
            access_token,
            refresh_token: raw_refresh_token,
            token_type: "Bearer",
            expires_in: 900,
            user: UserProfileDto {
                id: user_id,
                phone_number: phone,
                display_name,
                email,
                role: user_role,
                status: user_status,
                created_at,
            },
        }),
    )
        .into_response()
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Response {
    let Some(db) = &state.db else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "database_unavailable",
            "Database is not ready",
        );
    };

    let incoming_hash = to_hex(&hash_token(&payload.refresh_token));

    // Lock the session row before checking and rotating it. This serializes
    // concurrent refresh attempts using the same token.
    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            warn!(error = %e, "failed to begin refresh transaction");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Session refresh failed");
        }
    };
    let session_row = match sqlx::query(
        "SELECT id, user_id, (revoked_at IS NOT NULL) AS is_revoked, (expires_at <= now()) AS is_expired FROM auth_sessions WHERE refresh_token_hash = $1 FOR UPDATE"
    )
    .bind(&incoming_hash)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(row)) => row,
        _ => {
            let _ = tx.rollback().await;
            return api_error(StatusCode::UNAUTHORIZED, "invalid_refresh_token", "The refresh token is invalid");
        }
    };
    let session_id: Uuid = session_row.get("id");
    let user_id: Uuid = session_row.get("user_id");
    let is_revoked: bool = session_row.get("is_revoked");
    let is_expired: bool = session_row.get("is_expired");
    if is_revoked {
        let _ = tx.rollback().await;
        return api_error(StatusCode::CONFLICT, "session_revoked", "This session has already been revoked or replayed");
    }
    if is_expired {
        let _ = tx.rollback().await;
        return api_error(StatusCode::UNAUTHORIZED, "token_expired", "The refresh token has expired");
    }

    // Revoke old session atomically while holding its row lock.
    let revoke_res = sqlx::query(
        "UPDATE auth_sessions SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL AND expires_at > now()"
    )
    .bind(session_id)
    .execute(&mut *tx)
    .await;

    match revoke_res {
        Ok(res) if res.rows_affected() == 1 => {}
        _ => {
            let _ = tx.rollback().await;
            return api_error(
                StatusCode::CONFLICT,
                "session_revoked",
                "This session has already been revoked or replayed",
            );
        }
    }

    // Issue new random refresh token
    let mut raw_token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut raw_token_bytes);
    let new_refresh_token = to_hex(&raw_token_bytes);
    let new_hash = to_hex(&hash_token(&new_refresh_token));

    let new_session_row = match sqlx::query(
        r#"
        INSERT INTO auth_sessions (user_id, refresh_token_hash, expires_at)
        VALUES ($1, $2, now() + interval '30 days')
        RETURNING id
        "#
    )
    .bind(user_id)
    .bind(&new_hash)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(ns) => ns,
        Err(e) => {
            let _ = tx.rollback().await;
            warn!(error = %e, "failed to rotate session");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Failed to rotate session",
            );
        }
    };

    let user_row = match sqlx::query(
        "SELECT id, phone_e164, display_name, email, role::text, status::text, to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at_utc FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(u) => u,
        Err(_) => {
            let _ = tx.rollback().await;
            return api_error(
                StatusCode::NOT_FOUND,
                "user_not_found",
                "User account not found",
            );
        }
    };

    if let Err(_) = tx.commit().await {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Failed to commit rotated session",
        );
    }

    let new_session_id: Uuid = new_session_row.get("id");
    let user_role: String = user_row.get("role");
    let phone: String = user_row.get("phone_e164");
    let display_name: String = user_row.get("display_name");
    let email: Option<String> = user_row.get("email");
    let user_status: String = user_row.get("status");
    let created_at: String = user_row.get("created_at_utc");
    if user_status != "active" {
        return api_error(
            StatusCode::FORBIDDEN,
            "account_inactive",
            "Account is not active",
        );
    }

    let issued_at = unix_now_secs() as usize;
    let claims = Claims {
        sub: user_id,
        session_id: new_session_id,
        role: user_role.clone(),
        iat: issued_at,
        iss: "gojek-rust-api".to_owned(),
        aud: "gojek-rust-api".to_owned(),
        exp: issued_at + 15 * 60,
    };

    let access_token = match jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Failed to sign access token",
            );
        }
    };

    (
        StatusCode::OK,
        Json(SessionResponseDto {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer",
            expires_in: 900,
            user: UserProfileDto {
                id: user_id,
                phone_number: phone,
                display_name,
                email,
                role: user_role,
                status: user_status,
                created_at,
            },
        }),
    )
        .into_response()
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let Some(db) = &state.db else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "database_unavailable",
            "Database is not ready",
        );
    };

    let Some(auth_header) = headers.get(header::AUTHORIZATION).and_then(|h| h.to_str().ok()) else {
        return api_error(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Missing Authorization header",
        );
    };

    let token = if let Some(stripped) = auth_header.strip_prefix("Bearer ") {
        stripped.trim()
    } else {
        return api_error(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Invalid authorization scheme",
        );
    };

    let mut validation = Validation::default();
    validation.set_issuer(&["gojek-rust-api"]);
    validation.set_audience(&["gojek-rust-api"]);
    let token_data = match jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(_) => {
            return api_error(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Invalid or expired access token",
            );
        }
    };

    if let Err(e) = sqlx::query("UPDATE auth_sessions SET revoked_at = now() WHERE id = $1")
        .bind(token_data.claims.session_id)
        .execute(db)
        .await
    {
        warn!(error = %e, "failed to revoke session");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Failed to revoke session",
        );
    }

    StatusCode::NO_CONTENT.into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_normalization() {
        assert_eq!(normalize_phone("+6281234567890"), Some("+6281234567890".into()));
        assert_eq!(normalize_phone("  +6281234567890  "), Some("+6281234567890".into()));
        assert_eq!(normalize_phone("081234567890"), None);
        assert_eq!(normalize_phone("+123"), None); // Too short (<8 digits)
        assert_eq!(normalize_phone("+62812abc7890"), None);
    }

    #[test]
    fn test_hex_encoding() {
        let sample = b"hello";
        assert_eq!(to_hex(sample), "68656c6c6f");
    }

    #[test]
    fn test_hash_otp_determinism() {
        let hash1 = hash_otp("+6281234567890", "123456");
        let hash2 = hash_otp("+6281234567890", "123456");
        assert_eq!(hash1, hash2);

        let hash_different = hash_otp("+6281234567890", "654321");
        assert_ne!(hash1, hash_different);
    }
}
