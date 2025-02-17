// src/document.rs
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// A basic stroke representation for painting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    // Placeholder for now - will be expanded later
    pub points: Vec<(f32, f32)>,
}

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
    /// Creates a new layer with the given name
    /// 
    /// Args:
    ///     name (str): The name for the new layer
    ///
    /// Returns:
    ///     Layer: A new layer instance with default properties
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            visible: true,
            strokes: Vec::new(),
        }
    }
}

/// The main document structure containing all layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Vector of layers in the document
    pub layers: Vec<Layer>,
    /// Index of the currently active layer
    pub active_layer: Option<usize>,
}

impl Document {
    /// Adds a new layer with the given name
    /// 
    /// Args:
    ///     name (str): The name for the new layer
    pub fn add_layer(&mut self, name: &str) {
        self.layers.push(Layer::new(name));
        self.active_layer = Some(self.layers.len() - 1);
    }

    /// Removes the layer at the given index
    /// 
    /// Args:
    ///     index (usize): The index of the layer to remove
    pub fn remove_layer(&mut self, index: usize) {
        if index < self.layers.len() {
            self.layers.remove(index);
            // Update active layer (simple logic: set to last layer if available)
            self.active_layer = if self.layers.is_empty() {
                None
            } else {
                Some(self.layers.len() - 1)
            };
        }
    }

    /// Gets a reference to the currently active layer, if one exists
    /// 
    /// Returns:
    ///     Option<&Layer>: Reference to the active layer, or None if no layer is active
    pub fn active_layer(&self) -> Option<&Layer> {
        self.active_layer.and_then(|idx| self.layers.get(idx))
    }

    /// Gets a mutable reference to the currently active layer, if one exists
    /// 
    /// Returns:
    ///     Option<&mut Layer>: Mutable reference to the active layer, or None if no layer is active
    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.active_layer.and_then(|idx| self.layers.get_mut(idx))
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            layers: vec![Layer::new("Background")],
            active_layer: Some(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_document_has_background_layer() {
        let doc = Document::default();
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.layers[0].name, "Background");
        assert_eq!(doc.active_layer, Some(0));
    }

    #[test]
    fn test_add_layer() {
        let mut doc = Document::default();
        let initial_count = doc.layers.len();
        doc.add_layer("New Layer");
        assert_eq!(doc.layers.len(), initial_count + 1);
    }

    #[test]
    fn test_remove_layer() {
        let mut doc = Document::default();
        doc.add_layer("New Layer");
        let initial_count = doc.layers.len();
        doc.remove_layer(1);
        assert_eq!(doc.layers.len(), initial_count - 1);
    }

    // Keep the existing specific test cases as well
    #[test]
    fn test_add_layer_specific() {
        let mut doc = Document::default();
        doc.add_layer("Layer 1");
        assert_eq!(doc.layers.len(), 2);
        assert_eq!(doc.layers[1].name, "Layer 1");
        assert_eq!(doc.active_layer, Some(1));
    }

    #[test]
    fn test_remove_layer_specific() {
        let mut doc = Document::default();
        doc.add_layer("Layer 1");
        doc.remove_layer(0);
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.layers[0].name, "Layer 1");
        assert_eq!(doc.active_layer, Some(0));
    }
}