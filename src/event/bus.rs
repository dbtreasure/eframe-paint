use std::sync::{Arc, RwLock};
use super::{EventHandler, EditorEvent};

/// Event bus for broadcasting events to subscribers
#[derive(Clone)]
pub struct EventBus {
    subscribers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("subscribers", &"<event handlers>")
            .finish()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn subscribe(&mut self, handler: Box<dyn EventHandler>) {
        if let Ok(mut subscribers) = self.subscribers.write() {
            subscribers.push(handler);
        }
    }

    pub fn emit(&self, event: EditorEvent) {
        if let Ok(mut subscribers) = self.subscribers.write() {
            for handler in subscribers.iter_mut() {
                handler.handle_event(&event);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
} 