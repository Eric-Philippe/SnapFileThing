use serde::{Deserialize, Serialize};
use std::env;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub image: ImageConfig,
    pub cors: CorsConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub web_port: u16,
    pub static_port: u16,
    pub upload_dir: String,
    pub max_file_size: usize,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub mode: String, // "protected" or "local"
    pub admin_username: String,
    pub admin_password: String,
    pub jwt_secret: String,
    pub disabled_routes: Vec<String>, // Routes that don't require authentication
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub thumbnail_size: u32,
    pub jpeg_quality: u8,
    pub webp_quality: f32,
    pub qoi_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub auth_routes: RateLimitRule,
    pub upload_routes: RateLimitRule,
    pub static_routes: RateLimitRule,
    pub disabled_routes: Vec<String>, // Routes without rate limiting
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                web_port: 8080,
                static_port: 8081,
                upload_dir: "./uploads".to_string(),
                max_file_size: 104857600, // 100MB
                base_url: None,
            },
            auth: AuthConfig {
                mode: "protected".to_string(),
                admin_username: "admin".to_string(),
                admin_password: "changeme".to_string(),
                jwt_secret: "your-super-secret-jwt-key-change-this-in-production".to_string(),
                disabled_routes: vec![
                    "/".to_string(),
                    "/web".to_string(),
                    "/web/".to_string(),
                    "/web/*".to_string(),
                    "/favicon.ico".to_string(),
                    "/api/health".to_string(),
                    "/docs".to_string(),
                    "/api-docs".to_string(),
                    "/api/auth/login".to_string(),
                    "/api/auth/refresh".to_string(),
                ],
            },
            image: ImageConfig {
                thumbnail_size: 200,
                jpeg_quality: 85,
                webp_quality: 80.0,
                qoi_enabled: true,
            },
            cors: CorsConfig {
                allowed_origins: vec![
                    "http://localhost:3000".to_string(),
                    "http://127.0.0.1:3000".to_string(),
                ],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "DELETE".to_string(),
                ],
                allowed_headers: vec![
                    "Content-Type".to_string(),
                    "Authorization".to_string(),
                ],
            },
            rate_limit: RateLimitConfig {
                auth_routes: RateLimitRule {
                    enabled: true,
                    requests_per_minute: 10,
                    burst_size: 3,
                },
                upload_routes: RateLimitRule {
                    enabled: true,
                    requests_per_minute: 60,
                    burst_size: 10,
                },
                static_routes: RateLimitRule {
                    enabled: true,
                    requests_per_minute: 1000,
                    burst_size: 100,
                },
                disabled_routes: vec![
                    "/health".to_string(),
                    "/docs".to_string(),
                    "/api-docs".to_string(),
                ],
            },
        }
    }
}

impl AppConfig {
    /// Get the base URL for static file serving
    pub fn get_static_base_url(&self) -> String {
        self.server.base_url
            .clone()
            .unwrap_or_else(|| format!("http://localhost:{}", self.server.static_port))
    }

    pub fn load() -> Result<Self> {
        // Load .env file if present
        dotenv::dotenv().ok();
        
        let mut config = Self::default();
        
        // Server configuration
        if let Ok(port) = env::var("WEB_PORT") {
            config.server.web_port = port.parse()
                .context("Invalid WEB_PORT environment variable")?;
        }
        
        if let Ok(port) = env::var("STATIC_PORT") {
            config.server.static_port = port.parse()
                .context("Invalid STATIC_PORT environment variable")?;
        }
        
        if let Ok(dir) = env::var("UPLOAD_DIR") {
            config.server.upload_dir = dir;
        }
        
        if let Ok(size) = env::var("MAX_FILE_SIZE") {
            config.server.max_file_size = size.parse()
                .context("Invalid MAX_FILE_SIZE environment variable")?;
        }
        
        if let Ok(base_url) = env::var("BASE_URL") {
            config.server.base_url = Some(base_url);
        }
        
        // Auth configuration
        if let Ok(mode) = env::var("AUTH_MODE") {
            config.auth.mode = mode;
        }
        
        if let Ok(username) = env::var("ADMIN_USERNAME") {
            config.auth.admin_username = username;
        }
        
        if let Ok(password) = env::var("ADMIN_PASSWORD") {
            config.auth.admin_password = password;
        }
        
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }
        
        if let Ok(disabled_routes) = env::var("AUTH_DISABLED_ROUTES") {
            config.auth.disabled_routes = disabled_routes.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
        
        // Image configuration
        if let Ok(size) = env::var("THUMBNAIL_SIZE") {
            config.image.thumbnail_size = size.parse()
                .context("Invalid THUMBNAIL_SIZE environment variable")?;
        }
        
        if let Ok(quality) = env::var("JPEG_QUALITY") {
            config.image.jpeg_quality = quality.parse()
                .context("Invalid JPEG_QUALITY environment variable")?;
        }
        
        if let Ok(quality) = env::var("WEBP_QUALITY") {
            config.image.webp_quality = quality.parse()
                .context("Invalid WEBP_QUALITY environment variable")?;
        }
        
        if let Ok(qoi_enabled) = env::var("QOI_ENABLED") {
            config.image.qoi_enabled = qoi_enabled.parse()
                .context("Invalid QOI_ENABLED environment variable")?;
        }
        
        // CORS configuration
        if let Ok(origins) = env::var("ALLOWED_ORIGINS") {
            config.cors.allowed_origins = origins.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
        
        if let Ok(methods) = env::var("ALLOWED_METHODS") {
            config.cors.allowed_methods = methods.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
        
        if let Ok(headers) = env::var("ALLOWED_HEADERS") {
            config.cors.allowed_headers = headers.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
        
        // Rate limit configuration
        if let Ok(disabled_routes) = env::var("RATE_LIMIT_DISABLED_ROUTES") {
            config.rate_limit.disabled_routes = disabled_routes.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
        
        // Auth routes rate limiting
        if let Ok(enabled) = env::var("RATE_LIMIT_AUTH_ENABLED") {
            config.rate_limit.auth_routes.enabled = enabled.parse()
                .context("Invalid RATE_LIMIT_AUTH_ENABLED environment variable")?;
        }
        
        if let Ok(rpm) = env::var("RATE_LIMIT_AUTH_RPM") {
            config.rate_limit.auth_routes.requests_per_minute = rpm.parse()
                .context("Invalid RATE_LIMIT_AUTH_RPM environment variable")?;
        }
        
        if let Ok(burst) = env::var("RATE_LIMIT_AUTH_BURST") {
            config.rate_limit.auth_routes.burst_size = burst.parse()
                .context("Invalid RATE_LIMIT_AUTH_BURST environment variable")?;
        }
        
        // Upload routes rate limiting
        if let Ok(enabled) = env::var("RATE_LIMIT_UPLOAD_ENABLED") {
            config.rate_limit.upload_routes.enabled = enabled.parse()
                .context("Invalid RATE_LIMIT_UPLOAD_ENABLED environment variable")?;
        }
        
        if let Ok(rpm) = env::var("RATE_LIMIT_UPLOAD_RPM") {
            config.rate_limit.upload_routes.requests_per_minute = rpm.parse()
                .context("Invalid RATE_LIMIT_UPLOAD_RPM environment variable")?;
        }
        
        if let Ok(burst) = env::var("RATE_LIMIT_UPLOAD_BURST") {
            config.rate_limit.upload_routes.burst_size = burst.parse()
                .context("Invalid RATE_LIMIT_UPLOAD_BURST environment variable")?;
        }
        
        // Static routes rate limiting
        if let Ok(enabled) = env::var("RATE_LIMIT_STATIC_ENABLED") {
            config.rate_limit.static_routes.enabled = enabled.parse()
                .context("Invalid RATE_LIMIT_STATIC_ENABLED environment variable")?;
        }
        
        if let Ok(rpm) = env::var("RATE_LIMIT_STATIC_RPM") {
            config.rate_limit.static_routes.requests_per_minute = rpm.parse()
                .context("Invalid RATE_LIMIT_STATIC_RPM environment variable")?;
        }
        
        if let Ok(burst) = env::var("RATE_LIMIT_STATIC_BURST") {
            config.rate_limit.static_routes.burst_size = burst.parse()
                .context("Invalid RATE_LIMIT_STATIC_BURST environment variable")?;
        }

        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.auth.mode != "protected" && self.auth.mode != "local" {
            anyhow::bail!("Auth mode must be either 'protected' or 'local'");
        }
        
        if self.auth.mode == "protected" && self.auth.admin_password == "changeme" {
            anyhow::bail!("Admin password must be changed for protected mode");
        }
        
        if self.auth.jwt_secret == "your-super-secret-jwt-key-change-this-in-production" && self.auth.mode == "protected" {
            anyhow::bail!("JWT secret must be changed for protected mode");
        }
        
        if self.server.max_file_size == 0 {
            anyhow::bail!("Max file size must be greater than 0");
        }
        
        if self.image.thumbnail_size == 0 {
            anyhow::bail!("Thumbnail size must be greater than 0");
        }
        
        Ok(())
    }
}
