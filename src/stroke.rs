use serde::{Serialize, Deserialize};
use egui;

/// A basic stroke representation for painting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    /// Placeholder for now - will be expanded later
    pub points: Vec<(f32, f32)>,
    pub thickness: f32,
    pub color: egui::Color32,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            thickness: 2.0,
            color: egui::Color32::BLACK,
        }
    }
} 