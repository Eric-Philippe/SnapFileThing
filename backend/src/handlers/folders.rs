use actix_web::{delete, get, post, put, web, HttpResponse, Result};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{CreateFolderRequest, MoveFolderRequest};
use crate::services::folder_manager::FolderManager;
use tracing::info;

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct FolderQuery {
    /// Parent folder ID (optional, omit for root level)
    folder_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/folders",
    params(FolderQuery),
    responses(
        (status = 200, description = "Folder contents retrieved successfully", body = FolderListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Folder not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Folders"
)]
#[get("/folders")]
pub async fn list_folders(
    query: web::Query<FolderQuery>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    let folder_manager = FolderManager::new(&config.server.upload_dir);
    let response = folder_manager.list_folder_contents(query.folder_id.clone()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/api/folders",
    request_body = CreateFolderRequest,
    responses(
        (status = 201, description = "Folder created successfully", body = FolderInfo),
        (status = 400, description = "Invalid request or folder already exists", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Parent folder not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Folders"
)]
#[post("/folders")]
pub async fn create_folder(
    req: web::Json<CreateFolderRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    let folder_manager = FolderManager::new(&config.server.upload_dir);
    let folder = folder_manager.create_folder(&req.name, req.parent_id.clone()).await?;

    info!("Created folder: {} in parent: {:?}", req.name, req.parent_id);
    Ok(HttpResponse::Created().json(folder))
}

#[utoipa::path(
    delete,
    path = "/api/folders/{folder_id}",
    params(
        ("folder_id" = String, Path, description = "ID of the folder to delete")
    ),
    responses(
        (status = 200, description = "Folder deleted successfully"),
        (status = 400, description = "Folder is not empty", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Folder not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Folders"
)]
#[delete("/folders/{folder_id}")]
pub async fn delete_folder(
    path: web::Path<String>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    let folder_id = path.into_inner();
    let folder_manager = FolderManager::new(&config.server.upload_dir);

    folder_manager.delete_folder(&folder_id).await?;

    info!("Deleted folder: {}", folder_id);
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Folder '{}' deleted successfully", folder_id)
    })))
}

#[utoipa::path(
    put,
    path = "/api/folders/{folder_id}/move",
    request_body = MoveFolderRequest,
    params(
        ("folder_id" = String, Path, description = "ID of the folder to move")
    ),
    responses(
        (status = 200, description = "Folder moved successfully"),
        (status = 400, description = "Invalid move operation (would create circular reference)", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Folder not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Folders"
)]
#[put("/folders/{folder_id}/move")]
pub async fn move_folder(
    path: web::Path<String>,
    req: web::Json<MoveFolderRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    let folder_id = path.into_inner();
    let folder_manager = FolderManager::new(&config.server.upload_dir);
    
    folder_manager.move_folder(&folder_id, req.parent_id.clone()).await?;
    
    info!("Moved folder: {} to parent: {:?}", folder_id, req.parent_id);
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Folder '{}' moved successfully", folder_id)
    })))
}
