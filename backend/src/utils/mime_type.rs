use std::path::Path;

/// Get MIME type based on file extension
pub fn get_mime_type(filename: &str) -> String {
    let extension = Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    match extension.as_deref() {
        // Images
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("tiff") | Some("tif") => "image/tiff",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        
        // Documents
        Some("pdf") => "application/pdf",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        
        // Text
        Some("txt") => "text/plain",
        Some("csv") => "text/csv",
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        
        // Archives
        Some("zip") => "application/zip",
        Some("rar") => "application/vnd.rar",
        Some("7z") => "application/x-7z-compressed",
        Some("tar") => "application/x-tar",
        Some("gz") => "application/gzip",
        
        // Audio
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("flac") => "audio/flac",
        
        // Video
        Some("mp4") => "video/mp4",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("wmv") => "video/x-ms-wmv",
        Some("flv") => "video/x-flv",
        Some("webm") => "video/webm",
        Some("mkv") => "video/x-matroska",
        
        // Default
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Check if a MIME type represents an image
#[allow(dead_code)]
pub fn is_image_mime_type(mime_type: &str) -> bool {
    mime_type.starts_with("image/")
}

/// Get file extension from MIME type
#[allow(dead_code)]
pub fn get_extension_from_mime(mime_type: &str) -> Option<&'static str> {
    match mime_type {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/bmp" => Some("bmp"),
        "image/tiff" => Some("tiff"),
        "text/plain" => Some("txt"),
        "application/pdf" => Some("pdf"),
        "application/zip" => Some("zip"),
        "audio/mpeg" => Some("mp3"),
        "video/mp4" => Some("mp4"),
        _ => None,
    }
}
