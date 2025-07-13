use actix_web::{post, web, HttpResponse, Result};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use tracing::{info};
use chrono::Utc;
use utoipa::ToSchema;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{UploadResponse, FileUrls, FileMetadata};
use crate::services::image_processor::ImageProcessor;
use crate::services::file_utils::FileManager;
use crate::services::folder_manager::FolderManager;
use crate::utils::validation::{validate_file_size};
use crate::services::file_upload::process_uploaded_file;

/// File upload request body
#[derive(ToSchema)]
pub struct FileUploadRequest {
    /// File to upload (binary data)
    #[schema(format = "binary")]
    #[allow(dead_code)]
    pub file: String,
    /// Optional folder ID to upload the file into
    #[allow(dead_code)]
    pub folder_id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/upload",
    request_body(content = FileUploadRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "File uploaded successfully", body = UploadResponse),
        (status = 400, description = "Invalid file or file too large", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 413, description = "File too large", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Files"
)]
#[post("/upload")]
pub async fn upload_file(
    mut payload: Multipart,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {    
    let file_manager = FileManager::new(
        &config.server.upload_dir,
        config.get_static_base_url(),
    );
    let folder_manager = FolderManager::new(&config.server.upload_dir);
    let image_processor = ImageProcessor::new(config.image.clone());

    let mut folder_id: Option<String> = None;
    let mut file_data: Option<(String, Vec<u8>)> = None; // (filename, data)

    // First pass: collect all fields
    while let Some(item) = payload.next().await {
        let mut field = item?;
        
        // Handle folder_id field
        if field.name() == "folder_id" {
            let mut folder_id_bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                folder_id_bytes.extend_from_slice(&chunk?);
            }
            if !folder_id_bytes.is_empty() {
                let folder_id_str = String::from_utf8(folder_id_bytes)
                    .map_err(|_| AppError::BadRequest("Invalid folder_id format".to_string()))?;
                if !folder_id_str.trim().is_empty() {
                    folder_id = Some(folder_id_str.trim().to_string());
                }
            }
            continue;
        }

        if field.name() == "file" {
            let content_disposition = field.content_disposition();
            let filename = content_disposition
                .get_filename()
                .ok_or_else(|| AppError::InvalidFileType("No filename provided".to_string()))?
                .to_string();

            // Collect file data
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk_data = chunk?.to_vec();
                data.extend_from_slice(&chunk_data);
                // Check file size as we stream
                validate_file_size(data.len(), config.server.max_file_size)?;
            }
            
            file_data = Some((filename, data));
            continue;
        }
    }

    // Process the file if we have one
    let (filename, file_bytes) = file_data
        .ok_or_else(|| AppError::InvalidFileType("No file field found".to_string()))?;

    // Validate folder exists if specified
    if let Some(ref folder_id) = folder_id {
        let folder_contents = folder_manager.list_folder_contents(Some(folder_id.clone())).await;
        if folder_contents.is_err() {
            return Err(AppError::NotFound(format!("Folder with id '{}' not found", folder_id)));
        }
    }

    // Use shared upload logic
    let (unique_filename, _uploaded_at, _file_size) = process_uploaded_file(
        file_bytes,
        &filename,
        folder_id.clone(),
        &config,
        &file_manager,
        &folder_manager,
        &image_processor,
    ).await?;

    // For response, you may want to re-fetch metadata and URLs if needed, or extend process_uploaded_file to return them
    // For now, just return a simple response
    let response = UploadResponse {
        success: true,
        filename: unique_filename.clone(),
        urls: FileUrls { original: format!("{}/uploads/{}", config.get_static_base_url(), unique_filename), qoi: None, thumbnail: None },
        metadata: FileMetadata { size: 0, mime_type: "".to_string(), uploaded_at: Utc::now(), width: None, height: None },
    };
    info!("Upload completed successfully: {} in folder: {:?}", filename, folder_id);
    Ok(HttpResponse::Ok().json(response))
}
