pub mod brush;
pub mod eraser;
pub mod selection;
pub mod transform;

pub use brush::BrushTool;
pub use eraser::EraserTool;
pub use selection::SelectionTool;
pub use transform::TransformTool;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrawingTool {
    Brush(BrushTool),
    Eraser(EraserTool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Brush(BrushTool),
    Eraser(EraserTool),
    Selection(SelectionTool),
    Transform(TransformTool),
}

impl Default for ToolType {
    fn default() -> Self {
        Self::Brush(BrushTool::default())
    }
} 