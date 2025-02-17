use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::stroke::Stroke;

/// Represents a single layer in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Unique identifier for the layer
    pub id: Uuid,
    /// Display name of the layer
    pub name: String,
    /// Whether the layer is currently visible
    pub visible: bool,
    /// Collection of strokes on this layer
    pub strokes: Vec<Stroke>,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            visible: true,
            strokes: Vec::new(),
        }
    }

    /// Adds a stroke to the layer
    pub fn add_stroke(&mut self, stroke: Stroke) {
        self.strokes.push(stroke);
    }

    /// Removes and returns the last stroke from the layer
    pub fn remove_last_stroke(&mut self) -> Option<Stroke> {
        self.strokes.pop()
    }
} 