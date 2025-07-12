use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorTooManyRequests,
    Error,
};
use futures::future::{Ready, ready, LocalBoxFuture};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
};
use std::num::NonZeroU32;
use crate::config::RateLimitConfig;

pub struct RateLimitMiddleware {
    limiters: Arc<HashMap<String, Arc<DefaultKeyedRateLimiter<IpAddr>>>>,
    disabled_routes: Vec<String>,
}

impl RateLimitMiddleware {
    pub fn new(config: &RateLimitConfig) -> Self {
        let mut limiters = HashMap::new();

        // Create rate limiter for auth routes (strict)
        if config.auth_routes.enabled {
            let requests_per_minute = NonZeroU32::new(config.auth_routes.requests_per_minute)
                .unwrap_or(NonZeroU32::new(10).unwrap());
            let burst_size = NonZeroU32::new(config.auth_routes.burst_size)
                .unwrap_or(NonZeroU32::new(3).unwrap());
            let quota = Quota::per_minute(requests_per_minute)
                .allow_burst(burst_size);
            let limiter = Arc::new(RateLimiter::keyed(quota));
            limiters.insert("auth".to_string(), limiter);
        }

        // Create rate limiter for upload routes (moderate)
        if config.upload_routes.enabled {
            let requests_per_minute = NonZeroU32::new(config.upload_routes.requests_per_minute)
                .unwrap_or(NonZeroU32::new(60).unwrap());
            let burst_size = NonZeroU32::new(config.upload_routes.burst_size)
                .unwrap_or(NonZeroU32::new(10).unwrap());
            let quota = Quota::per_minute(requests_per_minute)
                .allow_burst(burst_size);
            let limiter = Arc::new(RateLimiter::keyed(quota));
            limiters.insert("upload".to_string(), limiter);
        }

        // Create rate limiter for static routes (lenient)
        if config.static_routes.enabled {
            let requests_per_minute = NonZeroU32::new(config.static_routes.requests_per_minute)
                .unwrap_or(NonZeroU32::new(1000).unwrap());
            let burst_size = NonZeroU32::new(config.static_routes.burst_size)
                .unwrap_or(NonZeroU32::new(100).unwrap());
            let quota = Quota::per_minute(requests_per_minute)
                .allow_burst(burst_size);
            let limiter = Arc::new(RateLimiter::keyed(quota));
            limiters.insert("static".to_string(), limiter);
        }

        Self {
            limiters: Arc::new(limiters),
            disabled_routes: config.disabled_routes.clone(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitService {
            service,
            limiters: self.limiters.clone(),
            disabled_routes: self.disabled_routes.clone(),
        }))
    }
}

pub struct RateLimitService<S> {
    service: S,
    limiters: Arc<HashMap<String, Arc<DefaultKeyedRateLimiter<IpAddr>>>>,
    disabled_routes: Vec<String>,
}

impl<S> RateLimitService<S> {
    fn determine_route_type(&self, path: &str) -> Option<&str> {
        // Check if route is disabled
        for disabled_route in &self.disabled_routes {
            if path.starts_with(disabled_route) {
                return None; // No rate limiting
            }
        }

        // Determine route type based on path
        if path.starts_with("/uploads") {
            Some("static")
        } else if path.starts_with("/upload") {
            Some("upload")
        } else if path.contains("login") || path.contains("auth") {
            Some("auth")
        } else {
            Some("upload") // Default to upload rate limits for other routes
        }
    }

    fn get_client_ip(&self, req: &ServiceRequest) -> IpAddr {
        // Try to get IP from X-Forwarded-For header first (for reverse proxies)
        if let Some(forwarded_for) = req.headers().get("x-forwarded-for") {
            if let Ok(forwarded_str) = forwarded_for.to_str() {
                if let Some(first_ip) = forwarded_str.split(',').next() {
                    if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                        return ip;
                    }
                }
            }
        }

        // Try X-Real-IP header
        if let Some(real_ip) = req.headers().get("x-real-ip") {
            if let Ok(ip_str) = real_ip.to_str() {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    return ip;
                }
            }
        }

        // Fall back to connection info
        req.connection_info()
            .peer_addr()
            .and_then(|addr| addr.split(':').next())
            .and_then(|ip_str| ip_str.parse().ok())
            .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))
    }
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        
        // Determine if rate limiting should be applied
        let route_type = self.determine_route_type(&path);
        
        if let Some(route_type) = route_type {
            if let Some(limiter) = self.limiters.get(route_type) {
                let client_ip = self.get_client_ip(&req);
                
                // Check rate limit
                match limiter.check_key(&client_ip) {
                    Ok(_) => {
                        // Rate limit passed, continue to service
                        let fut = self.service.call(req);
                        Box::pin(async move { fut.await })
                    }
                    Err(_) => {
                        // Rate limit exceeded
                        Box::pin(async move {
                            Err(ErrorTooManyRequests("Rate limit exceeded"))
                        })
                    }
                }
            } else {
                // No rate limiter configured for this route type
                let fut = self.service.call(req);
                Box::pin(async move { fut.await })
            }
        } else {
            // Route is disabled from rate limiting
            let fut = self.service.call(req);
            Box::pin(async move { fut.await })
        }
    }
}
