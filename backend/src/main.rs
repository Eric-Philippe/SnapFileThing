use actix_web::{web, App, HttpServer, middleware::Logger, http::Method};
use actix_files::Files;
use actix_cors::Cors;
use std::path::Path;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;
mod error;
mod docs;

use config::AppConfig;
use middleware::auth::AuthMiddleware;
use middleware::rate_limit::RateLimitMiddleware;
use handlers::auth::JwtService;
use docs::ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "snapfilething=info,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load().expect("Failed to load configuration");
    
    // Ensure upload directory exists
    if !Path::new(&config.server.upload_dir).exists() {
        std::fs::create_dir_all(&config.server.upload_dir)
            .expect("Failed to create upload directory");
        info!("Created upload directory: {}", config.server.upload_dir);
    }

    info!("Starting SnapFileThing server");
    info!("Web interface is available on http://localhost:{}/web/", config.server.web_port);
    info!("API documentation is available at http://localhost:{}/docs", config.server.web_port);
    info!("Static files will be served on http://localhost:{}", config.server.static_port);
    info!("Authentication mode: {}", config.auth.mode);

    let config_clone = config.clone();
    let config_clone2 = config.clone();
    let upload_dir = config.server.upload_dir.clone();
    let static_port = config.server.static_port;

    // Create JWT service
    let jwt_service = web::Data::new(JwtService::new(&config.auth.jwt_secret));

    // Start static file server (port 2)
    let static_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(RateLimitMiddleware::new(&config_clone.rate_limit))
            .service(
                Files::new("/uploads", &upload_dir)
                    .use_etag(true)
                    .use_last_modified(true)
                    .prefer_utf8(true)
            )
    })
    .bind(format!("0.0.0.0:{}", static_port))?
    .run();

    // Start main web server (port 1)
    let web_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn({
                let allowed_origins = config_clone2.cors.allowed_origins.clone();
                move |origin, _req_head| {
                    allowed_origins
                        .iter()
                        .any(|allowed| allowed == origin)
                }
            })
            .allowed_methods(
                config_clone2.cors.allowed_methods
                    .iter()
                    .filter_map(|method| method.parse::<Method>().ok())
                    .collect::<Vec<_>>()
            )
            .allowed_headers(config_clone2.cors.allowed_headers.clone())
            .max_age(3600);

        let app = App::new()
            .app_data(web::Data::new(config_clone2.clone()))
            .app_data(jwt_service.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(RateLimitMiddleware::new(&config_clone2.rate_limit))
            .wrap(AuthMiddleware::new(config_clone2.auth.clone()))
            .service(
                web::scope("/api")
                    .service(handlers::health::health_check)
                    .service(
                        web::scope("/auth")
                            .route("/login", web::post().to(handlers::auth::login))
                            .route("/logout", web::post().to(handlers::auth::logout))
                            .route("/refresh", web::post().to(handlers::auth::refresh_token))
                            .route("/verify", web::get().to(handlers::auth::verify_token))
                    )
                    .service(handlers::upload::upload_file)
                    .service(handlers::files::list_files)
                    .service(handlers::files::delete_file)
                    .service(handlers::files::move_file)
                    .service(handlers::files::export_files)
                    .service(handlers::folders::list_folders)
                    .service(handlers::folders::create_folder)
                    .service(handlers::folders::delete_folder)
                    .service(handlers::folders::move_folder)
            )
            .service(
                SwaggerUi::new("/docs/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .service(handlers::frontend::serve_assets)
            .service(handlers::frontend::serve_static_files)
            .service(handlers::frontend::serve_index);
        app
    })
    .bind(format!("0.0.0.0:{}", config.server.web_port))?
    .run();

    // Run both servers concurrently
    tokio::try_join!(static_server, web_server).map(|_| ())?;

    Ok(())
}
