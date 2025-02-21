// src/document.rs
use serde::{Serialize, Deserialize};
use crate::layer::{Layer, LayerId, Transform};
use crate::command::{Command, CommandContext, CommandResult, CommandError};
use crate::selection::Selection;
use crate::state::EditorContext;
use crate::renderer::Renderer;
use crate::event::EventBus;
use crate::tool::ToolType;
use eframe::egui::TextureHandle;

/// The main document structure containing all layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Vector of layers in the document
    pub layers: Vec<Layer>,
    /// Index of the currently active layer
    pub active_layer: Option<usize>,
    /// Current selection in the document
    pub current_selection: Option<Selection>,
}

impl Document {
    /// Creates a new empty document
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            active_layer: None,
            current_selection: None,
        }
    }

    /// Gets a reference to a layer by its ID
    pub fn get_layer(&self, id: LayerId) -> Result<&Layer, CommandError> {
        self.layers.get(id.index())
            .ok_or(CommandError::InvalidParameters)
    }

    /// Gets a mutable reference to a layer by its ID
    pub fn get_layer_mut(&mut self, id: LayerId) -> Result<&mut Layer, CommandError> {
        self.layers.get_mut(id.index())
            .ok_or(CommandError::InvalidParameters)
    }

    /// Gets the current selection
    pub fn current_selection(&self) -> &Option<Selection> {
        &self.current_selection
    }

    /// Sets the current selection
    pub fn set_selection(&mut self, selection: Selection) {
        self.current_selection = Some(selection);
    }

    /// Clears the current selection
    pub fn clear_selection(&mut self) {
        self.current_selection = None;
    }

    /// Adds a new layer with the given name
    pub fn add_layer(&mut self, name: &str) {
        let layer = Layer::new(name);
        self.layers.push(layer);
        self.active_layer = Some(self.layers.len() - 1);
    }

    /// Removes a layer at the given index
    pub fn remove_layer(&mut self, index: usize) {
        if index < self.layers.len() {
            self.layers.remove(index);
            // Update active layer if needed
            if let Some(active) = self.active_layer {
                if active >= self.layers.len() {
                    self.active_layer = if self.layers.is_empty() {
                        None
                    } else {
                        Some(self.layers.len() - 1)
                    };
                }
            }
        }
    }

    /// Gets a reference to the active layer
    pub fn active_layer(&self) -> Option<&Layer> {
        self.active_layer.and_then(|idx| self.layers.get(idx))
    }

    /// Gets a mutable reference to the active layer
    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.active_layer.and_then(|idx| self.layers.get_mut(idx))
    }

    /// Executes a command on the document
    pub fn execute_command(&mut self, command: Command) -> CommandResult {
        command.execute(&mut CommandContext::new(
            self.clone(),
            EditorContext::new(self.clone(), Renderer::default()),
            EventBus::new(),
            ToolType::default(),
        ))
    }

    /// Handles a layer being added at the specified index
    pub fn handle_layer_added(&mut self, index: usize) {
        // If index is out of bounds, append to end
        let name = format!("Layer {}", self.layers.len() + 1);
        let layer = Layer::new(&name);
        if index >= self.layers.len() {
            self.layers.push(layer);
        } else {
            self.layers.insert(index, layer);
        }
    }

    /// Handles a layer being removed at the specified index
    pub fn handle_layer_removed(&mut self, index: usize) {
        if index < self.layers.len() {
            self.layers.remove(index);
            // Update active layer if needed
            if let Some(active) = self.active_layer {
                if active >= self.layers.len() {
                    self.active_layer = if self.layers.is_empty() {
                        None
                    } else {
                        Some(self.layers.len() - 1)
                    };
                }
            }
        }
    }

    /// Handles a layer being reordered from one index to another
    pub fn handle_layer_reordered(&mut self, from: usize, to: usize) {
        if from < self.layers.len() && to < self.layers.len() {
            let layer = self.layers.remove(from);
            self.layers.insert(to, layer);
            // Update active layer if needed
            if let Some(active) = self.active_layer {
                if active == from {
                    self.active_layer = Some(to);
                }
            }
        }
    }

    /// Handles a layer's transform being changed
    pub fn handle_layer_transformed(&mut self, index: usize, _old_transform: Transform, new_transform: Transform) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.transform = new_transform;
        }
    }

    /// Handles a layer's visibility being changed
    pub fn handle_layer_visibility_changed(&mut self, index: usize, visible: bool) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = visible;
        }
    }

    /// Toggles the visibility of a layer
    pub fn toggle_layer_visibility(&mut self, index: usize) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = !layer.visible;
        }
    }

    /// Undoes the last command
    pub fn undo(&mut self) {
        // TODO: Implement undo functionality
    }

    /// Redoes the last undone command
    pub fn redo(&mut self) {
        // TODO: Implement redo functionality
    }

    pub fn add_image_layer(&mut self, name: &str, texture: TextureHandle) {
        let size = [100, 100]; // Default size, should be replaced with actual image dimensions
        let layer = Layer::new_image(name, texture, size);
        self.layers.push(layer);
        self.active_layer = Some(self.layers.len() - 1);
    }

    pub fn reorder_layer(&mut self, layer_id: LayerId, new_index: usize) {
        let old_index = layer_id.0;
        if old_index >= self.layers.len() || new_index >= self.layers.len() {
            return;
        }
        let layer = self.layers.remove(old_index);
        self.layers.insert(new_index, layer);
        if let Some(active) = self.active_layer {
            if active == old_index {
                self.active_layer = Some(new_index);
            } else if active > old_index && active <= new_index {
                self.active_layer = Some(active - 1);
            } else if active < old_index && active >= new_index {
                self.active_layer = Some(active + 1);
            }
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Stroke;

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
}