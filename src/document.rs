// src/document.rs
use serde::{Serialize, Deserialize};
use crate::layer::{Layer, LayerContent};
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
        self.execute_command(command, &egui::Context::default());
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

    pub fn execute_command(&mut self, command: Command, ctx: &egui::Context) {
        match &command {
            Command::AddStroke { layer_index, stroke } => {
                if let Some(layer) = self.layers.get_mut(*layer_index) {
                    if let LayerContent::Strokes(strokes) = &mut layer.content {
                        strokes.push(stroke.clone());
                        layer.update_gpu_texture(ctx);
                    }
                }
            }
            Command::AddImageLayer { name, texture, size } => {
                if let Some(texture) = texture {
                    let mut layer = Layer::new_image(name, texture.clone(), *size);
                    layer.update_gpu_texture(ctx);
                    self.layers.push(layer);
                    self.active_layer = Some(self.layers.len() - 1);
                }
            }
            Command::AddLayer { name } => {
                let mut layer = Layer::new(name);
                layer.update_gpu_texture(ctx);
                self.layers.push(layer);
                self.active_layer = Some(self.layers.len() - 1);
            }
        }
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, ctx: &egui::Context) {
        if let Some(cmd) = self.undo_stack.pop() {
            match &cmd {
                Command::AddStroke { layer_index, .. } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        if let LayerContent::Strokes(strokes) = &mut layer.content {
                            strokes.pop();
                            layer.update_gpu_texture(ctx);
                        }
                    }
                }
                Command::AddImageLayer { .. } | Command::AddLayer { .. } => {
                    self.layers.pop();
                    self.active_layer = (!self.layers.is_empty()).then_some(self.layers.len() - 1);
                }
            }
            self.redo_stack.push(cmd);
        }
    }

    pub fn redo(&mut self, ctx: &egui::Context) {
        if let Some(cmd) = self.redo_stack.pop() {
            match &cmd {
                Command::AddStroke { layer_index, stroke } => {
                    if let Some(layer) = self.layers.get_mut(*layer_index) {
                        if let LayerContent::Strokes(strokes) = &mut layer.content {
                            strokes.push(stroke.clone());
                            layer.update_gpu_texture(ctx);
                        }
                    }
                }
                Command::AddImageLayer { name, texture, size } => {
                    if let Some(texture) = texture {
                        let mut layer = Layer::new_image(name, texture.clone(), *size);
                        layer.update_gpu_texture(ctx);
                        self.layers.push(layer);
                        self.active_layer = Some(self.layers.len() - 1);
                    }
                }
                Command::AddLayer { name } => {
                    let mut layer = Layer::new(name);
                    layer.update_gpu_texture(ctx);
                    self.layers.push(layer);
                    self.active_layer = Some(self.layers.len() - 1);
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
        };
        self.execute_command(command, &egui::Context::default());
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
