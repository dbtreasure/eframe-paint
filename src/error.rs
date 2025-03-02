//! # Transition Error Handling
//! 
//! This module defines the error types and handling strategies for tool state transitions.
//! 
//! ## Error Types
//! 
//! | Error Type                 | Description                       | Recovery Strategy               |
//! |----------------------------|-----------------------------------|----------------------------------|
//! | `InvalidStateTransition`   | Incompatible state transition     | Rollback to previous state       |
//! | `ToolBusy`                 | Tool is performing an operation   | Wait for operation completion    |
//! | `MemorySafetyViolation`    | Transition violates memory safety | Emergency state reset            |
//! 
//! ## Error Handling Pattern
//! 
//! The recommended pattern for handling transition errors is:
//! 
//! ```rust
//! match tool.transition_to_new_state() {
//!     Ok(new_tool) => {
//!         // Transition succeeded, use new_tool
//!     },
//!     Err(original_tool) => {
//!         // Transition failed, original_tool is returned unchanged
//!         // Handle the error or retry with different parameters
//!     }
//! }
//! ```
//! 
//! ## Validation Before Transition
//! 
//! To avoid errors, validate transitions before attempting them:
//! 
//! ```rust
//! if tool.can_transition() {
//!     let result = tool.transition_to_new_state();
//!     // result is likely to succeed
//! } else {
//!     // Handle invalid transition case
//! }
//! ```
//! 
//! ## Error Context
//! 
//! Errors include context information to aid debugging:
//! 
//! - `InvalidStateTransition`: Source and target states
//! - `ToolBusy`: Reason why the tool is busy
//! - `MemorySafetyViolation`: No context (critical error)
//! 
//! ## Best Practices
//! 
//! 1. Always check `can_transition()` before attempting transitions
//! 2. Handle all error cases explicitly
//! 3. Provide clear error messages to users
//! 4. Use the returned original tool to retry or recover

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