use actix_web::{get, delete, put, web, HttpResponse, Result, HttpRequest};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::FileListResponse;
use crate::services::file_utils::FileManager;
use crate::services::folder_manager::FolderManager;
use tracing::{info, warn};
use std::io::Write;

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

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ExportQuery {
    /// Export only original files for images (skip thumbnails and QOI files)
    #[serde(default)]
    originals_only: bool,
    /// Folder ID to export files from (optional, omit for all files)
    folder_id: Option<String>,
}

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

#[utoipa::path(
    get,
    path = "/api/files/export",
    params(ExportQuery),
    responses(
        (status = 200, description = "ZIP archive with files", content_type = "application/zip"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No files found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Files"
)]
#[get("/files/export")]
pub async fn export_files(
    query: web::Query<ExportQuery>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    let file_manager = FileManager::new(
        &config.server.upload_dir,
        config.get_static_base_url(),
    );
    let folder_manager = FolderManager::new(&config.server.upload_dir);

    // Get files to export based on folder filter
    let files_to_export = if let Some(ref folder_id) = query.folder_id {
        folder_manager.get_files_in_folder(Some(folder_id.clone()))?
    } else {
        // Get all files if no folder specified
        let all_files_metadata = folder_manager.get_all_files()?;
        all_files_metadata.into_iter().map(|meta| meta.filename).collect()
    };

    if files_to_export.is_empty() {
        return Err(AppError::NotFound("No files found to export".to_string()));
    }

    // Create ZIP archive in memory
    let mut zip_data = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_data));
        
        // Options for files in ZIP
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        for filename in &files_to_export {
            let file_path = file_manager.get_file_path(filename);
            
            if !file_path.exists() {
                warn!("File not found during export: {}", filename);
                continue;
            }

            // Check if we should skip this file based on originals_only setting
            if query.originals_only {
                // Skip thumbnails and QOI files for images
                if filename.contains("_thumb.") || filename.ends_with(".qoi") {
                    continue;
                }
            }

            // Get folder path for this file to maintain directory structure
            let file_folder_id = folder_manager.get_file_folder(filename).await?;
            let folder_path = if let Some(folder_id) = file_folder_id {
                folder_manager.get_folder_path(&folder_id).await?
            } else {
                String::new() // Root folder
            };

            // Create the full path in ZIP (maintaining folder structure)
            let zip_path = if folder_path.is_empty() {
                filename.clone()
            } else {
                format!("{}/{}", folder_path, filename)
            };

            // Read file content
            let file_content = std::fs::read(&file_path)
                .map_err(|e| AppError::Internal(format!("Failed to read file {}: {}", filename, e)))?;

            // Add file to ZIP
            zip.start_file(&zip_path, options)
                .map_err(|e| AppError::Internal(format!("Failed to start ZIP file entry: {}", e)))?;
            
            zip.write_all(&file_content)
                .map_err(|e| AppError::Internal(format!("Failed to write file to ZIP: {}", e)))?;
        }

        zip.finish()
            .map_err(|e| AppError::Internal(format!("Failed to finalize ZIP archive: {}", e)))?;
    }

    // Generate filename for the ZIP
    let zip_filename = if let Some(ref folder_id) = query.folder_id {
        // Get folder name for ZIP filename
        let folder_info = folder_manager.get_folder_info(folder_id).await?;
        format!("{}_export.zip", folder_info.name)
    } else {
        "all_files_export.zip".to_string()
    };

    let export_type = if query.originals_only { "originals" } else { "all files" };
    info!("Exported {} files ({}) to ZIP: {} files", files_to_export.len(), export_type, zip_filename);

    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", zip_filename)))
        .body(zip_data))
}
