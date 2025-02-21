use crate::document::Document;
use crate::renderer::Renderer;
use crate::event::EventBus;
use super::EditorState;

pub struct EditorContext {
    pub state: EditorState,
    pub document: Document,
    pub renderer: Renderer,
    pub event_bus: EventBus,
}

impl EditorContext {
    pub fn new(document: Document, renderer: Renderer) -> Self {
        Self {
            state: EditorState::Idle,
            document,
            renderer,
            event_bus: EventBus::new(),
        }
    }
} 