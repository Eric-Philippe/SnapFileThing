use crate::error::AppError;

const MAX_MAGIC_BYTES: usize = 12;

/// Validate file type by checking magic numbers/file signatures
pub fn validate_file_type(data: &[u8], filename: &str) -> Result<String, AppError> {
    let magic_bytes = &data[..std::cmp::min(data.len(), MAX_MAGIC_BYTES)];
    
    // Check magic numbers
    let detected_type = detect_file_type(magic_bytes);
    
    // If we couldn't detect the type and it's supposed to be an image, reject it
    if detected_type.is_none() {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());
        
        if matches!(
            extension.as_deref(),
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | 
            Some("bmp") | Some("tiff") | Some("tif") | Some("webp")
        ) {
            return Err(AppError::InvalidFileType(
                "File claims to be an image but has invalid format".to_string()
            ));
        }
    }
    
    Ok(detected_type.unwrap_or_else(|| {
        crate::utils::mime_type::get_mime_type(filename)
    }))
}

/// Detect file type based on magic numbers
fn detect_file_type(data: &[u8]) -> Option<String> {
    if data.len() < 4 {
        return None;
    }
    
    match data {
        // JPEG
        [0xFF, 0xD8, 0xFF, ..] => Some("image/jpeg".to_string()),
        
        // PNG
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some("image/png".to_string()),
        
        // GIF87a or GIF89a
        [0x47, 0x49, 0x46, 0x38, 0x37, 0x61, ..] |
        [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, ..] => Some("image/gif".to_string()),
        
        // WebP
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50] => Some("image/webp".to_string()),
        
        // BMP
        [0x42, 0x4D, ..] => Some("image/bmp".to_string()),
        
        // TIFF (little endian)
        [0x49, 0x49, 0x2A, 0x00, ..] => Some("image/tiff".to_string()),
        
        // TIFF (big endian)
        [0x4D, 0x4D, 0x00, 0x2A, ..] => Some("image/tiff".to_string()),
        
        // PDF
        [0x25, 0x50, 0x44, 0x46, ..] => Some("application/pdf".to_string()),
        
        // ZIP
        [0x50, 0x4B, 0x03, 0x04, ..] |
        [0x50, 0x4B, 0x05, 0x06, ..] |
        [0x50, 0x4B, 0x07, 0x08, ..] => Some("application/zip".to_string()),
        
        // RAR
        [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00, ..] => Some("application/vnd.rar".to_string()),
        
        // 7z
        [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, ..] => Some("application/x-7z-compressed".to_string()),
        
        // MP3
        [0xFF, 0xFB, ..] |
        [0xFF, 0xF3, ..] |
        [0xFF, 0xF2, ..] |
        [0x49, 0x44, 0x33, ..] => Some("audio/mpeg".to_string()),
        
        // MP4
        [_, _, _, _, 0x66, 0x74, 0x79, 0x70, ..] => {
            // Check if it's MP4
            if data.len() >= 12 {
                match &data[8..12] {
                    [0x6D, 0x70, 0x34, 0x31] |  // mp41
                    [0x6D, 0x70, 0x34, 0x32] |  // mp42
                    [0x69, 0x73, 0x6F, 0x6D] |  // isom
                    [0x69, 0x73, 0x6F, 0x32] => Some("video/mp4".to_string()),  // iso2
                    _ => None,
                }
            } else {
                None
            }
        },
        
        _ => None,
    }
}

/// Validate file size
pub fn validate_file_size(size: usize, max_size: usize) -> Result<(), AppError> {
    if size > max_size {
        return Err(AppError::FileTooLarge(size));
    }
    Ok(())
}

/// Sanitize filename to prevent directory traversal attacks and normalize the name
pub fn sanitize_filename(filename: &str) -> String {
    // Split filename into name and extension
    let path = std::path::Path::new(filename);
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();
    
    let name = path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(filename);
    
    // Remove path separators and other dangerous characters, convert to lowercase
    let mut sanitized_name = name
        .to_lowercase()
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect::<String>();
    
    // Replace whitespace and other problematic characters with underscores
    sanitized_name = sanitized_name
        .chars()
        .map(|c| {
            if c.is_whitespace() || matches!(c, ' ' | '\t' | '\n' | '\r') {
                '_'
            } else if c.is_ascii_punctuation() && !matches!(c, '-' | '_' | '.') {
                '_'
            } else {
                c
            }
        })
        .collect::<String>();
    
    // Remove multiple consecutive underscores
    while sanitized_name.contains("__") {
        sanitized_name = sanitized_name.replace("__", "_");
    }
    
    // Remove leading/trailing underscores
    sanitized_name = sanitized_name.trim_matches('_').to_string();
    
    // Ensure filename is not empty and doesn't start with a dot (hidden file)
    if sanitized_name.is_empty() || sanitized_name.starts_with('.') {
        sanitized_name = format!("file_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    }
    
    // Reconstruct filename with extension
    if extension.is_empty() {
        sanitized_name
    } else {
        format!("{}.{}", sanitized_name, extension)
    }
}
