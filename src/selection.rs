use eframe::egui::{Rect, Pos2};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionShape {
    Rectangle(Rect),
    Freeform(Vec<Pos2>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub shape: SelectionShape,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SelectionMode {
    Rectangle,
    Freeform,
} 