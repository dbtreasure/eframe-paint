use crate::tool::ToolType;
use crate::document::Document;
use crate::command::CommandHistory;
use super::{EventHandler, EditorEvent, LayerEvent};

/// Handles tool-related events and maintains the current tool state
pub struct ToolEventHandler {
    current_tool: ToolType,
}

impl ToolEventHandler {
    pub fn new(initial_tool: ToolType) -> Self {
        Self {
            current_tool: initial_tool,
        }
    }

    pub fn current_tool(&self) -> &ToolType {
        &self.current_tool
    }
}

impl EventHandler for ToolEventHandler {
    fn handle_event(&mut self, event: &EditorEvent) {
        if let EditorEvent::ToolChanged { old: _, new } = event {
            self.current_tool = new.clone();
        }
    }
}

/// Handles layer-related events and updates the document accordingly
pub struct LayerEventHandler {
    document: Document,
}

impl LayerEventHandler {
    pub fn new(document: Document) -> Self {
        Self { document }
    }
}

impl EventHandler for LayerEventHandler {
    fn handle_event(&mut self, event: &EditorEvent) {
        if let EditorEvent::LayerChanged(layer_event) = event {
            match layer_event {
                LayerEvent::Added { index } => {
                    // Handle layer addition
                    self.document.handle_layer_added(*index);
                },
                LayerEvent::Removed { index } => {
                    // Handle layer removal
                    self.document.handle_layer_removed(*index);
                },
                LayerEvent::Reordered { from, to } => {
                    // Handle layer reordering
                    self.document.handle_layer_reordered(*from, *to);
                },
                LayerEvent::Transformed { index, old_transform, new_transform } => {
                    // Handle layer transformation
                    self.document.handle_layer_transformed(*index, *old_transform, *new_transform);
                },
                LayerEvent::VisibilityChanged { index, visible } => {
                    // Handle visibility change
                    self.document.handle_layer_visibility_changed(*index, *visible);
                },
            }
        }
    }
}

/// Handles events that should be recorded in the undo/redo history
pub struct UndoRedoEventHandler {
    history: CommandHistory,
}

impl UndoRedoEventHandler {
    pub fn new(history: CommandHistory) -> Self {
        Self { history }
    }
}

impl EventHandler for UndoRedoEventHandler {
    fn handle_event(&mut self, event: &EditorEvent) {
        match event {
            EditorEvent::LayerChanged(_) |
            EditorEvent::SelectionChanged(_) |
            EditorEvent::DocumentChanged(_) => {
                // Record the event in the command history for undo/redo
                self.history.record_event(event.clone());
            }
            _ => {} // Other events don't need to be recorded
        }
    }
} 