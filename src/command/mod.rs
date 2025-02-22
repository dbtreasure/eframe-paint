mod context;
pub mod commands;
mod history;

use crate::state::context::EditorError;
use std::fmt;
use crate::layer::LayerId;

pub use commands::Command;
pub use context::CommandContext;
pub use history::CommandHistory;

/// Result type for command operations
pub type CommandResult = Result<(), CommandError>;

/// Errors that can occur during command execution
#[derive(Debug)]
pub enum CommandError {
    /// The command cannot be executed in the current state
    InvalidState,
    /// The command parameters are invalid
    InvalidParameters,
    /// The command failed during execution
    ExecutionFailed(String),
    /// State transition error
    StateTransitionError(EditorError),
    /// Invalid layer ID
    InvalidLayerId,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::InvalidState => write!(f, "Invalid state for command"),
            CommandError::InvalidParameters => write!(f, "Invalid parameters for command"),
            CommandError::InvalidLayerId => write!(f, "Invalid layer ID"),
            CommandError::ExecutionFailed(message) => write!(f, "Execution failed: {}", message),
            CommandError::StateTransitionError(error) => write!(f, "State transition error: {}", error),
        }
    }
}

impl std::error::Error for CommandError {}

impl From<EditorError> for CommandError {
    fn from(error: EditorError) -> Self {
        CommandError::StateTransitionError(error)
    }
}