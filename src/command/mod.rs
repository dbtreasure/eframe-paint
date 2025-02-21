mod commands;
mod context;
mod history;

use crate::state::context::StateTransitionError;

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
    StateTransitionError(StateTransitionError),
}

impl From<StateTransitionError> for CommandError {
    fn from(error: StateTransitionError) -> Self {
        CommandError::StateTransitionError(error)
    }
}