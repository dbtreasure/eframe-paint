use egui::{Color32, ColorImage, Context, Painter, Pos2, Rect, TextureHandle, Vec2};
use log::info;

use super::Element;
use crate::element::common;
use crate::texture_manager::TextureGenerationError;

/// Image element representing a bitmap image
#[derive(Clone)]
pub(crate) struct Image {
    // Core properties
    id: usize,
    original_data: Vec<u8>,  // Original image data (JPG, PNG, etc)
    rgba_data: Vec<u8>,      // Processed RGBA data
    size: Vec2,              // Width and height
    position: Pos2,          // Position in the document

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
            .field("original_data_len", &self.original_data.len())
            .field("rgba_data_len", &self.rgba_data.len())
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
        // Store original data and create empty RGBA data (will be populated in generate_texture)
        Self {
            id,
            original_data: data,
            rgba_data: Vec::new(),
            size,
            position,
            texture_handle: None,
            texture_needs_update: true,
            texture_version: 0,
        }
    }

    /// Get the image data
    pub(crate) fn data(&self) -> &[u8] {
        &self.original_data
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
        let target_width = self.size.x as usize;
        let target_height = self.size.y as usize;
        
        // Try to load as standard image format from original data
        if let Ok(img) = image::load_from_memory(&self.original_data) {
            info!("✅ Successfully loaded image format: {:?}", img.color());
            
            let resized = img.resize_exact(
                target_width as u32,
                target_height as u32,
                image::imageops::FilterType::Lanczos3
            );
            let rgba = resized.to_rgba8();
            
            // Store the RGBA data for future use
            self.rgba_data = rgba.as_raw().to_vec();
            self.texture_needs_update = false;
            
            return Ok(ColorImage::from_rgba_unmultiplied(
                [target_width, target_height],
                &self.rgba_data
            ));
        }

        // If standard format loading fails, log error and fail
        info!("❌ Failed to process image data: len={}, format=unknown, target={}x{}", 
              self.original_data.len(), target_width, target_height);
        Err(TextureGenerationError::GenerationFailed)
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
            painter.image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
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

        // Invalidate texture when resizing since we need to adjust for the new dimensions
        self.invalidate_texture();

        info!(
            "✅ Image {} resized: pos={:?}, size={:?}",
            self.id, self.position, self.size
        );
        Ok(())
    }

    fn texture(&self) -> Option<&TextureHandle> {
        self.texture_handle.as_ref()
    }

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
        self.generate_texture_internal(ctx)
    }
}
