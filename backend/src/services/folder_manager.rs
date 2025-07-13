use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;
use crate::models::{FolderInfo, FolderListResponse};
use tracing::{info};

/// Folder metadata stored in JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMetadata {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// File metadata with folder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub filename: String,
    pub folder_id: Option<String>,
    pub uploaded_at: DateTime<Utc>,
    #[serde(default)]
    pub size: u64,
}

pub struct FolderManager {
    upload_dir: PathBuf,
    metadata_file: PathBuf,
    file_metadata_file: PathBuf,
}

impl FolderManager {
    pub fn new(upload_dir: impl Into<PathBuf>) -> Self {
        let upload_dir: PathBuf = upload_dir.into();
        let metadata_file = upload_dir.join(".folder_metadata.json");
        let file_metadata_file = upload_dir.join(".file_metadata.json");
        
        Self {
            upload_dir,
            metadata_file,
            file_metadata_file,
        }
    }

    /// Load folder metadata from file
    pub fn load_folder_metadata(&self) -> Result<HashMap<String, FolderMetadata>, AppError> {
        if !self.metadata_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.metadata_file)?;
        let metadata: HashMap<String, FolderMetadata> = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse folder metadata: {}", e)))?;
        
        Ok(metadata)
    }

    /// Save folder metadata to file
    fn save_folder_metadata(&self, metadata: &HashMap<String, FolderMetadata>) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(metadata)
            .map_err(|e| AppError::Internal(format!("Failed to serialize folder metadata: {}", e)))?;
        
        fs::write(&self.metadata_file, content)?;
        Ok(())
    }

    /// Load file metadata from file
    pub fn load_file_metadata(&self) -> Result<HashMap<String, FileMetadata>, AppError> {
        if !self.file_metadata_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.file_metadata_file)?;
        let metadata: HashMap<String, FileMetadata> = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(format!("Failed to parse file metadata: {}", e)))?;
        
        Ok(metadata)
    }

    /// Save file metadata to file
    fn save_file_metadata(&self, metadata: &HashMap<String, FileMetadata>) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(metadata)
            .map_err(|e| AppError::Internal(format!("Failed to serialize file metadata: {}", e)))?;
        
        fs::write(&self.file_metadata_file, content)?;
        Ok(())
    }

    /// Create a new folder
    pub async fn create_folder(&self, name: &str, parent_id: Option<String>) -> Result<FolderInfo, AppError> {
        let folder_manager = self.clone();
        let name = name.to_string();
        
        tokio::task::spawn_blocking(move || {
            let mut metadata = folder_manager.load_folder_metadata()?;
            
            // Validate parent folder exists if specified
            if let Some(ref parent_id) = parent_id {
                if !metadata.contains_key(parent_id) {
                    return Err(AppError::NotFound(format!("Parent folder with id '{}' not found", parent_id)));
                }
            }
            
            // Check if folder with same name already exists in the parent
            for folder in metadata.values() {
                if folder.name == name && folder.parent_id == parent_id {
                    return Err(AppError::BadRequest(format!("Folder '{}' already exists in this location", name)));
                }
            }
            
            let folder_id = Uuid::new_v4().to_string();
            let created_at = Utc::now();
            
            let folder_metadata = FolderMetadata {
                id: folder_id.clone(),
                name: name.clone(),
                parent_id: parent_id.clone(),
                created_at,
            };
            
            metadata.insert(folder_id.clone(), folder_metadata);
            folder_manager.save_folder_metadata(&metadata)?;
            
            info!("Created folder: {} (id: {})", name, folder_id);
            
            Ok(FolderInfo {
                id: folder_id,
                name,
                parent_id,
                created_at,
                file_count: 0,
                folder_count: 0,
                size: 0,
            })
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute folder creation task".to_string()))?
    }

    /// Delete a folder (must be empty)
    pub async fn delete_folder(&self, folder_id: &str) -> Result<(), AppError> {
        let folder_manager = self.clone();
        let folder_id = folder_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let mut folder_metadata = folder_manager.load_folder_metadata()?;
            let file_metadata = folder_manager.load_file_metadata()?;
            
            // Check if folder exists
            if !folder_metadata.contains_key(&folder_id) {
                return Err(AppError::NotFound(format!("Folder with id '{}' not found", folder_id)));
            }
            
            // Check if folder has any files
            let has_files = file_metadata.values().any(|file| file.folder_id.as_ref() == Some(&folder_id));
            if has_files {
                return Err(AppError::BadRequest("Cannot delete folder: folder contains files".to_string()));
            }
            
            // Check if folder has any subfolders
            let has_subfolders = folder_metadata.values().any(|folder| folder.parent_id.as_ref() == Some(&folder_id));
            if has_subfolders {
                return Err(AppError::BadRequest("Cannot delete folder: folder contains subfolders".to_string()));
            }
            
            // Remove folder
            folder_metadata.remove(&folder_id);
            folder_manager.save_folder_metadata(&folder_metadata)?;
            
            info!("Deleted folder: {}", folder_id);
            Ok(())
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute folder deletion task".to_string()))?
    }

    /// Get folder contents
    pub async fn list_folder_contents(&self, folder_id: Option<String>) -> Result<FolderListResponse, AppError> {
        let folder_manager = self.clone();
        
        tokio::task::spawn_blocking(move || {
            let folder_metadata = folder_manager.load_folder_metadata()?;
            let file_metadata = folder_manager.load_file_metadata()?;
            
            // Helper function to calculate counts and size for a folder
            let calculate_folder_stats = |target_folder_id: &Option<String>| -> (usize, usize, u64) {
                let file_count = file_metadata.values()
                    .filter(|file| &file.folder_id == target_folder_id)
                    .count();
                
                let folder_count = folder_metadata.values()
                    .filter(|folder| &folder.parent_id == target_folder_id)
                    .count();
                
                let size = file_metadata.values()
                    .filter(|file| &file.folder_id == target_folder_id)
                    .map(|file| file.size)
                    .sum();
                
                (file_count, folder_count, size)
            };
            
            // Validate folder exists if specified and calculate its stats
            let current_folder = if let Some(ref folder_id) = folder_id {
                match folder_metadata.get(folder_id) {
                    Some(metadata) => {
                        let (file_count, folder_count, size) = calculate_folder_stats(&Some(folder_id.clone()));
                        Some(FolderInfo {
                            id: metadata.id.clone(),
                            name: metadata.name.clone(),
                            parent_id: metadata.parent_id.clone(),
                            created_at: metadata.created_at,
                            file_count,
                            folder_count,
                            size,
                        })
                    },
                    None => return Err(AppError::NotFound(format!("Folder with id '{}' not found", folder_id))),
                }
            } else {
                None
            };
            
            // Get subfolders with calculated stats
            let mut folders: Vec<FolderInfo> = folder_metadata.values()
                .filter(|folder| folder.parent_id == folder_id)
                .map(|metadata| {
                    let (file_count, folder_count, size) = calculate_folder_stats(&Some(metadata.id.clone()));
                    FolderInfo {
                        id: metadata.id.clone(),
                        name: metadata.name.clone(),
                        parent_id: metadata.parent_id.clone(),
                        created_at: metadata.created_at,
                        file_count,
                        folder_count,
                        size,
                    }
                })
                .collect();
            
            // Sort folders by name
            folders.sort_by(|a, b| a.name.cmp(&b.name));
            
            // Build breadcrumbs with calculated stats
            let mut breadcrumbs = Vec::new();
            let mut current_id = folder_id.clone();
            
            while let Some(id) = current_id {
                if let Some(metadata) = folder_metadata.get(&id) {
                    let (file_count, folder_count, size) = calculate_folder_stats(&Some(id.clone()));
                    breadcrumbs.insert(0, FolderInfo {
                        id: metadata.id.clone(),
                        name: metadata.name.clone(),
                        parent_id: metadata.parent_id.clone(),
                        created_at: metadata.created_at,
                        file_count,
                        folder_count,
                        size,
                    });
                    current_id = metadata.parent_id.clone();
                } else {
                    break;
                }
            }
            
            Ok(FolderListResponse {
                folders,
                current_folder,
                breadcrumbs,
            })
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute folder listing task".to_string()))?
    }

    /// Assign a file to a folder
    pub async fn assign_file_to_folder(&self, filename: &str, folder_id: Option<String>, size: u64) -> Result<(), AppError> {
        let folder_manager = self.clone();
        let filename = filename.to_string();
        
        tokio::task::spawn_blocking(move || {
            let folder_metadata = folder_manager.load_folder_metadata()?;
            let mut file_metadata = folder_manager.load_file_metadata()?;
            
            // Validate folder exists if specified
            if let Some(ref folder_id) = folder_id {
                if !folder_metadata.contains_key(folder_id) {
                    return Err(AppError::NotFound(format!("Folder with id '{}' not found", folder_id)));
                }
            }
            
            // Update or create file metadata
            let file_meta = FileMetadata {
                filename: filename.clone(),
                folder_id: folder_id.clone(),
                uploaded_at: Utc::now(),
                size,
            };
            
            file_metadata.insert(filename.clone(), file_meta);
            folder_manager.save_file_metadata(&file_metadata)?;
            
            Ok(())
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute file assignment task".to_string()))?
    }

    /// Get folder ID for a file
    pub async fn get_file_folder(&self, filename: &str) -> Result<Option<String>, AppError> {
        let folder_manager = self.clone();
        let filename = filename.to_string();
        
        tokio::task::spawn_blocking(move || {
            let file_metadata = folder_manager.load_file_metadata()?;
            Ok(file_metadata.get(&filename).and_then(|meta| meta.folder_id.clone()))
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute file folder lookup task".to_string()))?
    }

    /// Get list of files in a specific folder
    pub fn get_files_in_folder(&self, folder_id: Option<String>) -> Result<Vec<String>, AppError> {
        let file_metadata = self.load_file_metadata()?;
        let files: Vec<String> = match folder_id {
            Some(ref fid) => file_metadata
                .values()
                .filter(|file| file.folder_id.as_ref() == Some(fid))
                .map(|file| file.filename.clone())
                .collect(),
            None => file_metadata
                .values()
                .map(|file| file.filename.clone())
                .collect(),
        };
        Ok(files)
    }

    /// Remove file from metadata when deleted
    pub async fn remove_file_metadata(&self, filename: &str) -> Result<(), AppError> {
        let folder_manager = self.clone();
        let filename = filename.to_string();
        
        tokio::task::spawn_blocking(move || {
            let mut file_metadata = folder_manager.load_file_metadata()?;
            file_metadata.remove(&filename);
            folder_manager.save_file_metadata(&file_metadata)?;
            Ok(())
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute remove file metadata task".to_string()))?
    }

    /// Move a folder to a new parent folder
    pub async fn move_folder(&self, folder_id: &str, new_parent_id: Option<String>) -> Result<(), AppError> {
        let folder_manager = self.clone();
        let folder_id = folder_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let mut folder_metadata = folder_manager.load_folder_metadata()?;
            
            // Check if the folder exists
            let folder = folder_metadata.get(&folder_id)
                .ok_or_else(|| AppError::NotFound(format!("Folder with id '{}' not found", folder_id)))?
                .clone();
            
            // Validate new parent folder exists if specified
            if let Some(ref parent_id) = new_parent_id {
                if !folder_metadata.contains_key(parent_id) {
                    return Err(AppError::NotFound(format!("Target parent folder with id '{}' not found", parent_id)));
                }
                
                // Check for circular reference - ensure we're not moving a folder into one of its descendants
                let mut current_parent = new_parent_id.clone();
                while let Some(parent_id) = current_parent {
                    if parent_id == folder_id {
                        return Err(AppError::BadRequest("Cannot move folder into one of its descendants".to_string()));
                    }
                    current_parent = folder_metadata.get(&parent_id).and_then(|f| f.parent_id.clone());
                }
            }
            
            // Check if a folder with the same name already exists in the target location
            for existing_folder in folder_metadata.values() {
                if existing_folder.name == folder.name 
                    && existing_folder.parent_id == new_parent_id 
                    && existing_folder.id != folder_id {
                    return Err(AppError::BadRequest(format!("Folder '{}' already exists in target location", folder.name)));
                }
            }
            
            // Update the folder's parent_id
            if let Some(folder_meta) = folder_metadata.get_mut(&folder_id) {
                folder_meta.parent_id = new_parent_id.clone();
            }
            
            folder_manager.save_folder_metadata(&folder_metadata)?;
            
            info!("Moved folder '{}' (id: {}) to new parent: {:?}", folder.name, folder_id, new_parent_id);
            Ok(())
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute move folder task".to_string()))?
    }

    /// Get folder info by ID
    pub async fn get_folder_info(&self, folder_id: &str) -> Result<FolderInfo, AppError> {
        let folder_manager = self.clone();
        let folder_id = folder_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let folder_metadata = folder_manager.load_folder_metadata()?;
            let file_metadata = folder_manager.load_file_metadata()?;
            
            match folder_metadata.get(&folder_id) {
                Some(metadata) => {
                    let file_count = file_metadata.values()
                        .filter(|file| file.folder_id.as_ref() == Some(&folder_id))
                        .count();
                    
                    let folder_count = folder_metadata.values()
                        .filter(|folder| folder.parent_id.as_ref() == Some(&folder_id))
                        .count();
                    
                    let size = file_metadata.values()
                        .filter(|file| file.folder_id.as_ref() == Some(&folder_id))
                        .map(|file| file.size)
                        .sum();
                    
                    Ok(FolderInfo {
                        id: metadata.id.clone(),
                        name: metadata.name.clone(),
                        parent_id: metadata.parent_id.clone(),
                        created_at: metadata.created_at,
                        file_count,
                        folder_count,
                        size,
                    })
                },
                None => Err(AppError::NotFound(format!("Folder with id '{}' not found", folder_id))),
            }
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute get folder info task".to_string()))?
    }

}

impl Clone for FolderManager {
    fn clone(&self) -> Self {
        Self {
            upload_dir: self.upload_dir.clone(),
            metadata_file: self.metadata_file.clone(),
            file_metadata_file: self.file_metadata_file.clone(),
        }
    }
}
