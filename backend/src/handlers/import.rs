use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::io::Cursor;

use crate::AppConfig;
use crate::error::AppError;
use crate::models::ErrorResponse;
use crate::services::folder_manager::FolderManager;

#[utoipa::path(
    post,
    path = "/api/files/import",
    request_body(content = ImportRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Files imported successfully"),
        (status = 400, description = "Invalid ZIP file or upload error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Files"
)]
#[post("/files/import")]
pub async fn import_files(
    mut payload: Multipart,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    let mut zip_data = Vec::new();
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            AppError::BadRequest(format!("Multipart error: {e}"))
        })?;
        let content_disposition = field.content_disposition();
        
        if let Some(cd) = content_disposition {
            if let Some(filename) = cd.get_filename() {
                if filename.ends_with(".zip") {
                    while let Some(chunk) = field.next().await {
                        let data = chunk.map_err(|e| {
                            AppError::BadRequest(format!("Upload error: {e}"))
                        })?;
                        zip_data.extend_from_slice(&data);
                    }
                    break;
                }
            }
        }
    }
    if zip_data.is_empty() {
        return Err(AppError::BadRequest("No ZIP file uploaded".to_string()));
    }

    // Remove all existing files/folders in upload dir
    let upload_dir = &config.server.upload_dir;
    if std::path::Path::new(upload_dir).exists() {
        std::fs::remove_dir_all(upload_dir).map_err(|e| {
            AppError::Internal(format!("Failed to clear upload dir: {e}"))
        })?;
    }
    std::fs::create_dir_all(upload_dir).map_err(|e| {
        AppError::Internal(format!("Failed to recreate upload dir: {e}"))
    })?;

    // Unzip the uploaded ZIP file into a temp dir
    use tempfile::tempdir;
    use zip::ZipArchive;
    let temp_dir = tempdir().map_err(|e| AppError::Internal(format!("Failed to create temp dir: {e}")))?;
    let mut zip = ZipArchive::new(Cursor::new(&zip_data)).map_err(|e| {
        AppError::BadRequest(format!("Invalid ZIP file: {e}"))
    })?;
    zip.extract(temp_dir.path()).map_err(|e| AppError::Internal(format!("Failed to extract ZIP: {e}")))?;

    // Traverse the unzipped directory: collect folders and files
    use walkdir::WalkDir;
    let mut folders = Vec::new();
    let mut files = Vec::new();
    for entry in WalkDir::new(temp_dir.path()).min_depth(1) {
        let entry = entry.map_err(|e| AppError::Internal(format!("WalkDir error: {e}")))?;
        let rel_path = entry.path().strip_prefix(temp_dir.path()).unwrap();
        if entry.file_type().is_dir() {
            folders.push(rel_path.to_path_buf());
        } else if entry.file_type().is_file() {
            files.push(rel_path.to_path_buf());
        }
    }

    // Sort folders by depth (parents first)
    folders.sort_by_key(|p| p.components().count());

    // Create FolderManager
    let folder_manager = FolderManager::new(upload_dir);

    // Map of rel_path -> folder_id
    let mut folder_ids: HashMap<std::path::PathBuf, String> = HashMap::new();
    folder_ids.insert(std::path::PathBuf::new(), None::<String>.map_or(String::new(), |s| s)); // root

    // Create folders
    for folder in &folders {
        let parent = folder.parent().unwrap_or(std::path::Path::new(""));
        let parent_id = if parent.as_os_str().is_empty() {
            None
        } else {
            folder_ids.get(parent).cloned()
        };
        let name = folder.file_name().unwrap().to_string_lossy();
        let info = folder_manager.create_folder(&name, parent_id.clone()).await?;
        folder_ids.insert(folder.clone(), info.id.clone());
    }

    // Copy files and assign to folders (flat, no physical subfolders)
    use crate::services::file_utils::FileManager;
    use crate::services::image_processor::ImageProcessor;
    use crate::services::file_upload::process_uploaded_file;
    let file_manager = FileManager::new(upload_dir, config.get_static_base_url());
    let image_processor = ImageProcessor::new(config.image.clone());

    for file in &files {
        let src_path = temp_dir.path().join(file);
        let folder = file.parent().unwrap_or(std::path::Path::new(""));
        let folder_id = if folder.as_os_str().is_empty() {
            None
        } else {
            folder_ids.get(folder).cloned()
        };
        let filename = file.file_name().unwrap().to_string_lossy();
        let file_bytes = std::fs::read(&src_path).map_err(|e| AppError::Internal(format!("Failed to read file: {e}")))?;
        // Write file and update metadata (flat in uploads/)
        let _ = process_uploaded_file(
            file_bytes,
            &filename,
            folder_id,
            &config,
            &file_manager,
            &folder_manager,
            &image_processor,
        ).await?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Files imported and upload folder rebuilt successfully"
    })))
}

/// ImportRequest for OpenAPI (multipart/form-data with a file)
#[allow(dead_code)]
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ImportRequest {
    /// ZIP file to import (binary data)
    #[schema(format = "binary")]
    pub file: String,
}
