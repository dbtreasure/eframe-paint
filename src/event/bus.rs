use super::{EventHandler, EditorEvent};
use std::sync::Arc;
use parking_lot::RwLock;

/// Event bus for broadcasting events to subscribers
pub struct EventBus {
    subscribers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn subscribe(&self, handler: Box<dyn EventHandler>) {
        self.subscribers.write().push(handler);
    }

    pub fn emit(&self, event: EditorEvent) {
        let mut subscribers = self.subscribers.write();
        for subscriber in subscribers.iter_mut() {
            subscriber.handle_event(&event);
        }
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("subscriber_count", &self.subscribers.read().len())
            .finish()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
} 