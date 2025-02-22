mod editor_state;
pub mod context;
mod persistence;

pub use editor_state::EditorState;
pub use context::EditorContext;
pub use persistence::{
    StatePersistence,
    EditorSnapshot,
    PersistenceError,
    PersistenceResult,
};

// Re-export any other types that should be public from this module 