pub mod bus;
pub mod events;
pub mod handlers;

pub use bus::EventBus;
pub use events::EditorEvent;
pub use events::{LayerEvent, SelectionEvent, TransformEvent};

/// Trait for handling editor events
pub trait EventHandler: Send {
    fn handle_event(&mut self, event: &EditorEvent);
}

pub use handlers::UndoRedoEventHandler; 