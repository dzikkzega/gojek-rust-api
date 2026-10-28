use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use lettre::AsyncTransport;
use rand::{Rng, RngCore, rngs::OsRng};
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
    pub email: String,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct OtpRequestAccepted {
    pub status: &'static str,
    pub expires_in_seconds: u64,
}

#[derive(Deserialize)]
pub struct OtpVerifyPayload {
    pub email: String,
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
    pub email: String,
    pub display_name: String,
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

pub fn normalize_email(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() > 254 || trimmed.chars().any(char::is_whitespace) {
        return None;
    }
    let (local, domain) = trimmed.split_once('@')?;
    if local.is_empty() || domain.is_empty() || domain.starts_with('.') || domain.ends_with('.') {
        return None;
    }
    if domain.split('.').count() < 2 || local.starts_with('.') || local.ends_with('.') {
        return None;
    }
    if trimmed.matches('@').count() != 1 {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            let _ = write!(acc, "{:02x}", b);
            acc
        })
}

pub fn hash_token(raw: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hasher.finalize().to_vec()
}

pub const MAX_ACTIVE_SESSIONS: usize = 8;

pub fn hash_otp(secret: &str, email: &str, code: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"otp-v1:");
    hasher.update(secret.as_bytes());
    hasher.update(b":");
    hasher.update(email.as_bytes());
    hasher.update(b":");
    hasher.update(code.trim().as_bytes());
    hasher.finalize().to_vec()
}

pub fn sessions_to_revoke<T: Clone>(
    active_session_ids_oldest_first: &[T],
    max_active: usize,
) -> Vec<T> {
    if max_active == 0 {
        return active_session_ids_oldest_first.to_vec();
    }
    let keep_before_insert = max_active.saturating_sub(1);
    if active_session_ids_oldest_first.len() <= keep_before_insert {
        Vec::new()
    } else {
        let revoke_count = active_session_ids_oldest_first.len() - keep_before_insert;
        active_session_ids_oldest_first[..revoke_count].to_vec()
    }
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
    let Some(email) = normalize_email(&payload.email) else {
        return api_error(
            StatusCode::BAD_REQUEST,
            "invalid_email_format",
            "A valid email address is required",
        );
    };
    let (Some(db), Some(mut redis_conn), Some(mailer), Some(sender)) = (
        &state.db,
        state.redis.clone(),
        &state.mailer,
        &state.smtp_from,
    ) else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "email_delivery_unavailable",
            "Authentication is temporarily unavailable",
        );
    };

    let email_key_hash = to_hex(&hash_token(&email));
    let rl_key = format!("rl:otp:email:{email_key_hash}");
    let count = match redis::cmd("INCR")
        .arg(&rl_key)
        .query_async::<i64>(&mut redis_conn)
        .await
    {
        Ok(count) => count,
        Err(error) => {
            warn!(error = %error, "OTP email rate limiter unavailable");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
            );
        }
    };
    if count == 1
        && let Err(error) = redis::cmd("EXPIRE")
            .arg(&rl_key)
            .arg(900)
            .query_async::<bool>(&mut redis_conn)
            .await
    {
        warn!(error = %error, "OTP email rate limit expiry setup failed");
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "authentication_unavailable",
            "Authentication is temporarily unavailable",
        );
    }
    if count > 5 {
        return api_error(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "Too many OTP requests; please try again later",
        );
    }

    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(error) => {
            warn!(error = %error, "OTP request transaction failed");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
            );
        }
    };

    let previous = sqlx::query(
        "SELECT resend_after > now() AS cooling_down FROM auth_otp_challenges WHERE email_normalized = $1 AND status = 'active' ORDER BY created_at DESC LIMIT 1 FOR UPDATE"
    )
    .bind(&email)
    .fetch_optional(&mut *tx)
    .await;
    match previous {
        Ok(Some(row)) if row.get::<bool, _>("cooling_down") => {
            let _ = tx.rollback().await;
            return api_error(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Please wait before requesting another code",
            );
        }
        Ok(_) => {}
        Err(error) => {
            let _ = tx.rollback().await;
            warn!(error = %error, "OTP cooldown lookup failed");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
            );
        }
    }

    if let Err(error) = sqlx::query(
        "UPDATE auth_otp_challenges SET status = 'expired' WHERE email_normalized = $1 AND status = 'active'"
    )
    .bind(&email)
    .execute(&mut *tx)
    .await
    {
        let _ = tx.rollback().await;
        warn!(error = %error, "OTP challenge cleanup failed");
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "authentication_unavailable", "Authentication is temporarily unavailable");
    }

    let code = format!("{:06}", OsRng.gen_range(0..1_000_000_u32));
    let code_hash = hash_otp(&state.jwt_secret, &email, &code);
    if let Err(error) = sqlx::query(
        "INSERT INTO auth_otp_challenges (phone_e164, email_normalized, code_hash, status, attempt_count, max_attempts, expires_at, resend_after) VALUES (NULL, $1, $2, 'active', 0, 5, now() + interval '5 minutes', now() + interval '60 seconds')"
    )
    .bind(&email)
    .bind(code_hash)
    .execute(&mut *tx)
    .await
    {
        let _ = tx.rollback().await;
        warn!(error = %error, "OTP challenge insert failed");
        return api_error(StatusCode::SERVICE_UNAVAILABLE, "authentication_unavailable", "Authentication is temporarily unavailable");
    }
    if let Err(error) = tx.commit().await {
        warn!(error = %error, "OTP request commit failed");
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "authentication_unavailable",
            "Authentication is temporarily unavailable",
        );
    }

    // Send to the supplied address regardless of account existence to reduce enumeration.
    let message = match lettre::Message::builder()
        .from(sender.clone())
        .to(match email.parse() {
            Ok(to) => to,
            Err(_) => return api_error(StatusCode::BAD_REQUEST, "invalid_email_format", "A valid email address is required"),
        })
        .subject("Your sign-in verification code")
        .header(lettre::message::header::ContentType::TEXT_PLAIN)
        .body(format!("Your verification code is {code}. It expires in 5 minutes. If you did not request this, ignore this email."))
    {
        Ok(message) => message,
        Err(error) => {
            warn!(error = %error, "OTP email message construction failed");
            return api_error(StatusCode::SERVICE_UNAVAILABLE, "email_delivery_unavailable", "Authentication is temporarily unavailable");
        }
    };
    if let Err(error) = mailer.send(message).await {
        warn!(error = %error, "OTP email delivery failed");
        let _ = sqlx::query("UPDATE auth_otp_challenges SET status = 'expired' WHERE email_normalized = $1 AND status = 'active'")
            .bind(&email).execute(db).await;
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "email_delivery_unavailable",
            "Authentication is temporarily unavailable",
        );
    }

    (
        StatusCode::ACCEPTED,
        Json(OtpRequestAccepted {
            status: "accepted",
            expires_in_seconds: 300,
        }),
    )
        .into_response()
}

pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpVerifyPayload>,
) -> Response {
    let Some(email) = normalize_email(&payload.email) else {
        return api_error(
            StatusCode::BAD_REQUEST,
            "invalid_email_format",
            "A valid email address is required",
        );
    };
    let Some(db) = &state.db else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "unavailable",
            "Authentication is temporarily unavailable",
        );
    };

    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(error) => {
            warn!(error = %error, "OTP verification transaction failed");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
            );
        }
    };
    let row = match sqlx::query(
        "SELECT id, code_hash, attempt_count, max_attempts FROM auth_otp_challenges WHERE email_normalized = $1 AND status = 'active' AND expires_at > now() ORDER BY created_at DESC LIMIT 1 FOR UPDATE"
    )
    .bind(&email)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(row)) => row,
        _ => {
            let _ = tx.rollback().await;
            return api_error(StatusCode::UNAUTHORIZED, "invalid_or_expired_otp", "Invalid or expired code");
        }
    };
    let challenge_id: Uuid = row.get("id");
    let stored_hash: Vec<u8> = row.get("code_hash");
    let attempts: i16 = row.get("attempt_count");
    let max_attempts: i16 = row.get("max_attempts");

    if attempts >= max_attempts {
        let _ = sqlx::query("UPDATE auth_otp_challenges SET status='locked' WHERE id=$1")
            .bind(challenge_id)
            .execute(&mut *tx)
            .await;
        let _ = tx.commit().await;
        return api_error(
            StatusCode::UNAUTHORIZED,
            "otp_attempts_exceeded",
            "Maximum attempts exceeded",
        );
    }
    let candidate_hash = hash_otp(&state.jwt_secret, &email, &payload.code);
    if !bool::from(stored_hash.as_slice().ct_eq(candidate_hash.as_slice())) {
        let new_attempts = attempts.saturating_add(1);
        let new_status = if new_attempts >= max_attempts {
            "locked"
        } else {
            "active"
        };
        let update = sqlx::query("UPDATE auth_otp_challenges SET attempt_count=$2, status=$3 WHERE id=$1 AND status='active'")
            .bind(challenge_id).bind(new_attempts).bind(new_status).execute(&mut *tx).await;
        if let Err(error) = update {
            let _ = tx.rollback().await;
            warn!(error = %error, "OTP attempt update failed");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
            );
        }
        let _ = tx.commit().await;
        return api_error(
            StatusCode::UNAUTHORIZED,
            "invalid_or_expired_otp",
            "Invalid or expired code",
        );
    }

    let consumed = sqlx::query("UPDATE auth_otp_challenges SET status='consumed', consumed_at=now() WHERE id=$1 AND status='active' AND expires_at > now()")
        .bind(challenge_id).execute(&mut *tx).await;
    match consumed {
        Ok(result) if result.rows_affected() == 1 => {}
        _ => {
            let _ = tx.rollback().await;
            return api_error(
                StatusCode::UNAUTHORIZED,
                "invalid_or_expired_otp",
                "Invalid or expired code",
            );
        }
    }

    let user_row = match sqlx::query(
        "INSERT INTO users (email, email_normalized, email_verified_at, status, role) VALUES ($1,$1,now(),'active','rider') ON CONFLICT (email_normalized) DO UPDATE SET email_verified_at=COALESCE(users.email_verified_at,now()), updated_at=now() RETURNING id, email_normalized, display_name, role::text, status::text, to_char(created_at AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at_utc"
    )
    .bind(&email).fetch_one(&mut *tx).await {
        Ok(user) => user,
        Err(error) => {
            let _ = tx.rollback().await;
            warn!(error = %error, "email account upsert failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Account creation failed");
        }
    };
    let user_id: Uuid = user_row.get("id");
    let role: String = user_row.get("role");
    let status: String = user_row.get("status");
    let display_name: String = user_row.get("display_name");
    let normalized_email: String = user_row.get("email_normalized");
    let created_at: String = user_row.get("created_at_utc");
    if status != "active" {
        let _ = tx.rollback().await;
        return api_error(
            StatusCode::FORBIDDEN,
            "account_inactive",
            "Account is not active",
        );
    }

    // Enforce MAX_ACTIVE_SESSIONS (8) by revoking oldest active sessions before inserting the new one
    let active_rows = match sqlx::query(
        "SELECT id FROM auth_sessions WHERE user_id = $1 AND revoked_at IS NULL AND expires_at > now() ORDER BY created_at ASC FOR UPDATE"
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(error) => {
            let _ = tx.rollback().await;
            warn!(error = %error, "failed to lock active sessions");
            return api_error(StatusCode::SERVICE_UNAVAILABLE, "authentication_unavailable", "Authentication is temporarily unavailable");
        }
    };
    let active_ids: Vec<Uuid> = active_rows
        .into_iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect();
    for old_id in sessions_to_revoke(&active_ids, MAX_ACTIVE_SESSIONS) {
        if let Err(error) = sqlx::query(
            "UPDATE auth_sessions SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL",
        )
        .bind(old_id)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            warn!(error = %error, "failed to enforce active session limit");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
            );
        }
    }

    let mut token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut token_bytes);
    let refresh_token = to_hex(&token_bytes);
    let refresh_hash = to_hex(&hash_token(&refresh_token));
    let session_row = match sqlx::query("INSERT INTO auth_sessions (user_id, refresh_token_hash, device_label, expires_at) VALUES ($1,$2,$3,now()+interval '30 days') RETURNING id")
        .bind(user_id).bind(refresh_hash).bind(&payload.device_label).fetch_one(&mut *tx).await {
        Ok(session) => session,
        Err(error) => {
            let _ = tx.rollback().await;
            warn!(error = %error, "session creation failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Session creation failed");
        }
    };
    if let Err(error) = tx.commit().await {
        warn!(error = %error, "OTP verification commit failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Session creation failed",
        );
    }

    let session_id: Uuid = session_row.get("id");
    let now = unix_now_secs() as usize;
    let claims = Claims {
        sub: user_id,
        session_id,
        role: role.clone(),
        iat: now,
        iss: "gojek-rust-api".to_owned(),
        aud: "gojek-rust-api".to_owned(),
        exp: now + 15 * 60,
    };
    let access_token = match jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ) {
        Ok(token) => token,
        Err(error) => {
            warn!(error = %error, "JWT signing failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Session creation failed",
            );
        }
    };
    (
        StatusCode::OK,
        Json(SessionResponseDto {
            access_token,
            refresh_token,
            token_type: "Bearer",
            expires_in: 900,
            user: UserProfileDto {
                id: user_id,
                email: normalized_email,
                display_name,
                role,
                status,
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
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Session refresh failed",
            );
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
        if let Err(error) = sqlx::query(
            "UPDATE auth_sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            warn!(error = %error, "failed to revoke all sessions after refresh replay");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
            );
        }
        if let Err(error) = tx.commit().await {
            warn!(error = %error, "failed to commit replay response session revocation");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
            );
        }
        return api_error(
            StatusCode::CONFLICT,
            "session_revoked",
            "This session has already been revoked or replayed",
        );
    }
    if is_expired {
        let _ = tx.rollback().await;
        return api_error(
            StatusCode::UNAUTHORIZED,
            "token_expired",
            "The refresh token has expired",
        );
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

    // Enforce MAX_ACTIVE_SESSIONS (8) by revoking oldest active sessions before inserting the rotated one
    let active_rows = match sqlx::query(
        "SELECT id FROM auth_sessions WHERE user_id = $1 AND revoked_at IS NULL AND expires_at > now() ORDER BY created_at ASC FOR UPDATE"
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(error) => {
            let _ = tx.rollback().await;
            warn!(error = %error, "failed to lock active sessions");
            return api_error(StatusCode::SERVICE_UNAVAILABLE, "authentication_unavailable", "Authentication is temporarily unavailable");
        }
    };
    let active_ids: Vec<Uuid> = active_rows
        .into_iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect();
    for old_id in sessions_to_revoke(&active_ids, MAX_ACTIVE_SESSIONS) {
        if let Err(error) = sqlx::query(
            "UPDATE auth_sessions SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL",
        )
        .bind(old_id)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            warn!(error = %error, "failed to enforce active session limit");
            return api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable",
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
        "#,
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
        "SELECT id, email_normalized, display_name, role::text, status::text, to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at_utc FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(user) => user,
        Err(error) => {
            let _ = tx.rollback().await;
            warn!(error = %error, "failed to load account for refresh");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Session refresh failed");
        }
    };
    let user_status: String = user_row.get("status");
    if user_status != "active" {
        let _ = tx.rollback().await;
        return api_error(
            StatusCode::FORBIDDEN,
            "account_inactive",
            "Account is not active",
        );
    }
    let user_role: String = user_row.get("role");
    let email: String = match user_row.get::<Option<String>, _>("email_normalized") {
        Some(email) => email,
        None => {
            let _ = tx.rollback().await;
            return api_error(
                StatusCode::FORBIDDEN,
                "email_not_verified",
                "Email verification is required",
            );
        }
    };
    let display_name: String = user_row.get("display_name");
    let created_at: String = user_row.get("created_at_utc");
    if let Err(error) = tx.commit().await {
        warn!(error = %error, "failed to commit rotated session");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Session refresh failed",
        );
    }
    let new_session_id: Uuid = new_session_row.get("id");
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
                email,
                display_name,
                role: user_role,
                status: user_status,
                created_at,
            },
        }),
    )
        .into_response()
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(db) = &state.db else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "database_unavailable",
            "Database is not ready",
        );
    };

    let Some(auth_header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    else {
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
    fn test_email_normalization() {
        assert_eq!(
            normalize_email("  Rider@Example.TEST "),
            Some("rider@example.test".into())
        );
        assert_eq!(normalize_email("no-at-sign"), None);
        assert_eq!(normalize_email("bad@domain"), None);
        assert_eq!(normalize_email(".bad@example.test"), None);
    }

    #[test]
    fn test_hex_encoding() {
        let sample = b"hello";
        assert_eq!(to_hex(sample), "68656c6c6f");
    }

    #[test]
    fn test_hash_otp_determinism() {
        let hash1 = hash_otp("secret-key", "rider@example.test", "123456");
        let hash2 = hash_otp("secret-key", "rider@example.test", "123456");
        assert_eq!(hash1, hash2);

        let hash_different = hash_otp("secret-key", "rider@example.test", "654321");
        assert_ne!(hash1, hash_different);

        let hash_other_secret = hash_otp("other-key", "rider@example.test", "123456");
        assert_ne!(hash1, hash_other_secret);
    }

    #[test]
    fn test_sessions_to_revoke_cap() {
        let ids: Vec<u32> = (1..=8).collect();
        let to_revoke = sessions_to_revoke(&ids, MAX_ACTIVE_SESSIONS);
        assert_eq!(to_revoke, vec![1]);
    }
}
