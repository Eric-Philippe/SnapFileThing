use actix_web::{get, HttpResponse, Result, HttpRequest};
use actix_files::NamedFile;
use std::path::Path;

const FRONTEND_DIST_PATH: &str = "../frontend/dist";

#[get("/web")]
pub async fn serve_index() -> Result<HttpResponse> {
    let path = Path::new(FRONTEND_DIST_PATH).join("index.html");
    if path.exists() {
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(std::fs::read_to_string(path)?))
    } else {
        Ok(HttpResponse::NotFound().body("Frontend not built. Please run 'cd frontend && npm run build'."))
    }
}

// Serve static assets (CSS, JS, images, etc.)
#[get("/web/assets/{filename:.*}")]
pub async fn serve_assets(req: HttpRequest) -> Result<NamedFile> {
    let filename: String = req.match_info().query("filename").parse().unwrap();
    let path = Path::new(FRONTEND_DIST_PATH).join("assets").join(&filename);
    Ok(NamedFile::open(path)?)
}

// Serve other static files (favicon, etc.) and handle frontend routing
#[get("/web/{path:.*}")]
pub async fn serve_static_files(req: HttpRequest) -> Result<HttpResponse> {
    let path_param: String = req.match_info().query("path").parse().unwrap_or_default();
    
    // If it's an empty path, serve index.html
    if path_param.is_empty() {
        let path = Path::new(FRONTEND_DIST_PATH).join("index.html");
        if path.exists() {
            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(std::fs::read_to_string(path)?));
        } else {
            return Ok(HttpResponse::NotFound().body("Frontend not built. Please run 'cd frontend && npm run build'."));
        }
    }
    
    // Only serve specific static files to avoid conflicts with API routes
    if path_param.ends_with(".ico") || path_param.ends_with(".txt") || path_param.ends_with(".json") 
        || path_param.ends_with(".svg") || path_param.ends_with(".png") || path_param.ends_with(".webmanifest") {
        let path = Path::new(FRONTEND_DIST_PATH).join(&path_param);
        if path.exists() {
            let content = std::fs::read(&path)?;
            let content_type = match path_param.split('.').last() {
                Some("ico") => "image/x-icon",
                Some("svg") => "image/svg+xml",
                Some("png") => "image/png",
                Some("json" | "webmanifest") => "application/json",
                Some("txt") => "text/plain",
                _ => "application/octet-stream"
            };
            return Ok(HttpResponse::Ok()
                .content_type(content_type)
                .body(content));
        }
    }
    
    // For all other routes (like /web/files, /web/upload, etc.), serve the index.html (for React routing)
    let path = Path::new(FRONTEND_DIST_PATH).join("index.html");
    if path.exists() {
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(std::fs::read_to_string(path)?))
    } else {
        Ok(HttpResponse::NotFound().body("Frontend not built. Please run 'cd frontend && npm run build'."))
    }
}
