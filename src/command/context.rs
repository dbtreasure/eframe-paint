use crate::document::Document;
use crate::event::EventBus;
use crate::state::EditorContext;
use crate::tool::ToolType;

/// Context for command execution, providing access to the document,
/// editor state, and event system.
#[derive(Debug)]
pub struct CommandContext {
    /// The document being edited
    pub document: Document,
    /// The editor context for state management
    pub editor_context: EditorContext,
    /// The event bus for broadcasting changes
    pub event_bus: EventBus,
    /// The current tool
    pub current_tool: ToolType,
}

impl CommandContext {
    /// Create a new command context
    pub fn new(
        document: Document,
        editor_context: EditorContext,
        event_bus: EventBus,
        initial_tool: ToolType,
    ) -> Self {
        Self {
            document,
            editor_context,
            event_bus,
            current_tool: initial_tool,
        }
    }
} 