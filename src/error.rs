use std::fmt;

/// Errors that can occur during tool state transitions
#[derive(Debug)]
pub enum TransitionError {
    /// Attempted to transition between incompatible states
    InvalidStateTransition {
        from: &'static str,
        to: &'static str,
        state: String
    },
    /// Tool is busy and cannot transition
    ToolBusy(String),
    /// Attempted operation would violate memory safety
    MemorySafetyViolation
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStateTransition { from, to, state } => 
                write!(f, "Cannot transition from {}({}) to {}", from, state, to),
            Self::ToolBusy(reason) => 
                write!(f, "Tool busy: {}", reason),
            Self::MemorySafetyViolation => 
                write!(f, "Memory safety violation during transition")
        }
    }
}

impl std::error::Error for TransitionError {} 