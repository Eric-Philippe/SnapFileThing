use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{LoginRequest, LoginResponse, RefreshRequest, TokenVerifyResponse, LogoutResponse, ErrorResponse};

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    /// Subject (username)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// JWT ID for tracking
    pub jti: String,
    /// Token type: "access" or "refresh"
    pub token_type: String,
}

// Token blacklist
type TokenBlacklist = Arc<Mutex<HashSet<String>>>;

// JWT service for token operations
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_duration: Duration,
    refresh_token_duration: Duration,
    blacklist: TokenBlacklist,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        Self {
            encoding_key,
            decoding_key,
            access_token_duration: Duration::hours(1),     // 1 hour for access tokens
            refresh_token_duration: Duration::days(7),     // 7 days for refresh tokens
            blacklist: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn create_access_token(&self, username: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims {
            sub: username.to_string(),
            exp: (now + self.access_token_duration).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| {
                error!("Failed to create access token: {}", e);
                AppError::Internal("Failed to create access token".to_string())
            })
    }

    pub fn create_refresh_token(&self, username: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims {
            sub: username.to_string(),
            exp: (now + self.refresh_token_duration).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| {
                error!("Failed to create refresh token: {}", e);
                AppError::Internal("Failed to create refresh token".to_string())
            })
    }

    pub fn validate_token(&self, token: &str) -> Result<TokenData<Claims>, AppError> {
        // Check if token is blacklisted
        if let Ok(blacklist) = self.blacklist.lock() {
            if blacklist.contains(token) {
                return Err(AppError::Unauthorized("Token has been revoked".to_string()));
            }
        }

        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map_err(|e| {
                warn!("Token validation failed: {}", e);
                AppError::Unauthorized("Invalid token".to_string())
            })
    }

    pub fn blacklist_token(&self, token: &str) -> Result<(), AppError> {
        if let Ok(mut blacklist) = self.blacklist.lock() {
            blacklist.insert(token.to_string());
            Ok(())
        } else {
            error!("Failed to acquire blacklist lock");
            Err(AppError::Internal("Failed to blacklist token".to_string()))
        }
    }

    pub fn get_access_token_duration_seconds(&self) -> i64 {
        self.access_token_duration.num_seconds()
    }
}

/// Authenticate user and return JWT tokens
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 429, description = "Too many requests", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn login(
    request: web::Json<LoginRequest>,
    config: web::Data<AppConfig>,
    jwt_service: web::Data<JwtService>,
) -> Result<HttpResponse, AppError> {
    // Validate credentials using constant-time comparison
    let username_valid = constant_time_eq::constant_time_eq(
        config.auth.admin_username.as_bytes(),
        request.username.as_bytes(),
    );
    let password_valid = constant_time_eq::constant_time_eq(
        config.auth.admin_password.as_bytes(),
        request.password.as_bytes(),
    );

    if !username_valid || !password_valid {
        warn!("Failed login attempt for username: {}", request.username);
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate tokens
    let access_token = jwt_service.create_access_token(&request.username)?;
    let refresh_token = jwt_service.create_refresh_token(&request.username)?;

    info!("Successful login for user: {}", request.username);

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_service.get_access_token_duration_seconds(),
    }))
}

/// Logout user and invalidate token
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn logout(
    req: HttpRequest,
    jwt_service: web::Data<JwtService>,
) -> Result<HttpResponse, AppError> {
    // Extract token from Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                
                // Validate token first to ensure it's properly formatted
                match jwt_service.validate_token(token) {
                    Ok(token_data) => {
                        // Add token to blacklist
                        jwt_service.blacklist_token(token)?;
                        info!("User {} logged out successfully", token_data.claims.sub);
                        
                        return Ok(HttpResponse::Ok().json(LogoutResponse {
                            message: "Logged out successfully".to_string(),
                        }));
                    }
                    Err(_) => {
                        // Token is invalid, but we can still respond with success
                        // to avoid leaking information about token validity
                        return Ok(HttpResponse::Ok().json(LogoutResponse {
                            message: "Logged out successfully".to_string(),
                        }));
                    }
                }
            }
        }
    }

    // No valid token found, but still return success
    Ok(HttpResponse::Ok().json(LogoutResponse {
        message: "Logged out successfully".to_string(),
    }))
}

/// Refresh access token using refresh token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = LoginResponse),
        (status = 401, description = "Invalid refresh token", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn refresh_token(
    request: web::Json<RefreshRequest>,
    jwt_service: web::Data<JwtService>,
) -> Result<HttpResponse, AppError> {
    // Validate refresh token
    let token_data = jwt_service.validate_token(&request.refresh_token)?;
    
    // Ensure it's a refresh token
    if token_data.claims.token_type != "refresh" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    // Blacklist the old refresh token
    jwt_service.blacklist_token(&request.refresh_token)?;

    // Create new tokens
    let access_token = jwt_service.create_access_token(&token_data.claims.sub)?;
    let refresh_token = jwt_service.create_refresh_token(&token_data.claims.sub)?;

    info!("Token refreshed for user: {}", token_data.claims.sub);

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_service.get_access_token_duration_seconds(),
    }))
}

/// Verify if the provided token is valid
#[utoipa::path(
    get,
    path = "/api/auth/verify",
    responses(
        (status = 200, description = "Token verification result", body = TokenVerifyResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn verify_token(
    req: HttpRequest,
    jwt_service: web::Data<JwtService>,
) -> Result<HttpResponse, AppError> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                
                match jwt_service.validate_token(token) {
                    Ok(token_data) => {
                        if token_data.claims.token_type == "access" {
                            return Ok(HttpResponse::Ok().json(TokenVerifyResponse {
                                valid: true,
                                username: Some(token_data.claims.sub),
                                expires_at: Some(token_data.claims.exp),
                            }));
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(TokenVerifyResponse {
        valid: false,
        username: None,
        expires_at: None,
    }))
}
