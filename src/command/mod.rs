use crate::state::context::EditorError;

pub mod commands;
pub mod context;
pub mod history;

pub use commands::Command;
pub use context::CommandContext;

/// Result type for command operations
pub type CommandResult = Result<(), CommandError>;

/// Errors that can occur during command execution
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Invalid state transition")]
    InvalidStateTransition,
    #[error("Invalid parameters provided")]
    InvalidParameters,
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("State transition error: {0}")]
    StateTransitionError(#[from] EditorError),
    #[error("Invalid layer ID")]
    InvalidLayerId,
}