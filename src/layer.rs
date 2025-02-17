use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::stroke::Stroke;
use egui::TextureHandle;

/// Represents a single layer in the document
#[derive(Clone, Serialize, Deserialize)]
pub enum LayerContent {
    Strokes(Vec<Stroke>),
    Image {
        #[serde(skip)]
        texture: Option<TextureHandle>,
        size: [usize; 2],
    }
}

impl LayerContent {
    pub fn strokes(&self) -> Option<&Vec<Stroke>> {
        match self {
            LayerContent::Strokes(strokes) => Some(strokes),
            LayerContent::Image { .. } => None,
        }
    }
}

impl std::fmt::Debug for LayerContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerContent::Strokes(strokes) => f.debug_tuple("Strokes").field(strokes).finish(),
            LayerContent::Image { size, .. } => f
                .debug_struct("Image")
                .field("size", size)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Unique identifier for the layer
    pub id: Uuid,
    /// Display name of the layer
    pub name: String,
    /// Whether the layer is currently visible
    pub visible: bool,
    /// Content of the layer
    pub content: LayerContent,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            visible: true,
            content: LayerContent::Strokes(Vec::new()),
        }
    }

    pub fn new_image(name: &str, texture: TextureHandle, size: [usize; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            visible: true,
            content: LayerContent::Image { 
                texture: Some(texture),
                size,
            },
        }
    }

    /// Adds a stroke to the layer
    pub fn add_stroke(&mut self, stroke: Stroke) {
        if let LayerContent::Strokes(strokes) = &mut self.content {
            strokes.push(stroke);
        }
    }

    /// Removes and returns the last stroke from the layer
    pub fn remove_last_stroke(&mut self) -> Option<Stroke> {
        match &mut self.content {
            LayerContent::Strokes(strokes) => strokes.pop(),
            LayerContent::Image { .. } => None,
        }
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
        assert_eq!(layer.content.strokes().map(|strokes| strokes.len()).unwrap_or(0), 1);
        
        let removed = layer.remove_last_stroke();
        assert!(removed.is_some());
        assert_eq!(layer.content.strokes().map(|strokes| strokes.len()).unwrap_or(0), 0);
    }
} 