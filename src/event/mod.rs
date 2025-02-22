mod events;
mod bus;
mod handlers;

pub use events::{EditorEvent, LayerEvent, SelectionEvent, DocumentEvent, TransformEvent};
pub use bus::EventBus;
pub use handlers::{ToolEventHandler, LayerEventHandler};

pub trait EventHandler: Send {
    fn handle_event(&mut self, event: &EditorEvent);
} 