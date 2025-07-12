use utoipa::OpenApi;
use utoipa::{Modify, openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder}};
use crate::handlers::{health, upload, files, auth, folders};
use crate::models::{
    UploadResponse, FileListResponse, HealthResponse, ErrorResponse,
    FileUrls, FileMetadata, FileInfo, LoginRequest, LoginResponse,
    RefreshRequest, TokenVerifyResponse, LogoutResponse, FolderInfo,
    CreateFolderRequest, FolderListResponse, MoveFolderRequest
};
use crate::handlers::files::{ListQuery, ExportQuery, MoveFileRequest};
use crate::handlers::folders::FolderQuery;
use crate::handlers::upload::FileUploadRequest;
use crate::handlers::auth::Claims;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        // Health endpoints
        health::health_check,
        
        // Authentication endpoints  
        auth::login,
        auth::logout,
        auth::refresh_token,
        auth::verify_token,
        
        // File management endpoints
        upload::upload_file,
        files::list_files,
        files::delete_file,
        files::move_file,
        files::export_files,
        
        // Folder management endpoints
        folders::list_folders,
        folders::create_folder,
        folders::delete_folder,
    ),
    components(
        schemas(
            // Response models
            UploadResponse,
            FileListResponse,
            HealthResponse,
            ErrorResponse,
            FileUrls,
            FileMetadata,
            FileInfo,
            
            // Authentication models
            LoginRequest,
            LoginResponse,
            RefreshRequest,
            TokenVerifyResponse,
            LogoutResponse,
            Claims,
            
            // Folder models
            FolderInfo,
            CreateFolderRequest,
            MoveFolderRequest,
            FolderListResponse,
            
            // Request models
            ListQuery,
            ExportQuery,
            MoveFileRequest,
            FolderQuery,
            FileUploadRequest
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Authentication", description = "Authentication and authorization endpoints"),
        (name = "Files", description = "File upload, listing, and management endpoints"),
        (name = "Folders", description = "Folder creation, listing, and management endpoints")
    ),
    info(
        title = "SnapFileThing API",
        version = "0.1.0",
        description = "A lightweight file hosting service with QOI conversion, thumbnails, and JWT authentication.\n\n## Authentication\n\nThis API uses JWT (JSON Web Token) for authentication. To access protected endpoints:\n\n1. **Login**: Send your credentials to `/api/auth/login` to get access and refresh tokens\n2. **Use Bearer Token**: Include the access token in the `Authorization` header as `Bearer <token>`\n3. **Refresh Token**: When the access token expires, use the refresh token at `/api/auth/refresh` to get new tokens\n\n## File Operations\n\nAll file operations require authentication. You can:\n- Upload files (supports images with automatic QOI conversion and thumbnail generation)\n- List uploaded files with pagination\n- Delete files by filename\n\n## Public Endpoints\n\n- Health check (`/api/health`) - No authentication required\n- Authentication endpoints (`/api/auth/*`) - Login and refresh don't require existing authentication",
        contact(
            name = "API Support",
            email = "ericphlpp@proton.me",
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct ApiDoc;
