use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse};
use futures_util::StreamExt;
use utoipa::ToSchema;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{ErrorResponse, FileMetadata, UploadResponse, FileUrls};
use crate::services::file_upload::process_uploaded_file;
use crate::services::file_utils::FileManager;
use crate::services::folder_manager::FolderManager;
use crate::services::image_processor::ImageProcessor;
use crate::utils::validation::validate_file_size;

#[derive(ToSchema)]
#[allow(dead_code)]
pub struct FileUploadRequest {
    #[schema(format = "binary")]
    file: Vec<u8>,
    folder_id: Option<String>,
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
    let mut file_field = None;
    let mut folder_id = None;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        
        // Get field name first
        let name = field.name().ok_or_else(|| AppError::BadRequest("Invalid field".to_string()))?.to_string();
        
        match name.as_str() {
            "file" => {
                // Then get content disposition
                let filename = if let Some(cd) = field.content_disposition() {
                    cd.get_filename().ok_or_else(|| AppError::BadRequest("Filename missing".to_string()))?.to_string()
                } else {
                    return Err(AppError::BadRequest("Content disposition missing".to_string()));
                };
                
                // Finally read the data
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk?);
                }
                validate_file_size(data.len(), config.server.max_file_size)?;
                file_field = Some((filename, data));
            },
            "folder_id" => {
                let mut folder_data = String::new();
                while let Some(chunk) = field.next().await {
                    let chunk_bytes = chunk?;
                    let chunk_str = std::str::from_utf8(&chunk_bytes)
                        .map_err(|e| AppError::BadRequest(format!("Invalid UTF-8 in folder_id: {}", e)))?;
                    folder_data.push_str(chunk_str);
                }
                if !folder_data.is_empty() {
                    folder_id = Some(folder_data);
                }
            },
            _ => continue,
        }
    }
    
    // Process the file if we have one
    if let Some((filename, data)) = file_field {
        let file_manager = FileManager::new(&config.server.upload_dir, config.server.base_url.clone().unwrap_or_default());
        let folder_manager = FolderManager::new(&config.server.upload_dir);
        let image_processor = ImageProcessor::new(config.image.clone());
        
        let (unique_filename, uploaded_at, file_size) = process_uploaded_file(
            data,
            &filename,
            folder_id,
            &config,
            &file_manager,
            &folder_manager,
            &image_processor,
        ).await?;
        
        // Generate URLs and response
        let base_url = config.server.base_url.as_deref().unwrap_or("http://localhost:8080");
        let stem = unique_filename.rsplit('.').nth(1).unwrap_or("file");
        
        let response = UploadResponse {
            success: true,
            filename: unique_filename.clone(),
            urls: FileUrls { 
                original: format!("{}/uploads/{}", base_url, unique_filename),
                qoi: if config.image.qoi_enabled && ImageProcessor::is_image_file(&unique_filename) {
                    Some(format!("{}/uploads/{}.qoi", base_url, stem))
                } else {
                    None
                },
                thumbnail: if ImageProcessor::is_image_file(&unique_filename) {
                    Some(format!("{}/uploads/{}_thumb.webp", base_url, stem))
                } else {
                    None
                }
            },
            metadata: FileMetadata { 
                size: file_size,
                mime_type: "application/octet-stream".to_string(), // TODO: Implement proper MIME type detection
                uploaded_at,
                width: None, // TODO: Add image dimensions if it's an image
                height: None 
            }
        };
        
        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(AppError::BadRequest("No file provided".to_string()))
    }
}
