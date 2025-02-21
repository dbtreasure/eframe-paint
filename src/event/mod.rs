mod bus;
mod events;
mod handlers;

pub use bus::EventBus;
pub use events::{EditorEvent, LayerEvent, SelectionEvent, DocumentEvent};
pub use handlers::{ToolEventHandler, LayerEventHandler, UndoRedoEventHandler};

pub trait EventHandler: Send {
    fn handle_event(&mut self, event: &EditorEvent);
} 