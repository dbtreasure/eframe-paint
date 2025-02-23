use std::cell::RefCell;
use crate::event::{EditorEvent, EventHandler};

/// A simple event bus for broadcasting editor events to registered handlers
pub struct EventBus {
    handlers: RefCell<Vec<Box<dyn EventHandler>>>,
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        // When cloning, create a new empty event bus
        Self::new()
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("handlers", &format!("<{} handlers>", self.handlers.borrow().len()))
            .finish()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Creates a new event bus
    pub fn new() -> Self {
        Self {
            handlers: RefCell::new(Vec::new()),
        }
    }

    /// Subscribe a handler to receive events
    pub fn subscribe(&self, handler: Box<dyn EventHandler>) {
        self.handlers.borrow_mut().push(handler);
    }

    /// Emit an event to all registered handlers
    pub fn emit(&self, event: EditorEvent) {
        for handler in &mut *self.handlers.borrow_mut() {
            handler.handle_event(&event);
        }
    }
} 