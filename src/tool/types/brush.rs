use eframe::egui::Color32;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushTool {
    pub color: Color32,
    pub thickness: f32,
}

impl Default for BrushTool {
    fn default() -> Self {
        Self {
            color: Color32::BLACK,
            thickness: 1.0,
        }
    }
} 