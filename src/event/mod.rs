mod bus;
mod events;

pub use bus::EventBus;
pub use events::*;

pub trait EventHandler: Send {
    fn handle_event(&mut self, event: &EditorEvent);
}

// Re-export the event types
pub use events::EditorEvent;
pub use events::LayerEvent;
pub use events::SelectionEvent;
pub use events::DocumentEvent; 