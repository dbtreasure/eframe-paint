use super::{Command, CommandContext, CommandResult, CommandError};
use crate::event::{EditorEvent, LayerEvent, SelectionEvent};
use crate::layer::LayerId;

/// Manages the history of executed commands for undo/redo functionality
pub struct CommandHistory {
    /// Stack of commands that can be undone
    undo_stack: Vec<Command>,
    /// Stack of commands that can be redone
    redo_stack: Vec<Command>,
}

impl CommandHistory {
    /// Creates a new empty command history
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Execute a command and add it to the history if successful
    pub fn execute(&mut self, command: Command, ctx: &mut CommandContext) -> CommandResult {
        // Try to execute the command
        command.execute(ctx)?;

        // If execution was successful and command is undoable, add to history
        if command.can_undo() {
            self.undo_stack.push(command);
            self.redo_stack.clear(); // Clear redo stack when new command is executed
        }

        Ok(())
    }

    /// Undo the last executed command
    pub fn undo(&mut self, ctx: &mut CommandContext) -> CommandResult {
        if let Some(command) = self.undo_stack.pop() {
            if let Some(inverse) = command.inverse(ctx) {
                // Execute the inverse command
                inverse.execute(ctx)?;
                // Add the original command to redo stack
                self.redo_stack.push(command);
                return Ok(());
            }
        }
        Err(CommandError::InvalidState)
    }

    /// Redo the last undone command
    pub fn redo(&mut self, ctx: &mut CommandContext) -> CommandResult {
        if let Some(command) = self.redo_stack.pop() {
            // Re-execute the command
            command.execute(ctx)?;
            // Add it back to the undo stack
            self.undo_stack.push(command);
            return Ok(());
        }
        Err(CommandError::InvalidState)
    }

    /// Record an event in the history
    pub fn record_event(&mut self, event: EditorEvent) {
        // Convert event to command if appropriate
        if let Some(command) = Self::event_to_command(&event) {
            if command.can_undo() {
                self.undo_stack.push(command);
                self.redo_stack.clear();
            }
        }
    }

    /// Convert an event to a command if possible
    fn event_to_command(event: &EditorEvent) -> Option<Command> {
        match event {
            EditorEvent::ToolChanged { old: _, new } => {
                Some(Command::SetTool(new.clone()))
            }
            EditorEvent::LayerChanged(layer_event) => {
                // Convert layer events to appropriate commands
                match layer_event {
                    LayerEvent::Transformed { index, old_transform: _, new_transform } => {
                        Some(Command::TransformLayer {
                            layer_id: LayerId::new(*index),
                            transform: new_transform.clone(),
                        })
                    }
                    _ => None, // Other layer events not yet supported
                }
            }
            EditorEvent::SelectionChanged(selection_event) => {
                match selection_event {
                    SelectionEvent::Modified(selection) => {
                        Some(Command::SetSelection {
                            selection: selection.clone(),
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Returns true if there are commands that can be undone
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns true if there are commands that can be redone
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear the command history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
} 