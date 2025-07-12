use thiserror::Error;
use actix_web::{HttpResponse, ResponseError};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("File too large: {0} bytes")]
    FileTooLarge(usize),
    
    #[error("Invalid file type: {0}")]
    InvalidFileType(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Image processing error: {0}")]
    ImageProcessing(#[from] image::ImageError),
    
    #[error("QOI encoding error: {0}")]
    QoiEncoding(String),
    
    #[error("Multipart error: {0}")]
    Multipart(String),
    
    #[allow(dead_code)]
    #[error("Authentication required")]
    AuthenticationRequired,
    
    #[allow(dead_code)]
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::FileTooLarge(_) => HttpResponse::PayloadTooLarge().json(
                serde_json::json!({
                    "error": "File too large",
                    "message": self.to_string()
                })
            ),
            AppError::InvalidFileType(_) => HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": "Invalid file type",
                    "message": self.to_string()
                })
            ),
            AppError::FileNotFound(_) => HttpResponse::NotFound().json(
                serde_json::json!({
                    "error": "File not found",
                    "message": self.to_string()
                })
            ),
            AppError::BadRequest(_) => HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": "Bad request",
                    "message": self.to_string()
                })
            ),
            AppError::NotFound(_) => HttpResponse::NotFound().json(
                serde_json::json!({
                    "error": "Not found",
                    "message": self.to_string()
                })
            ),
            AppError::AuthenticationRequired => HttpResponse::Unauthorized()
                .insert_header(("WWW-Authenticate", "Basic realm=\"SnapFileThing\""))
                .json(
                    serde_json::json!({
                        "error": "Authentication required",
                        "message": "Please provide valid credentials"
                    })
                ),
            AppError::InvalidCredentials => HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Invalid credentials",
                    "message": "Username or password is incorrect"
                })
            ),
            AppError::Unauthorized(_) => HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": self.to_string()
                })
            ),
            AppError::Internal(_) => HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "Internal server error",
                    "message": self.to_string()
                })
            ),
            _ => HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "Internal server error",
                    "message": "An unexpected error occurred"
                })
            ),
        }
    }
}

impl From<actix_multipart::MultipartError> for AppError {
    fn from(err: actix_multipart::MultipartError) -> Self {
        AppError::Multipart(err.to_string())
    }
}
