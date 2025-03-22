use egui::{
    Color32, ColorImage, Context, Painter, Pos2, Rect, Stroke as EguiStroke, TextureHandle, Vec2,
};
use log::info;

use super::Element;
use crate::element::common;
use crate::texture_manager::TextureGenerationError;

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

    /// Internal helper for generating a texture representation (used by the trait implementation)
    fn internal_generate_texture(&mut self) -> Result<ColorImage, TextureGenerationError> {
        // If we have no points, we can't generate a texture
        if self.points.is_empty() {
            return Err(TextureGenerationError::InvalidDimensions);
        }

        info!(
            "🖌️ Generating texture for stroke {}: {} points",
            self.id,
            self.points.len()
        );

        // Calculate bounds
        let bounds = self.rect();

        // Safety margins for stroke thickness
        let padding = self.thickness * 1.5;
        let width = (bounds.width() + padding * 2.0).max(1.0) as usize;
        let height = (bounds.height() + padding * 2.0).max(1.0) as usize;

        // Create a new color image
        let mut image = ColorImage::new([width, height], Color32::TRANSPARENT);

        // Offset points to the image coordinate space
        let offset = Vec2::new(bounds.min.x - padding, bounds.min.y - padding);
        let transformed_points: Vec<Pos2> = self
            .points
            .iter()
            .map(|p| Pos2::new(p.x - offset.x, p.y - offset.y))
            .collect();

        // Draw the stroke to the image
        // This is a simplified approach that draws color blocks along the stroke path
        if transformed_points.len() >= 2 {
            for window in transformed_points.windows(2) {
                let (p1, p2) = (window[0], window[1]);

                // Draw line from p1 to p2
                // Simple Bresenham-like algorithm
                let dist = p1.distance(p2);
                let steps = (dist * 2.0).ceil() as usize;

                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    let point = p1.lerp(p2, t);

                    // Draw a circle at this point
                    let radius = (self.thickness / 2.0).ceil() as i32;

                    for dy in -radius..=radius {
                        for dx in -radius..=radius {
                            let d = (dx * dx + dy * dy) as f32;
                            if d <= (radius as f32 * radius as f32) {
                                let x = (point.x + dx as f32) as i32;
                                let y = (point.y + dy as f32) as i32;

                                // Check bounds
                                if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                                    let idx = y as usize * width + x as usize;
                                    if idx < image.pixels.len() {
                                        image.pixels[idx] = self.color;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Mark as not needing update
        self.texture_needs_update = false;

        Ok(image)
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
            EguiStroke::new(self.thickness, self.color),
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

    // Only keep the generate_texture method, remove regenerate_texture

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

    // Element trait implementation for generate_texture
    // Implementation of the Element trait method
    fn generate_texture(&mut self, _ctx: &Context) -> Result<ColorImage, TextureGenerationError> {
        // Call the internal implementation
        let result = self.internal_generate_texture();

        // Mark as not needing update if successful
        if result.is_ok() {
            self.texture_needs_update = false;
        }

        result
    }
}
