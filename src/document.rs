// src/document.rs
use serde::{Serialize, Deserialize};
use crate::layer::{Layer, LayerContent, Transform};
use crate::command::Command;

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
    pub fn add_layer(&mut self, name: &str) {
        let command = Command::AddLayer {
            name: name.to_string(),
        };
        self.execute_command(command);
    }

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

    pub fn active_layer(&self) -> Option<&Layer> {
        self.active_layer.and_then(|idx| self.layers.get(idx))
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.active_layer.and_then(|idx| self.layers.get_mut(idx))
    }

    pub fn execute_command(&mut self, command: Command) {
        match &command {
            Command::AddStroke { layer_index, stroke } => {
                if let Some(layer) = self.layers.get_mut(*layer_index) {
                    if let LayerContent::Strokes(strokes) = &mut layer.content {
                        strokes.push(stroke.clone());
                    }
                }
            }
            Command::AddImageLayer { name, texture, size, initial_transform } => {
                if let Some(texture) = texture {
                    let mut layer = Layer::new_image(name, texture.clone(), *size);
                    layer.transform = *initial_transform;
                    self.layers.insert(0, layer);
                    self.active_layer = Some(0);
                }
            }
            Command::AddLayer { name } => {
                self.layers.insert(0, Layer::new(name));
                self.active_layer = Some(0);
            }
            Command::TransformLayer { layer_index, new_transform, .. } => {
                if let Some(layer) = self.layers.get_mut(*layer_index) {
                    layer.transform = *new_transform;
                }
            }
            Command::ReorderLayer { from_index, to_index } => {
                if *from_index < self.layers.len() && *to_index < self.layers.len() {
                    let layer = self.layers.remove(*from_index);
                    self.layers.insert(*to_index, layer);
                    // Update active layer index if needed
                    if let Some(active_idx) = self.active_layer {
                        self.active_layer = Some(if active_idx == *from_index {
                            *to_index
                        } else if active_idx < *from_index && active_idx > *to_index {
                            active_idx + 1
                        } else if active_idx > *from_index && active_idx < *to_index {
                            active_idx - 1
                        } else {
                            active_idx
                        });
                    }
                }
            }
            Command::RenameLayer { layer_index, new_name, .. } => {
                if let Some(layer) = self.layers.get_mut(*layer_index) {
                    layer.name = new_name.clone();
                }
            }
        }
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(cmd) = self.undo_stack.pop() {
            match &cmd {
                Command::AddStroke { layer_index, .. } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        if let LayerContent::Strokes(strokes) = &mut layer.content {
                            strokes.pop();
                        }
                    }
                }
                Command::AddImageLayer { .. } | Command::AddLayer { .. } => {
                    self.layers.remove(0);
                    self.active_layer = if self.layers.is_empty() {
                        None
                    } else {
                        Some(0)
                    };
                }
                Command::TransformLayer { layer_index, old_transform, .. } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        layer.transform = *old_transform;
                    }
                }
                Command::ReorderLayer { from_index, to_index } => {
                    if *from_index < self.layers.len() && *to_index < self.layers.len() {
                        let layer = self.layers.remove(*to_index);
                        self.layers.insert(*from_index, layer);
                        // Update active layer index if needed
                        if let Some(active_idx) = self.active_layer {
                            self.active_layer = Some(if active_idx == *to_index {
                                *from_index
                            } else if active_idx < *to_index && active_idx > *from_index {
                                active_idx - 1
                            } else if active_idx > *to_index && active_idx < *from_index {
                                active_idx + 1
                            } else {
                                active_idx
                            });
                        }
                    }
                }
                Command::RenameLayer { layer_index, old_name, .. } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        layer.name = old_name.clone();
                    }
                }
            }
            self.redo_stack.push(cmd);
        }
    }

    pub fn redo(&mut self) {
        if let Some(cmd) = self.redo_stack.pop() {
            match &cmd {
                Command::AddStroke { layer_index, stroke } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        if let LayerContent::Strokes(strokes) = &mut layer.content {
                            strokes.push(stroke.clone());
                        }
                    }
                }
                Command::AddImageLayer { name, texture, size, initial_transform } => {
                    if let Some(texture) = texture {
                        let mut layer = Layer::new_image(name, texture.clone(), *size);
                        layer.transform = *initial_transform;
                        self.layers.insert(0, layer);
                        self.active_layer = Some(0);
                    }
                }
                Command::AddLayer { name } => {
                    self.layers.insert(0, Layer::new(name));
                    self.active_layer = Some(0);
                }
                Command::TransformLayer { layer_index, new_transform, .. } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        layer.transform = *new_transform;
                    }
                }
                Command::ReorderLayer { from_index, to_index } => {
                    if *from_index < self.layers.len() && *to_index < self.layers.len() {
                        let layer = self.layers.remove(*from_index);
                        self.layers.insert(*to_index, layer);
                        // Update active layer index if needed
                        if let Some(active_idx) = self.active_layer {
                            self.active_layer = Some(if active_idx == *from_index {
                                *to_index
                            } else if active_idx < *from_index && active_idx > *to_index {
                                active_idx + 1
                            } else if active_idx > *from_index && active_idx < *to_index {
                                active_idx - 1
                            } else {
                                active_idx
                            });
                        }
                    }
                }
                Command::RenameLayer { layer_index, new_name, .. } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        layer.name = new_name.clone();
                    }
                }
            }
            self.undo_stack.push(cmd);
        }
    }

    pub fn add_image_layer(&mut self, name: &str, texture: egui::TextureHandle) {
        let size = texture.size();
        let command = Command::AddImageLayer {
            name: name.to_string(),
            texture: Some(texture),
            size,
            initial_transform: Transform::default(),
        };
        self.execute_command(command);
    }

    pub fn toggle_layer_visibility(&mut self, index: usize) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = !layer.visible;
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
    fn test_undo_redo_stroke() {
        let mut doc = Document::default();
        let stroke = Stroke::default();
        
        doc.execute_command(Command::AddStroke {
            layer_index: 0,
            stroke: stroke.clone(),
        });
        
        let strokes_len = |layer: &Layer| {
            if let LayerContent::Strokes(strokes) = &layer.content {
                strokes.len()
            } else {
                0
            }
        };

        assert_eq!(strokes_len(&doc.layers[0]), 1);
        doc.undo();
        assert_eq!(strokes_len(&doc.layers[0]), 0);
        doc.redo();
        assert_eq!(strokes_len(&doc.layers[0]), 1);
    }

    #[test]
    fn test_redo_stack_cleared() {
        let mut doc = Document::default();
        let stroke1 = Stroke::default();
        let stroke2 = Stroke::default();
        
        let strokes_len = |layer: &Layer| {
            if let LayerContent::Strokes(strokes) = &layer.content {
                strokes.len()
            } else {
                0
            }
        };
        
        doc.execute_command(Command::AddStroke {
            layer_index: 0,
            stroke: stroke1,
        });
        doc.undo();
        
        doc.execute_command(Command::AddStroke {
            layer_index: 0,
            stroke: stroke2,
        });
        
        doc.redo();
        assert_eq!(strokes_len(&doc.layers[0]), 1);
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
        
        let strokes_len = |layer: &Layer| {
            if let LayerContent::Strokes(strokes) = &layer.content {
                strokes.len()
            } else {
                0
            }
        };
        
        doc.execute_command(Command::AddStroke {
            layer_index: 0,
            stroke: stroke.clone(),
        });
        
        assert_eq!(doc.undo_stack.len(), 1);
        assert_eq!(doc.redo_stack.len(), 0);
        assert_eq!(strokes_len(&doc.layers[0]), 1);
    }
}