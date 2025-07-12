use actix_web::{get, HttpResponse, Result, web};
use crate::config::AppConfig;
use crate::models::HealthResponse;
use std::time::{SystemTime, UNIX_EPOCH};

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse),
    ),
    tag = "Health"
)]
#[get("/health")]
pub async fn health_check(config: web::Data<AppConfig>) -> Result<HttpResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
        upload_dir: config.server.upload_dir.clone(),
        auth_mode: config.auth.mode.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}
