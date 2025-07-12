use actix_web::{post, web, HttpResponse, Result};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::io::Write;
use std::path::Path;
use tracing::{info, error, debug};
use chrono::Utc;
use utoipa::ToSchema;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{UploadResponse, FileUrls, FileMetadata};
use crate::services::image_processor::ImageProcessor;
use crate::services::file_utils::FileManager;
use crate::services::folder_manager::FolderManager;
use crate::utils::validation::{validate_file_type, validate_file_size, sanitize_filename};

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
    debug!("Processing file upload request");
    
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
                    debug!("Received folder_id: {:?}", folder_id);
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

    debug!("Processing file: {} for folder: {:?}", filename, folder_id);

    // Validate folder exists if specified
    if let Some(ref folder_id) = folder_id {
        let folder_contents = folder_manager.list_folder_contents(Some(folder_id.clone())).await;
        if folder_contents.is_err() {
            return Err(AppError::NotFound(format!("Folder with id '{}' not found", folder_id)));
        }
    }

    // Sanitize filename
    let sanitized_filename = sanitize_filename(&filename);
    let unique_filename = file_manager.generate_unique_filename(&sanitized_filename);
    let file_path = file_manager.get_file_path(&unique_filename);

    // Create file and write data
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all(&file_bytes)?;
    file.flush()?;
    drop(file);

    info!("File saved: {} -> {} in folder: {:?}", filename, unique_filename, folder_id);

    // Get file metadata
    let file_size = file_bytes.len() as u64;

    // Assign file to folder
    folder_manager.assign_file_to_folder(&unique_filename, folder_id.clone(), file_size).await?;

    // Validate file type
    let mime_type = validate_file_type(&file_bytes, &unique_filename)?;
    let is_image = ImageProcessor::is_image_file(&unique_filename);

    let uploaded_at = Utc::now();

    // Process image if applicable
    let mut dimensions = None;
    let mut qoi_url = None;
    let mut thumbnail_url = None;

    if is_image {
        debug!("Processing image: {}", unique_filename);
        
        // Get dimensions
        if let Ok(dims) = image_processor.get_dimensions(&file_path).await {
            dimensions = Some((dims.0, dims.1));
            
            // Generate QOI version only if enabled in config
            if config.image.qoi_enabled {
                let path = Path::new(&unique_filename);
                let stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("file");
                
                let qoi_filename = format!("{}.qoi", stem);
                let qoi_path = file_manager.get_file_path(&qoi_filename);
                
                if let Err(e) = image_processor.convert_to_qoi(&file_path, &qoi_path).await {
                    error!("Failed to convert to QOI: {}", e);
                } else {
                    qoi_url = Some(format!("{}/uploads/{}", 
                        config.get_static_base_url(), 
                        qoi_filename
                    ));
                }
            }
            
            // Generate thumbnail
            let path = Path::new(&unique_filename);
            let stem = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            
            let thumb_filename = format!("{}_thumb.webp", stem);
            let thumb_path = file_manager.get_file_path(&thumb_filename);
            
            if let Err(e) = image_processor.generate_thumbnail(&file_path, &thumb_path).await {
                error!("Failed to generate thumbnail: {}", e);
            } else {
                thumbnail_url = Some(format!("{}/uploads/{}", 
                    config.get_static_base_url(), 
                    thumb_filename
                ));
            }
        }
    }

    // Generate URLs
    let urls = FileUrls {
        original: format!("{}/uploads/{}", 
            config.get_static_base_url(), 
            unique_filename
        ),
        qoi: qoi_url,
        thumbnail: thumbnail_url,
    };

    let metadata = FileMetadata {
        size: file_size,
        mime_type,
        uploaded_at,
        width: dimensions.map(|(w, _)| w),
        height: dimensions.map(|(_, h)| h),
    };

    let response = UploadResponse {
        success: true,
        filename: unique_filename,
        urls,
        metadata,
    };

    info!("Upload completed successfully: {} in folder: {:?}", filename, folder_id);
    Ok(HttpResponse::Ok().json(response))
}
