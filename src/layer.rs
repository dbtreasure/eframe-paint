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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_visibility() {
        let mut layer = Layer::new("Test Layer");
        assert!(layer.visible);
        layer.visible = false;
        assert!(!layer.visible);
    }

    #[test]
    fn test_stroke_operations() {
        let mut layer = Layer::new("Test Layer");
        let stroke = Stroke::default();
        
        layer.add_stroke(stroke);
        assert_eq!(layer.strokes.len(), 1);
        
        let removed = layer.remove_last_stroke();
        assert!(removed.is_some());
        assert_eq!(layer.strokes.len(), 0);
    }
} 