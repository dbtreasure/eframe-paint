use crate::event::{EditorEvent, EventHandler};
use crate::command::history::CommandHistory;

/// Handles events related to undo/redo functionality
#[derive(Debug)]
pub struct UndoRedoEventHandler {
    history: CommandHistory,
}

impl UndoRedoEventHandler {
    /// Creates a new undo/redo event handler
    pub fn new() -> Self {
        Self {
            history: CommandHistory::new(),
        }
    }

    /// Get a reference to the command history
    pub fn history(&self) -> &CommandHistory {
        &self.history
    }

    /// Get a mutable reference to the command history
    pub fn history_mut(&mut self) -> &mut CommandHistory {
        &mut self.history
    }
}

impl EventHandler for UndoRedoEventHandler {
    fn handle_event(&mut self, event: &EditorEvent) {
        // Record events that can be undone
        self.history.record_event(event.clone());
    }
} 