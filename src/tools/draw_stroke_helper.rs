use crate::renderer::StrokePreview;
use egui::{Color32, Pos2};

/// Helper struct for creating strokes during drawing
/// 
/// This replaces the legacy MutableStroke and provides only the functionality
/// needed by the DrawStrokeTool to gather points and properties during stroke creation.
#[derive(Clone)]
pub struct DrawStrokeHelper {
    points: Vec<Pos2>,
    color: Color32,
    thickness: f32,
}

impl DrawStrokeHelper {
    /// Create a new helper for stroke drawing
    pub fn new(color: Color32, thickness: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            thickness,
        }
    }

    /// Add a point to the stroke
    pub fn add_point(&mut self, point: Pos2) {
        self.points.push(point);
    }

    /// Get the current points
    pub fn points(&self) -> &[Pos2] {
        &self.points
    }

    /// Get the stroke color
    pub fn color(&self) -> Color32 {
        self.color
    }

    /// Get the stroke thickness
    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    /// Convert to a StrokePreview for rendering
    pub fn to_stroke_preview(&self) -> StrokePreview {
        StrokePreview::new(self.points.clone(), self.thickness, self.color)
    }
}