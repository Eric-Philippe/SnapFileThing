use image::{ImageFormat, GenericImageView};
use std::path::Path;
use crate::error::AppError;
use crate::config::ImageConfig;
use tracing::{info, debug};

pub struct ImageProcessor {
    config: ImageConfig,
}

impl ImageProcessor {
    pub fn new(config: ImageConfig) -> Self {
        Self { config }
    }

    /// Check if a file is an image based on its extension
    pub fn is_image_file(filename: &str) -> bool {
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        matches!(
            extension.as_deref(),
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | 
            Some("bmp") | Some("tiff") | Some("tif") | Some("webp")
        )
    }

    /// Convert image to QOI format
    pub async fn convert_to_qoi(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(u32, u32), AppError> {
        let input_path = input_path.to_owned();
        let output_path = output_path.to_owned();

        tokio::task::spawn_blocking(move || -> Result<(u32, u32), AppError> {
            debug!("Converting image to QOI: {:?} -> {:?}", input_path, output_path);
            
            let img = image::open(&input_path)?;
            let (width, height) = img.dimensions();
            
            // Convert to RGBA8
            let rgba_img = img.to_rgba8();
            let raw_data: Vec<u8> = rgba_img.into_raw();
            
            // Create QOI image
            let qoi_data = qoi::encode_to_vec(&raw_data, width, height)
                .map_err(|e| AppError::QoiEncoding(e.to_string()))?;
            
            // Write QOI file
            std::fs::write(&output_path, qoi_data)?;
            
            info!("Successfully converted image to QOI: {:?}", output_path);
            Ok((width, height))
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute QOI conversion task".to_string()))?
    }

    /// Generate thumbnail for an image
    pub async fn generate_thumbnail(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), AppError> {
        let input_path = input_path.to_owned();
        let output_path = output_path.to_owned();
        let thumbnail_size = self.config.thumbnail_size;
        let _webp_quality = self.config.webp_quality;

        tokio::task::spawn_blocking(move || -> Result<(), AppError> {
            debug!("Generating thumbnail: {:?} -> {:?}", input_path, output_path);
            
            let img = image::open(&input_path)?;
            
            // Calculate thumbnail dimensions while maintaining aspect ratio
            let (orig_width, orig_height) = img.dimensions();
            let aspect_ratio = orig_width as f32 / orig_height as f32;
            
            let (thumb_width, thumb_height) = if aspect_ratio > 1.0 {
                // Landscape
                (thumbnail_size, (thumbnail_size as f32 / aspect_ratio) as u32)
            } else {
                // Portrait or square
                ((thumbnail_size as f32 * aspect_ratio) as u32, thumbnail_size)
            };
            
            let thumbnail = img.resize(
                thumb_width,
                thumb_height,
                image::imageops::FilterType::Lanczos3,
            );
            
            // Save as WebP for better compression
            thumbnail.save_with_format(&output_path, ImageFormat::WebP)?;
            
            info!("Successfully generated thumbnail: {:?}", output_path);
            Ok(())
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute thumbnail generation task".to_string()))?
    }

    /// Get image dimensions without loading the full image
    pub async fn get_dimensions(&self, path: &Path) -> Result<(u32, u32), AppError> {
        let path = path.to_owned();
        
        tokio::task::spawn_blocking(move || -> Result<(u32, u32), AppError> {
            let reader = image::ImageReader::open(&path)?;
            let dimensions = reader.into_dimensions()?;
            Ok(dimensions)
        })
        .await
        .map_err(|_| AppError::Internal("Failed to execute image dimensions task".to_string()))?
    }
}
