use egui::{Color32, Context, Painter, Pos2, Rect, Stroke as EguiStroke, TextureHandle, Vec2};
use log::info;

use super::Element;
use crate::element::common;

/// Stroke element representing a series of connected points
#[derive(Clone)]
pub(crate) struct Stroke {
    // Core properties
    id: usize,
    points: Vec<Pos2>,
    color: Color32,
    thickness: f32,
    
    // Texture caching
    texture_handle: Option<TextureHandle>,
    texture_needs_update: bool,
    texture_version: u64,
}

// Custom Debug implementation since TextureHandle doesn't implement Debug
impl std::fmt::Debug for Stroke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stroke")
            .field("id", &self.id)
            .field("points", &self.points)
            .field("color", &self.color)
            .field("thickness", &self.thickness)
            .field("texture_needs_update", &self.texture_needs_update)
            .field("texture_version", &self.texture_version)
            .finish()
    }
}

impl Stroke {
    /// Create a new stroke with the given properties
    pub(crate) fn new(id: usize, points: Vec<Pos2>, thickness: f32, color: Color32) -> Self {
        Self {
            id,
            points,
            color,
            thickness,
            texture_handle: None,
            texture_needs_update: true,
            texture_version: 0,
        }
    }

    /// Get the points that make up this stroke
    pub(crate) fn points(&self) -> &[Pos2] {
        &self.points
    }

    /// Get the stroke color
    pub(crate) fn color(&self) -> Color32 {
        self.color
    }

    /// Get the stroke thickness
    pub(crate) fn thickness(&self) -> f32 {
        self.thickness
    }
    
    /// Generates a textured representation of the stroke
    fn generate_texture(&mut self, _ctx: &Context) -> bool {
        // If we have no points, we can't generate a texture
        if self.points.is_empty() {
            return false;
        }
        
        // This is a simplified approach - in a full implementation,
        // we would render the stroke to a texture here
        // For now, we'll just mark it as not needing update
        
        // The real implementation would:
        // 1. Create an off-screen render target at the appropriate size
        // 2. Draw the stroke to that target
        // 3. Convert to a texture
        
        info!("🖌️ Generating texture for stroke {}: {} points", self.id, self.points.len());
        
        // Since we're not actually generating a texture yet, we'll return 
        // false to indicate nothing changed
        self.texture_needs_update = false;
        true
    }
}

impl Element for Stroke {
    fn id(&self) -> usize {
        self.id
    }
    
    fn element_type(&self) -> &'static str {
        "stroke"
    }
    
    fn rect(&self) -> Rect {
        // Calculate bounding box from points with padding for stroke thickness
        if self.points.is_empty() {
            return Rect::NOTHING;
        }
        
        common::calculate_bounds(&self.points, self.thickness / 2.0)
    }
    
    fn draw(&self, painter: &Painter) {
        // For now, we use direct line drawing
        // In the final implementation, this would use the texture
        if self.points.len() < 2 {
            return;
        }
        
        painter.add(egui::Shape::line(
            self.points.clone(),
            EguiStroke::new(self.thickness, self.color)
        ));
    }
    
    fn hit_test(&self, pos: Pos2) -> bool {
        // For simplicity, check if the position is close to any line segment
        if self.points.len() < 2 {
            return false;
        }
        
        for window in self.points.windows(2) {
            let distance = common::distance_to_line_segment(pos, window[0], window[1]);
            if distance <= self.thickness / 2.0 {
                return true;
            }
        }
        
        false
    }
    
    fn translate(&mut self, delta: Vec2) -> Result<(), String> {
        for point in &mut self.points {
            *point += delta;
        }
        
        self.invalidate_texture();
        Ok(())
    }
    
    fn resize(&mut self, new_rect: Rect) -> Result<(), String> {
        common::validate_rect(&new_rect)?;
        
        let old_rect = self.rect();
        if old_rect == Rect::NOTHING {
            return Err("Cannot resize empty stroke".to_string());
        }
        
        // Calculate scale factors
        let scale_x = new_rect.width() / old_rect.width();
        let scale_y = new_rect.height() / old_rect.height();
        
        // Transform each point
        for point in &mut self.points {
            // Convert to relative coordinates in the original rect
            let relative_x = (point.x - old_rect.min.x) / old_rect.width();
            let relative_y = (point.y - old_rect.min.y) / old_rect.height();
            
            // Apply to new rect
            point.x = new_rect.min.x + (relative_x * new_rect.width());
            point.y = new_rect.min.y + (relative_y * new_rect.height());
        }
        
        // Scale thickness proportionally
        self.thickness *= (scale_x + scale_y) / 2.0;
        
        self.invalidate_texture();
        Ok(())
    }
    
    fn texture(&self) -> Option<&TextureHandle> {
        self.texture_handle.as_ref()
    }
    
    fn regenerate_texture(&mut self, ctx: &Context) -> bool {
        if self.needs_texture_update() {
            self.generate_texture(ctx)
        } else {
            false
        }
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
}