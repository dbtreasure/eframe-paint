use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EraserTool {
    pub thickness: f32,
}

impl Default for EraserTool {
    fn default() -> Self {
        Self {
            thickness: 10.0,
        }
    }
} 