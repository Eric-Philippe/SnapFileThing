use actix_web::{delete, get, put, web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{ErrorResponse, FileListResponse};
use crate::services::folder_manager::FolderManager;
use crate::services::file_utils::FileManager;

// Re-export handlers and their OpenAPI paths
pub use crate::handlers::export::{export_files, __path_export_files};
pub use crate::handlers::import::{ImportRequest, import_files, __path_import_files};



#[utoipa::path(
    get,
    path = "/api/files",
    params(ListQuery),
    responses(
        (status = 200, description = "List of files retrieved successfully", body = FileListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Files"
)]
#[get("/files")]
pub async fn list_files(
    query: web::Query<ListQuery>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(0);
    let per_page = std::cmp::min(query.per_page.unwrap_or(20), 100); // Max 100 items per page

    let file_manager = FileManager::new(
        &config.server.upload_dir,
        config.get_static_base_url(),
    );
    let folder_manager = FolderManager::new(&config.server.upload_dir);

    // Get folder information
    let folder_response = folder_manager.list_folder_contents(query.folder_id.clone()).await?;

    // Get files in the specified folder
    let files_in_folder = folder_manager.get_files_in_folder(query.folder_id.clone())?;
    // If querying root (None), filter only files with folder_id == None
    let files_in_folder = if query.folder_id.is_none() {
        let file_metadata = folder_manager.load_file_metadata()?;
        file_metadata
            .iter()
            .filter_map(|(filename, meta)| {
                // Accept files with folder_id == None or folder_id == "root"
                if meta.folder_id.is_none() || meta.folder_id.as_deref() == Some("root") {
                    Some(filename.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        files_in_folder
    };
    let (files, total) = file_manager.list_files_with_filter(page, per_page, Some(files_in_folder)).await?;
    
    let total_pages = if per_page > 0 {
        (total + per_page - 1) / per_page
    } else {
        0
    };

    // Add folder_id to each file info
    let mut files_with_folder = Vec::new();
    for mut file in files {
        file.folder_id = folder_manager.get_file_folder(&file.filename).await?;
        files_with_folder.push(file);
    }

    let response = FileListResponse {
        files: files_with_folder,
        folders: folder_response.folders,
        current_folder: folder_response.current_folder,
        breadcrumbs: folder_response.breadcrumbs,
        total,
        page,
        per_page,
        total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ListQuery {
    /// Page number (0-based)
    page: Option<usize>,
    /// Number of items per page (max 100)
    per_page: Option<usize>,
    /// Folder ID to filter files (optional, omit for root level)
    folder_id: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct MoveFileRequest {
    /// Target folder ID (optional, use None for root folder)
    folder_id: Option<String>,
}

#[derive(Deserialize, IntoParams, ToSchema, Clone)]
pub struct ExportQuery {
    /// Folder ID to export files from (optional, omit for all files)
    pub folder_id: Option<String>,
}



#[utoipa::path(
    delete,
    path = "/api/files/{filename}",
    params(
        ("filename" = String, Path, description = "Name of the file to delete")
    ),
    responses(
        (status = 200, description = "File deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "File not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Files"
)]
#[delete("/files/{filename}")]
pub async fn delete_file(
    path: web::Path<String>,
    config: web::Data<AppConfig>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let filename = path.into_inner();
    
    // Log the deletion attempt
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    info!("File deletion requested: {} (User-Agent: {})", filename, user_agent);

    let file_manager = FileManager::new(
        &config.server.upload_dir,
        config.get_static_base_url(),
    );
    let folder_manager = FolderManager::new(&config.server.upload_dir);

    // First, try to find the actual file by the provided filename
    let actual_filename = if file_manager.file_exists(&filename) {
        // Exact filename exists
        filename.clone()
    } else {
        // Try to find a file that starts with the provided filename (stem matching)
        match file_manager.find_file_by_stem(&filename).await? {
            Some(found_filename) => found_filename,
            None => {
                warn!("No file found matching stem: {}", filename);
                return Err(AppError::FileNotFound(filename));
            }
        }
    };

    // Delete the file
    file_manager.delete_file(&actual_filename).await?;
    
    // Remove file metadata
    folder_manager.remove_file_metadata(&actual_filename).await?;
    
    info!("File deleted successfully: {} (original request: {})", actual_filename, filename);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("File '{}' and related files deleted successfully", actual_filename)
    })))
}

#[utoipa::path(
    put,
    path = "/api/files/{filename}/move",
    request_body = MoveFileRequest,
    params(
        ("filename" = String, Path, description = "Name of the file to move")
    ),
    responses(
        (status = 200, description = "File moved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "File or folder not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Files"
)]
#[put("/files/{filename}/move")]
pub async fn move_file(
    path: web::Path<String>,
    req: web::Json<MoveFileRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    let filename = path.into_inner();
    
    let file_manager = FileManager::new(
        &config.server.upload_dir,
        config.get_static_base_url(),
    );
    let folder_manager = FolderManager::new(&config.server.upload_dir);

    // First, check if the file exists
    let actual_filename = if file_manager.file_exists(&filename) {
        filename.clone()
    } else {
        // Try to find a file that starts with the provided filename (stem matching)
        match file_manager.find_file_by_stem(&filename).await? {
            Some(found_filename) => found_filename,
            None => {
                warn!("No file found matching stem: {}", filename);
                return Err(AppError::FileNotFound(filename));
            }
        }
    };

    // Get current file size for the folder assignment
    let file_size = file_manager.get_file_size(&actual_filename)?;

    // Move the file by updating its folder assignment
    folder_manager.assign_file_to_folder(&actual_filename, req.folder_id.clone(), file_size).await?;
    
    info!("File moved successfully: {} to folder: {:?}", actual_filename, req.folder_id);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("File '{}' moved successfully", actual_filename)
    })))
}


