use actix_web::get;
use actix_web::{web, HttpResponse};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::services::file_utils::FileManager;
use crate::services::folder_manager::FolderManager;
use crate::handlers::files::ExportQuery;

#[utoipa::path(
    get,
    path = "/api/files/export",
    params(ExportQuery),
    responses(
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Files"
)]
#[get("/files/export")]
pub async fn export_files(
    query: web::Query<ExportQuery>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, AppError> {
    use tracing::info;
    use zip::write::FileOptions;
    use std::io::Cursor;

    let file_manager = FileManager::new(
        &config.server.upload_dir,
        config.get_static_base_url(),
    );
    let folder_manager = FolderManager::new(&config.server.upload_dir);


    // Load all file and folder metadata
    let file_metadata = folder_manager.load_file_metadata()?;
    let folder_metadata = folder_manager.load_folder_metadata()?;

    // Helper to build relative path for a file by walking up the folder tree
    fn build_relative_path(file: &crate::services::folder_manager::FileMetadata, folder_metadata: &std::collections::HashMap<String, crate::services::folder_manager::FolderMetadata>) -> String {
        let mut components = vec![file.filename.clone()];
        let mut current_folder = file.folder_id.clone();
        while let Some(ref folder_id) = current_folder {
            if let Some(folder) = folder_metadata.get(folder_id) {
                if folder.name != "root" { // skip adding root to path
                    components.push(folder.name.clone());
                }
                current_folder = folder.parent_id.clone();
            } else {
                break;
            }
        }
        components.reverse();
        components.join("/")
    }

    // Select files to export
    let files_to_export: Vec<&crate::services::folder_manager::FileMetadata> = if let Some(ref folder_id) = query.folder_id {
        file_metadata.values().filter(|file| file.folder_id.as_ref() == Some(folder_id)).collect()
    } else {
        file_metadata.values().collect()
    };

    if files_to_export.is_empty() {
        return Err(AppError::BadRequest("No files to export".to_string()));
    }

    // Create ZIP archive in memory, preserving folder structure and including empty folders
    let mut zip_data = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut zip_data));
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // 1. Add empty folders
        use std::collections::HashSet;
        // Build a set of all folder paths that will be needed
        let mut folders_with_files = HashSet::new();
        for file in &files_to_export {
            let mut current_folder = file.folder_id.clone();
            while let Some(ref folder_id) = current_folder {
                if let Some(folder) = folder_metadata.get(folder_id) {
                    folders_with_files.insert(folder_id.clone());
                    current_folder = folder.parent_id.clone();
                } else {
                    break;
                }
            }
        }
        // Find all folders that are not root
        let all_folder_ids: Vec<_> = folder_metadata.iter().filter(|(_, f)| f.name != "root").map(|(id, _)| id.clone()).collect();
        // For each folder, check if it contains any files
        for folder_id in all_folder_ids {
            let has_file = files_to_export.iter().any(|file| file.folder_id.as_ref() == Some(&folder_id));
            if !has_file {
                // Build the relative path for the folder
                let mut components = vec![];
                let mut current_folder = Some(folder_id.clone());
                while let Some(ref fid) = current_folder {
                    if let Some(folder) = folder_metadata.get(fid) {
                        if folder.name != "root" {
                            components.push(folder.name.clone());
                        }
                        current_folder = folder.parent_id.clone();
                    } else {
                        break;
                    }
                }
                components.reverse();
                if !components.is_empty() {
                    let folder_path = format!("{}/", components.join("/"));
                    let _ = zip.add_directory(folder_path, options);
                }
            }
        }

        // 2. Add files
        for file in &files_to_export {
            let rel_path = build_relative_path(file, &folder_metadata);
            let file_path = file_manager.get_file_path(&file.filename);
            if let Ok(mut f) = std::fs::File::open(&file_path) {
                let _ = zip.start_file(&rel_path, options);
                let _ = std::io::copy(&mut f, &mut zip);
            }
        }
        let _ = zip.finish();
    }

    // Generate filename for the ZIP
    let zip_filename = if let Some(ref folder_id) = query.folder_id {
        let folder_info = folder_manager.get_folder_info(folder_id).await?;
        format!("{}_export.zip", folder_info.name)
    } else {
        "export.zip".to_string()
    };

    info!("Exported {} files to ZIP: {} files", files_to_export.len(), zip_filename);

    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", zip_filename)))
        .body(zip_data))
}
