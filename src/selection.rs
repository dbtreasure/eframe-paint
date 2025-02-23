use eframe::egui::{Rect, Pos2};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SelectionShape {
    Rectangle(Rect),
    Freeform(Vec<Pos2>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Selection {
    pub shape: SelectionShape,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SelectionMode {
    Rectangle,
    Freeform,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectionInProgress {
    pub start: Pos2,
    pub current: Pos2,
    pub mode: SelectionMode,
    pub points: Vec<Pos2>,
} 