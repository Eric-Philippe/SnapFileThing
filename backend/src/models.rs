use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UploadResponse {
    pub success: bool,
    pub filename: String,
    pub urls: FileUrls,
    pub metadata: FileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileUrls {
    pub original: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qoi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FileMetadata {
    pub size: u64,
    pub mime_type: String,
    pub uploaded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileInfo {
    pub filename: String,
    pub size: u64,
    pub mime_type: String,
    pub uploaded_at: DateTime<Utc>,
    pub is_image: bool,
    pub urls: FileUrls,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<(u32, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FileListResponse {
    pub files: Vec<FileInfo>,
    pub folders: Vec<FolderInfo>,
    pub current_folder: Option<FolderInfo>,
    pub breadcrumbs: Vec<FolderInfo>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub upload_dir: String,
    pub auth_mode: String,
}

// Auth-related schemas
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    /// JWT access token
    pub access_token: String,
    /// JWT refresh token for getting new access tokens
    pub refresh_token: String,
    /// Type of token (always "Bearer")
    pub token_type: String,
    /// Access token expiration time in seconds
    pub expires_in: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    /// Refresh token to exchange for new access token
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenVerifyResponse {
    /// Whether the token is valid
    pub valid: bool,
    /// Username associated with the token (if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Token expiration timestamp (if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LogoutResponse {
    /// Success message
    pub message: String,
}

// Folder-related models
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FolderInfo {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub file_count: usize,
    pub folder_count: usize,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MoveFolderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FolderListResponse {
    pub folders: Vec<FolderInfo>,
    pub current_folder: Option<FolderInfo>,
    pub breadcrumbs: Vec<FolderInfo>,
}
