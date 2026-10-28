use std::{env, net::SocketAddr, time::Duration};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use redis::aio::ConnectionManager;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod auth;

#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
    pub redis: Option<ConnectionManager>,
    pub jwt_secret: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ReadinessResponse {
    status: &'static str,
    postgres: &'static str,
    redis: &'static str,
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let state = connect_dependencies().await?;
    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/ready", get(ready))
        .merge(auth::routes())
        .with_state(state);

    let address: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
        .parse()?;
    let listener = TcpListener::bind(address).await?;
    info!(%address, "API listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn connect_dependencies() -> Result<AppState, Box<dyn std::error::Error>> {
    let db = match env::var("DATABASE_URL") {
        Ok(url) => Some(
            PgPoolOptions::new()
                .max_connections(10)
                .acquire_timeout(Duration::from_secs(5))
                .connect_lazy(&url)?,
        ),
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };

    let redis = match env::var("REDIS_URL") {
        Ok(url) => {
            let client = redis::Client::open(url)?;
            Some(ConnectionManager::new_lazy_with_config(
                client,
                redis::aio::ConnectionManagerConfig::new()
                    .set_connection_timeout(Some(Duration::from_secs(2)))
                    .set_response_timeout(Some(Duration::from_secs(2))),
            )?)
        }
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };

    let jwt_secret = env::var("JWT_SECRET")
        .map_err(|_| "JWT_SECRET must be configured")?;
    if jwt_secret.len() < 32 {
        return Err("JWT_SECRET must contain at least 32 bytes".into());
    }

    Ok(AppState { db, redis, jwt_secret })
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn ready(State(state): State<AppState>) -> Response {
    let postgres = match &state.db {
        Some(pool) if sqlx::query("SELECT 1").execute(pool).await.is_ok() => "ok",
        Some(_) => "unavailable",
        None => "not_configured",
    };
    let redis = match state.redis.clone() {
        Some(mut connection) => {
            if redis::cmd("PING")
                .query_async::<String>(&mut connection)
                .await
                .is_ok()
            {
                "ok"
            } else {
                "unavailable"
            }
        }
        None => "not_configured",
    };

    let is_ready = postgres == "ok" && redis == "ok";
    let body = ReadinessResponse {
        status: if is_ready { "ready" } else { "not_ready" },
        postgres,
        redis,
    };

    if is_ready {
        (StatusCode::OK, Json(body)).into_response()
    } else {
        let error = ErrorBody {
            error: ErrorDetail {
                code: "dependencies_unavailable",
                message: "Required dependencies are not ready",
            },
        };
        (StatusCode::SERVICE_UNAVAILABLE, Json(error)).into_response()
    }
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!(%error, "failed to listen for shutdown signal");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_is_live_but_readiness_requires_dependencies() {
        assert_eq!(health().await.0.status, "ok");

        let response = ready(State(AppState {
            db: None,
            redis: None,
            jwt_secret: "x".repeat(32),
        }))
        .await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
