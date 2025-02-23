// src/document.rs
use serde::{Serialize, Deserialize};
use crate::layer::{Layer, LayerId, Transform};
use crate::command::{Command, CommandContext, CommandResult, CommandError};
use crate::command::history::CommandHistory;
use crate::selection::Selection;
use crate::state::{EditorContext, context::FeedbackLevel};
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
    /// Command history for undo/redo
    #[serde(skip)]
    pub history: CommandHistory,
    /// Current feedback message
    #[serde(skip)]
    feedback: Option<(String, FeedbackLevel)>,
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.layers == other.layers &&
        self.active_layer == other.active_layer &&
        self.current_selection == other.current_selection
    }
}

impl Document {
    /// Creates a new empty document
    pub fn new() -> Self {
        let mut doc = Self {
            layers: Vec::new(),
            active_layer: None,
            current_selection: None,
            history: CommandHistory::new(),
            feedback: None,
        };
        
        // Create a default layer
        doc.add_layer("Background");
        
        doc
    }

    /// Get the current feedback message
    pub fn current_feedback(&self) -> Option<(&str, FeedbackLevel)> {
        self.feedback.as_ref().map(|(msg, level)| (msg.as_str(), *level))
    }

    /// Set a feedback message
    pub fn set_feedback(&mut self, message: impl Into<String>, level: FeedbackLevel) {
        self.feedback = Some((message.into(), level));
    }

    /// Clear the current feedback message
    pub fn clear_feedback(&mut self) {
        self.feedback = None;
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

    /// Adds a new image layer with the given name and texture
    pub fn add_image_layer(&mut self, name: &str, texture: TextureHandle, size: [usize; 2]) {
        let layer = Layer::new_image(name, texture, size);
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

    /// Execute a command on the document
    pub fn execute_command(&mut self, command: Command) -> CommandResult {
        // Take ownership of history temporarily to avoid multiple mutable borrows
        let mut history = std::mem::take(&mut self.history);
        
        // Create required components
        let mut editor_context = EditorContext::new(self.clone(), Renderer::default());
        let event_bus = EventBus::default();
        
        // Create a context for command execution
        let mut ctx = CommandContext::new(
            self,
            &mut editor_context,
            &event_bus,
            ToolType::Brush(crate::tool::BrushTool::default()),
            &mut history,
        );

        // Execute the command
        let result = command.validate(&ctx)
            .and_then(|_| command.execute(&mut ctx));

        // If the command can be undone and execution was successful, add it to history
        if command.can_undo() && result.is_ok() {
            history.push(command);
        }

        // Restore history
        self.history = history;

        result
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

    /// Handles a layer's content being changed
    pub fn handle_layer_content_changed(&mut self, index: usize) {
        // For now, we just emit an event that the document was modified
        // In the future, this could trigger thumbnail regeneration, etc.
        if let Some(layer) = self.layers.get_mut(index) {
            // Mark the layer as needing thumbnail update
            layer.needs_thumbnail_update = true;
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}