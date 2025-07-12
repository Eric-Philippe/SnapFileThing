use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use crate::error::AppError;
use crate::models::{FileInfo, FileUrls};
use crate::services::image_processor::ImageProcessor;
use crate::utils::mime_type::get_mime_type;
use tracing::{debug, info};

pub struct FileManager {
    upload_dir: PathBuf,
    static_base_url: String,
}

impl FileManager {
    pub fn new(upload_dir: impl Into<PathBuf>, static_base_url: String) -> Self {
        Self {
            upload_dir: upload_dir.into(),
            static_base_url,
        }
    }

    /// Generate a unique filename to avoid conflicts
    pub fn generate_unique_filename(&self, original_filename: &str) -> String {
        let sanitized = sanitize_filename::sanitize(original_filename);
        let path = Path::new(&sanitized);
        
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let timestamp = chrono::Utc::now().timestamp();
        let uuid = uuid::Uuid::new_v4().to_string()[..8].to_string();
        
        if extension.is_empty() {
            format!("{}_{}_{}_{}", stem, timestamp, uuid, "bin")
        } else {
            format!("{}_{}_{}_.{}", stem, timestamp, uuid, extension)
        }
    }

    /// Get the full path for a filename in the upload directory
    pub fn get_file_path(&self, filename: &str) -> PathBuf {
        self.upload_dir.join(filename)
    }

    /// Generate URLs for a file
    #[allow(dead_code)]
    pub fn generate_urls(&self, filename: &str, is_image: bool) -> FileUrls {
        let base_url = format!("{}/uploads", self.static_base_url);
        let original = format!("{}/{}", base_url, filename);
        
        if !is_image {
            return FileUrls {
                original,
                qoi: None,
                thumbnail: None,
            };
        }

        // Generate QOI and thumbnail filenames
        let path = Path::new(filename);
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        
        let qoi_filename = format!("{}.qoi", stem);
        let thumb_filename = format!("{}_thumb.webp", stem);
        
        FileUrls {
            original,
            qoi: Some(format!("{}/{}", base_url, qoi_filename)),
            thumbnail: Some(format!("{}/{}", base_url, thumb_filename)),
        }
    }

    /// List files with optional filter by filename list
    pub async fn list_files_with_filter(
        &self,
        page: usize,
        per_page: usize,
        filter_files: Option<Vec<String>>,
    ) -> Result<(Vec<FileInfo>, usize), AppError> {
        let upload_dir = self.upload_dir.clone();
        let static_base_url = self.static_base_url.clone();
        
        tokio::task::spawn_blocking(move || -> Result<(Vec<FileInfo>, usize), AppError> {
            let mut files = Vec::new();
            
            if !upload_dir.exists() {
                return Ok((files, 0));
            }
            
            let entries = fs::read_dir(&upload_dir)?;
            let mut file_entries = Vec::new();
            
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    let filename = path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    // Skip thumbnail and QOI files in listing
                    if filename.contains("_thumb.") || filename.ends_with(".qoi") {
                        continue;
                    }
                    
                    // If filter is provided, only include files in the filter list
                    if let Some(ref filter) = filter_files {
                        if !filter.contains(&filename) {
                            continue;
                        }
                    }
                    
                    let metadata = entry.metadata()?;
                    let size = metadata.len();
                    let modified = metadata.modified()?;
                    let uploaded_at: DateTime<Utc> = modified.into();
                    
                    let mime_type = get_mime_type(&filename);
                    let is_image = ImageProcessor::is_image_file(&filename);
                    
                    let urls = FileUrls {
                        original: format!("{}/uploads/{}", static_base_url, filename),
                        qoi: if is_image {
                            let stem = Path::new(&filename).file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("file");
                            let qoi_filename = format!("{}.qoi", stem);
                            let qoi_path = upload_dir.join(&qoi_filename);
                            if qoi_path.exists() {
                                Some(format!("{}/uploads/{}", static_base_url, qoi_filename))
                            } else {
                                None
                            }
                        } else {
                            None
                        },
                        thumbnail: if is_image {
                            let stem = Path::new(&filename).file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("file");
                            let thumb_filename = format!("{}_thumb.webp", stem);
                            let thumb_path = upload_dir.join(&thumb_filename);
                            if thumb_path.exists() {
                                Some(format!("{}/uploads/{}", static_base_url, thumb_filename))
                            } else {
                                None
                            }
                        } else {
                            None
                        },
                    };
                    
                    // Try to get image dimensions if it's an image
                    let dimensions = if is_image {
                        match image::ImageReader::open(&path).and_then(|r| r.into_dimensions().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))) {
                            Ok(dims) => Some(dims),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    
                    file_entries.push((uploaded_at, FileInfo {
                        filename,
                        size,
                        mime_type,
                        uploaded_at,
                        is_image,
                        urls,
                        dimensions,
                        folder_id: None, // Will be set by the caller
                    }));
                }
            }
            
            // Sort by upload date (newest first)
            file_entries.sort_by(|a, b| b.0.cmp(&a.0));
            
            let total = file_entries.len();
            let start = page * per_page;
            let end = std::cmp::min(start + per_page, total);
            
            if start < total {
                files = file_entries[start..end].iter().map(|(_, info)| info.clone()).collect();
            }
            
            Ok((files, total))
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute file listing task".to_string()))?
    }

    /// Delete a file and its associated files (QOI, thumbnail)
    pub async fn delete_file(&self, filename: &str) -> Result<(), AppError> {
        let upload_dir = self.upload_dir.clone();
        let filename = filename.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<(), AppError> {
            let file_path = upload_dir.join(&filename);
            
            if !file_path.exists() {
                return Err(AppError::FileNotFound(filename));
            }
            
            // Remove the main file
            fs::remove_file(&file_path)?;
            info!("Deleted file: {:?}", file_path);
            
            // Remove associated files if they exist
            let path = Path::new(&filename);
            let stem = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            
            // Remove QOI file
            let qoi_path = upload_dir.join(format!("{}.qoi", stem));
            if qoi_path.exists() {
                fs::remove_file(&qoi_path)?;
                debug!("Deleted QOI file: {:?}", qoi_path);
            }
            
            // Remove thumbnail
            let thumb_path = upload_dir.join(format!("{}_thumb.webp", stem));
            if thumb_path.exists() {
                fs::remove_file(&thumb_path)?;
                debug!("Deleted thumbnail: {:?}", thumb_path);
            }
            
            Ok(())
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute file deletion task".to_string()))?
    }

    /// Find a file by its stem (base filename)
    /// This allows deleting files by providing just the base name
    pub async fn find_file_by_stem(&self, stem: &str) -> Result<Option<String>, AppError> {
        let upload_dir = self.upload_dir.clone();
        let stem = stem.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<Option<String>, AppError> {
            if !upload_dir.exists() {
                return Ok(None);
            }
            
            let entries = fs::read_dir(&upload_dir)?;
            
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    let filename = path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("")
                        .to_string();
                    
                    // Skip thumbnail and QOI files - we want to find the original
                    if filename.contains("_thumb.") || filename.ends_with(".qoi") {
                        continue;
                    }
                    
                    // Extract the stem from the filename
                    let file_path = Path::new(&filename);
                    let file_stem = file_path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    
                    // Check if this file's stem matches what we're looking for
                    if file_stem == stem {
                        return Ok(Some(filename));
                    }
                    
                    // Also check if the provided stem is part of the filename
                    // This handles cases where user provides partial filename
                    if filename.starts_with(&stem) {
                        return Ok(Some(filename));
                    }
                }
            }
            
            Ok(None)
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute filename generation task".to_string()))?
    }

    /// Check if a file exists
    pub fn file_exists(&self, filename: &str) -> bool {
        self.get_file_path(filename).exists()
    }

    /// Get the size of a file in bytes
    pub fn get_file_size(&self, filename: &str) -> Result<u64, AppError> {
        let file_path = self.get_file_path(filename);
        if !file_path.exists() {
            return Err(AppError::FileNotFound(filename.to_string()));
        }
        
        let metadata = fs::metadata(&file_path)?;
        Ok(metadata.len())
    }
}
