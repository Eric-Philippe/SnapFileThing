use crate::config::AppConfig;
use crate::error::AppError;
use crate::services::file_utils::FileManager;
use crate::services::folder_manager::FolderManager;
use crate::services::image_processor::ImageProcessor;
use crate::utils::validation::{validate_file_type, validate_file_size, sanitize_filename};
use chrono::{DateTime, Utc};
use std::path::Path;

/// Shared logic for processing and saving an uploaded file (from upload or import)
pub async fn process_uploaded_file(
    file_bytes: Vec<u8>,
    original_filename: &str,
    folder_id: Option<String>,
    config: &AppConfig,
    file_manager: &FileManager,
    folder_manager: &FolderManager,
    image_processor: &ImageProcessor,
) -> Result<(String, DateTime<Utc>, u64), AppError> {
    // Validate file size
    validate_file_size(file_bytes.len(), config.server.max_file_size)?;
    // Sanitize filename
    let sanitized_filename = sanitize_filename(original_filename);
    let unique_filename = file_manager.generate_unique_filename(&sanitized_filename);
    let file_path = file_manager.get_file_path(&unique_filename);
    // Write file
    std::fs::write(&file_path, &file_bytes)?;
    // Validate file type
    let _mime_type = validate_file_type(&file_bytes, &unique_filename)?;
    // Assign file to folder
    let file_size = file_bytes.len() as u64;
    folder_manager.assign_file_to_folder(&unique_filename, folder_id.clone(), file_size).await?;
    // Image processing
    if ImageProcessor::is_image_file(&unique_filename) {
        let stem = Path::new(&unique_filename).file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        if config.image.qoi_enabled {
            let qoi_filename = format!("{}.qoi", stem);
            let qoi_path = file_manager.get_file_path(&qoi_filename);
            let _ = image_processor.convert_to_qoi(&file_path, &qoi_path).await;
        }
        let thumb_filename = format!("{}_thumb.webp", stem);
        let thumb_path = file_manager.get_file_path(&thumb_filename);
        let _ = image_processor.generate_thumbnail(&file_path, &thumb_path).await;
    }
    let uploaded_at = Utc::now();
    Ok((unique_filename, uploaded_at, file_size))
}
