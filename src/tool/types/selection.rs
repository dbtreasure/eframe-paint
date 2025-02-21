use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SelectionMode {
    Rectangle,
    Freeform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionTool {
    pub mode: SelectionMode,
}

impl Default for SelectionTool {
    fn default() -> Self {
        Self {
            mode: SelectionMode::Rectangle,
        }
    }
} 