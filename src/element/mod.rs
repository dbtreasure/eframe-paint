use egui::{ColorImage, Context, Painter, Pos2, Rect, TextureHandle, Vec2};

// Re-export concrete implementations
mod common;
pub(crate) mod stroke;
pub(crate) mod image;
// We'll add text later
// pub(crate) mod text;

pub use common::MIN_ELEMENT_SIZE;
use crate::texture_manager::TextureGenerationError;

/// Common trait that all document elements must implement
pub trait Element {
    /// Get the unique identifier for this element
    fn id(&self) -> usize;
    
    /// Get the element type as a string
    fn element_type(&self) -> &'static str;
    
    /// Get the bounding rectangle for this element
    fn rect(&self) -> Rect;
    
    /// Draw the element using the provided painter
    fn draw(&self, painter: &Painter);
    
    /// Test if the element contains the given position
    fn hit_test(&self, pos: Pos2) -> bool;
    
    /// Translate the element by the given delta
    fn translate(&mut self, delta: Vec2) -> Result<(), String>;
    
    /// Resize the element to the new rectangle
    fn resize(&mut self, new_rect: Rect) -> Result<(), String>;
    
    /// Get the element's texture handle if available
    fn texture(&self) -> Option<&TextureHandle>;
    
    /// Check if the element needs a texture update
    fn needs_texture_update(&self) -> bool;
    
    /// Get the current texture version for cache invalidation
    fn texture_version(&self) -> u64;
    
    /// Invalidate the element's texture (called when element is modified)
    fn invalidate_texture(&mut self);
    
    /// Generate a texture for this element
    /// 
    /// This method should create a texture that represents the current state of the element.
    /// It's typically called by the TextureManager when a texture needs to be created or updated.
    fn generate_texture(&mut self, ctx: &Context) -> Result<ColorImage, TextureGenerationError>;
}

/// Enumeration of all element types in the document
#[derive(Clone)]
pub enum ElementType {
    Stroke(stroke::Stroke),
    Image(image::Image),
    // We'll add text later
    // Text(text::Text),
}

// Implement Debug for ElementType
impl std::fmt::Debug for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementType::Stroke(s) => {
                f.debug_tuple("Stroke")
                    .field(s)
                    .finish()
            },
            ElementType::Image(i) => {
                f.debug_tuple("Image")
                    .field(i)
                    .finish()
            },
        }
    }
}

// Export constants needed for compatibility
pub const RESIZE_HANDLE_RADIUS: f32 = 15.0;

impl ElementType {
    /// LEGACY: Get stable ID for backward compatibility
    pub fn get_stable_id(&self) -> usize {
        self.id()
    }
}

/// Legacy function for computing an element's rectangle with padding
/// This is kept for backward compatibility with existing code
pub fn compute_element_rect(element: &ElementType) -> egui::Rect {
    // Get the base rectangle from the Element trait
    let base_rect = element.rect();
    
    // Apply padding based on element type
    match element {
        ElementType::Stroke(_) => {
            // For strokes, add the base padding
            let padding = common::STROKE_BASE_PADDING;
            
            egui::Rect::from_min_max(
                egui::pos2(base_rect.min.x - padding, base_rect.min.y - padding),
                egui::pos2(base_rect.max.x + padding, base_rect.max.y + padding),
            )
        }
        ElementType::Image(_) => {
            // For images, add the image padding
            let padding = common::IMAGE_PADDING;
            egui::Rect::from_min_max(
                egui::pos2(base_rect.min.x - padding, base_rect.min.y - padding),
                egui::pos2(base_rect.max.x + padding, base_rect.max.y + padding),
            )
        }
    }
}

// Additional methods for ElementType that aren't part of the Element trait
impl ElementType {
    pub fn regenerate_texture(&mut self, ctx: &Context) -> bool {
        match self {
            ElementType::Stroke(s) => {
                if s.needs_texture_update() {
                    match s.generate_texture(ctx) {
                        Ok(_) => true,
                        Err(_) => false
                    }
                } else {
                    false
                }
            },
            ElementType::Image(i) => {
                if i.needs_texture_update() {
                    match i.generate_texture(ctx) {
                        Ok(_) => true,
                        Err(_) => false
                    }
                } else {
                    false
                }
            },
            // ElementType::Text(t) => t.regenerate_texture(ctx),
        }
    }
}

impl Element for ElementType {
    fn id(&self) -> usize {
        match self {
            ElementType::Stroke(s) => s.id(),
            ElementType::Image(i) => i.id(),
            // ElementType::Text(t) => t.id(),
        }
    }
    
    fn element_type(&self) -> &'static str {
        match self {
            ElementType::Stroke(_) => "stroke",
            ElementType::Image(_) => "image",
            // ElementType::Text(_) => "text",
        }
    }
    
    fn rect(&self) -> Rect {
        match self {
            ElementType::Stroke(s) => s.rect(),
            ElementType::Image(i) => i.rect(),
            // ElementType::Text(t) => t.rect(),
        }
    }
    
    fn draw(&self, painter: &Painter) {
        match self {
            ElementType::Stroke(s) => s.draw(painter),
            ElementType::Image(i) => i.draw(painter),
            // ElementType::Text(t) => t.draw(painter),
        }
    }
    
    fn hit_test(&self, pos: Pos2) -> bool {
        match self {
            ElementType::Stroke(s) => s.hit_test(pos),
            ElementType::Image(i) => i.hit_test(pos),
            // ElementType::Text(t) => t.hit_test(pos),
        }
    }
    
    fn translate(&mut self, delta: Vec2) -> Result<(), String> {
        match self {
            ElementType::Stroke(s) => s.translate(delta),
            ElementType::Image(i) => i.translate(delta),
            // ElementType::Text(t) => t.translate(delta),
        }
    }
    
    fn resize(&mut self, new_rect: Rect) -> Result<(), String> {
        match self {
            ElementType::Stroke(s) => s.resize(new_rect),
            ElementType::Image(i) => i.resize(new_rect),
            // ElementType::Text(t) => t.resize(new_rect),
        }
    }
    
    fn texture(&self) -> Option<&TextureHandle> {
        match self {
            ElementType::Stroke(s) => s.texture(),
            ElementType::Image(i) => i.texture(),
            // ElementType::Text(t) => t.texture(),
        }
    }
    
    fn needs_texture_update(&self) -> bool {
        match self {
            ElementType::Stroke(s) => s.needs_texture_update(),
            ElementType::Image(i) => i.needs_texture_update(),
            // ElementType::Text(t) => t.needs_texture_update(),
        }
    }
    
    fn texture_version(&self) -> u64 {
        match self {
            ElementType::Stroke(s) => s.texture_version(),
            ElementType::Image(i) => i.texture_version(),
            // ElementType::Text(t) => t.texture_version(),
        }
    }
    
    fn invalidate_texture(&mut self) {
        match self {
            ElementType::Stroke(s) => s.invalidate_texture(),
            ElementType::Image(i) => i.invalidate_texture(),
            // ElementType::Text(t) => t.invalidate_texture(),
        }
    }
    
    fn generate_texture(&mut self, ctx: &Context) -> Result<ColorImage, TextureGenerationError> {
        match self {
            ElementType::Stroke(s) => s.generate_texture(ctx),
            ElementType::Image(i) => i.generate_texture(ctx),
            // ElementType::Text(t) => t.generate_texture(ctx),
        }
    }
}

/// Factory functions for creating elements
pub mod factory {
    use super::*;
    use egui::{Color32, Pos2, Vec2};
    
    /// Create a new stroke element
    pub fn create_stroke(
        id: usize,
        points: Vec<Pos2>,
        thickness: f32,
        color: Color32
    ) -> ElementType {
        ElementType::Stroke(stroke::Stroke::new(id, points, thickness, color))
    }
    
    /// Create a new image element
    pub fn create_image(
        id: usize,
        data: Vec<u8>,
        size: Vec2,
        position: Pos2
    ) -> ElementType {
        ElementType::Image(image::Image::new(id, data, size, position))
    }
    
    // We'll add text factory later
    /*
    /// Create a new text element
    pub fn create_text(
        id: usize,
        content: String,
        font: egui::FontId,
        position: Pos2
    ) -> ElementType {
        ElementType::Text(text::Text::new(id, content, font, position))
    }
    */
}