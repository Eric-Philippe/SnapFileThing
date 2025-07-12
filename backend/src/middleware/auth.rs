use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpResponse, body::EitherBody, web,
};
use actix_web::dev::{Service, Transform};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use base64::Engine;
use constant_time_eq::constant_time_eq;
use tracing::warn;

use crate::handlers::auth::JwtService;
use crate::config::AuthConfig;

pub struct AuthMiddleware {
    auth_config: AuthConfig,
}

impl AuthMiddleware {
    pub fn new(auth_config: AuthConfig) -> Self {
        Self { auth_config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service,
            auth_config: self.auth_config.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    auth_config: AuthConfig,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();
        
        // Backward compatibility mapping
        let auth_mode = match self.auth_config.mode.as_str() {
            "public" => "disabled",  // Old "public" -> new "disabled"
            "local" => "protected",  // Old "local" -> new "protected"
            mode => mode,            // Use new naming directly
        };
        
        // If authentication mode is disabled, skip auth entirely
        if auth_mode == "disabled" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        // Check if the route is in the disabled routes list
        let is_auth_disabled = self.auth_config.disabled_routes.iter().any(|route| {
            if route == path {
                return true;
            }
            // Handle special patterns
            if route.ends_with("/*") {
                let prefix = &route[..route.len() - 2];
                return path.starts_with(prefix) || path == prefix;
            }
            false
        });

        // Also check for static file patterns that should always be accessible
        let is_static_file = path.ends_with(".ico") 
            || path.ends_with(".png") 
            || path.ends_with(".jpg") 
            || path.ends_with(".jpeg") 
            || path.ends_with(".gif") 
            || path.ends_with(".svg") 
            || path.ends_with(".webp") 
            || path.ends_with(".css") 
            || path.ends_with(".js") 
            || path.ends_with(".woff") 
            || path.ends_with(".woff2") 
            || path.ends_with(".ttf") 
            || path.ends_with(".eot") 
            || path.ends_with(".txt") 
            || path.ends_with(".json") 
            || path.ends_with(".webmanifest")
            || path.starts_with("/assets/")
            || path.starts_with("/web/assets/")
            || path.starts_with("/uploads/");

        if is_auth_disabled || is_static_file {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        let username = self.auth_config.admin_username.clone();
        let password = self.auth_config.admin_password.clone();

        let auth_header = req.headers().get("Authorization");
        
        if let Some(auth_value) = auth_header {
            if let Ok(auth_str) = auth_value.to_str() {
                // Try JWT Bearer token first
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    
                    // Get JWT service from app data
                    if let Some(jwt_service) = req.app_data::<web::Data<JwtService>>() {
                        match jwt_service.validate_token(token) {
                            Ok(token_data) => {
                                // Ensure it's an access token
                                if token_data.claims.token_type == "access" {
                                    let fut = self.service.call(req);
                                    return Box::pin(async move {
                                        let res = fut.await?;
                                        Ok(res.map_into_left_body())
                                    });
                                }
                            }
                            Err(_) => {
                                // JWT validation failed, continue to basic auth fallback
                            }
                        }
                    }
                }
                // Fallback to Basic Auth for backward compatibility
                else if auth_str.starts_with("Basic ") {
                    let encoded = &auth_str[6..];
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
                        if let Ok(credentials) = String::from_utf8(decoded) {
                            let parts: Vec<&str> = credentials.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                let provided_username = parts[0];
                                let provided_password = parts[1];
                                
                                // Use constant time comparison to prevent timing attacks
                                let username_valid = constant_time_eq(
                                    username.as_bytes(),
                                    provided_username.as_bytes()
                                );
                                let password_valid = constant_time_eq(
                                    password.as_bytes(),
                                    provided_password.as_bytes()
                                );
                                
                                if username_valid && password_valid {
                                    let fut = self.service.call(req);
                                    return Box::pin(async move {
                                        let res = fut.await?;
                                        Ok(res.map_into_left_body())
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        warn!("Unauthorized access attempt to: {}", path);
        
        Box::pin(async move {
            let response = HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "Authentication required",
                    "message": "Please provide valid credentials"
                }));
            Ok(req.into_response(response).map_into_right_body())
        })
    }
}
