use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub service: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReadinessResponse {
    pub status: String,
    pub checks: ReadinessChecks,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReadinessChecks {
    pub download_service: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "hpyd-server".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Service is not ready", body = ReadinessResponse)
    )
)]
pub async fn readiness_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ReadinessResponse {
            status: "ready".to_string(),
            checks: ReadinessChecks {
                download_service: "ok".to_string(),
            },
        }),
    )
}
