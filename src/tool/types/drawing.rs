#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DrawingTool {
    Brush(BrushTool),
    Eraser(EraserTool),
} 