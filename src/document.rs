// src/document.rs
use serde::{Serialize, Deserialize};
use crate::layer::Layer;
use crate::command::Command;
use crate::Stroke;

/// The main document structure containing all layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Vector of layers in the document
    pub layers: Vec<Layer>,
    /// Index of the currently active layer
    pub active_layer: Option<usize>,
    /// Stack of commands that can be undone
    undo_stack: Vec<Command>,
    /// Stack of commands that can be redone
    redo_stack: Vec<Command>,
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

    /// Executes a command and adds it to the undo stack
    pub fn execute_command(&mut self, command: Command) {
        match &command {
            Command::AddStroke { layer_index, stroke } => {
                if let Some(layer) = self.layers.get_mut(*layer_index) {
                    layer.add_stroke(stroke.clone());
                }
            }
        }
        self.undo_stack.push(command);
        self.redo_stack.clear(); // Clear redo stack when new action is performed
    }

    /// Undoes the last command
    pub fn undo(&mut self) {
        if let Some(cmd) = self.undo_stack.pop() {
            match &cmd {
                Command::AddStroke { layer_index, stroke: _ } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        layer.strokes.pop();
                    }
                }
            }
            self.redo_stack.push(cmd);
        }
    }

    /// Redoes the last undone command
    pub fn redo(&mut self) {
        if let Some(cmd) = self.redo_stack.pop() {
            match &cmd {
                Command::AddStroke { layer_index, stroke } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        layer.strokes.push(stroke.clone());
                    }
                }
            }
            self.undo_stack.push(cmd);
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            layers: vec![Layer::new("Background")],
            active_layer: Some(0),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
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

    #[test]
    fn test_undo_redo_stroke() {
        let mut doc = Document::default();
        let stroke = Stroke::default();
        
        // Execute a stroke command
        doc.execute_command(Command::AddStroke {
            layer_index: 0,
            stroke: stroke.clone(),
        });
        assert_eq!(doc.layers[0].strokes.len(), 1);
        
        // Test undo
        doc.undo();
        assert_eq!(doc.layers[0].strokes.len(), 0);
        
        // Test redo
        doc.redo();
        assert_eq!(doc.layers[0].strokes.len(), 1);
    }

    #[test]
    fn test_redo_stack_cleared() {
        let mut doc = Document::default();
        let stroke1 = Stroke::default();
        let stroke2 = Stroke::default();
        
        // Add first stroke and undo it
        doc.execute_command(Command::AddStroke {
            layer_index: 0,
            stroke: stroke1,
        });
        doc.undo();
        
        // Add second stroke - should clear redo stack
        doc.execute_command(Command::AddStroke {
            layer_index: 0,
            stroke: stroke2,
        });
        
        // Try to redo - should do nothing since redo stack was cleared
        doc.redo();
        assert_eq!(doc.layers[0].strokes.len(), 1);
    }

    #[test]
    fn test_active_layer_methods() {
        let mut doc = Document::default();
        doc.add_layer("Layer 1");
        
        // Test active_layer()
        let layer = doc.active_layer().unwrap();
        assert_eq!(layer.name, "Layer 1");
        
        // Test active_layer_mut()
        let layer = doc.active_layer_mut().unwrap();
        layer.visible = false;
        assert!(!doc.layers[1].visible);
    }

    #[test]
    fn test_execute_command() {
        let mut doc = Document::default();
        let stroke = Stroke::default();
        
        // Test that execute_command properly adds the stroke
        doc.execute_command(Command::AddStroke {
            layer_index: 0,
            stroke: stroke.clone(),
        });
        
        assert_eq!(doc.undo_stack.len(), 1);
        assert_eq!(doc.redo_stack.len(), 0);
        assert_eq!(doc.layers[0].strokes.len(), 1);
    }
}