use egui::{ColorImage, Context, Painter, Pos2, Rect, TextureHandle, Vec2, Color32};
use log::info;

use super::Element;
use crate::element::common;
use crate::texture_manager::TextureGenerationError;

/// Image element representing a bitmap image
#[derive(Clone)]
pub(crate) struct Image {
    // Core properties
    id: usize,
    data: Vec<u8>, // Raw image data
    size: Vec2,    // Width and height
    position: Pos2, // Position in the document
    
    // Texture caching
    texture_handle: Option<TextureHandle>,
    texture_needs_update: bool,
    texture_version: u64,
}

// Custom Debug implementation since TextureHandle doesn't implement Debug
impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("id", &self.id)
            .field("data_len", &self.data.len())
            .field("size", &self.size)
            .field("position", &self.position)
            .field("texture_needs_update", &self.texture_needs_update)
            .field("texture_version", &self.texture_version)
            .finish()
    }
}

impl Image {
    /// Create a new image with the given properties
    pub(crate) fn new(id: usize, data: Vec<u8>, size: Vec2, position: Pos2) -> Self {
        Self {
            id,
            data,
            size,
            position,
            texture_handle: None,
            texture_needs_update: true,
            texture_version: 0,
        }
    }

    /// Get the image data
    pub(crate) fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the image size
    pub(crate) fn size(&self) -> Vec2 {
        self.size
    }

    /// Get the image position
    pub(crate) fn position(&self) -> Pos2 {
        self.position
    }
    
    /// Generates a texture representation of the image
    fn generate_texture_internal(&mut self, _ctx: &Context) -> Result<ColorImage, TextureGenerationError> {
        info!("🖼️ Generating texture for image {}: {}x{}", self.id, self.size.x, self.size.y);
        
        #[cfg(feature = "image_support")]
        {
            if let Ok(image) = image::load_from_memory(&self.data) {
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                
                // Mark as not needing update
                self.texture_needs_update = false;
                
                return Ok(ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice()
                ));
            } else {
                info!("❌ Failed to load image data for image {}", self.id);
                return Err(TextureGenerationError::GenerationFailed);
            }
        }
        
        #[cfg(not(feature = "image_support"))]
        {
            info!("⚠️ Image support not enabled");
            return Err(TextureGenerationError::GenerationFailed);
        }
    }
}

impl Element for Image {
    fn id(&self) -> usize {
        self.id
    }
    
    fn element_type(&self) -> &'static str {
        "image"
    }
    
    fn rect(&self) -> Rect {
        Rect::from_min_size(self.position, self.size)
    }
    
    fn draw(&self, painter: &Painter) {
        // If we have a texture, use it
        if let Some(texture) = &self.texture_handle {
            let rect = self.rect();
            
            // Draw the texture
            painter.image(texture.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        } else {
            // Draw a placeholder rectangle
            let rect = self.rect();
            painter.rect_filled(rect, 0.0, Color32::from_gray(200));
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, Color32::from_gray(100)));
        }
    }
    
    fn hit_test(&self, pos: Pos2) -> bool {
        self.rect().contains(pos)
    }
    
    fn translate(&mut self, delta: Vec2) -> Result<(), String> {
        self.position += delta;
        // No need to invalidate texture for translation
        Ok(())
    }
    
    fn resize(&mut self, new_rect: Rect) -> Result<(), String> {
        common::validate_rect(&new_rect)?;
        
        // Update position and size
        self.position = new_rect.min;
        self.size = new_rect.size();
        
        // No need to invalidate texture for basic resize since we're just
        // displaying the same texture in a different size
        
        info!("✅ Image {} resized: pos={:?}, size={:?}", self.id, self.position, self.size);
        Ok(())
    }
    
    fn texture(&self) -> Option<&TextureHandle> {
        self.texture_handle.as_ref()
    }
    
    // Remove regenerate_texture method
    
    fn needs_texture_update(&self) -> bool {
        self.texture_needs_update
    }
    
    fn texture_version(&self) -> u64 {
        self.texture_version
    }
    
    fn invalidate_texture(&mut self) {
        self.texture_needs_update = true;
        self.texture_version += 1;
    }
    
    fn generate_texture(&mut self, ctx: &Context) -> Result<ColorImage, TextureGenerationError> {
        // Call the internal implementation
        self.generate_texture_internal(ctx)
    }
}