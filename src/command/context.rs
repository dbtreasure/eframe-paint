use crate::document::Document;
use crate::event::EventBus;
use crate::state::EditorContext;
use crate::tool::ToolType;

/// Context for command execution, providing access to the document,
/// editor state, and event system.
#[derive(Debug)]
pub struct CommandContext<'a> {
    /// The document being edited
    pub document: &'a mut Document,
    /// The editor context for state management
    pub editor_context: &'a mut EditorContext,
    /// The event bus for broadcasting changes
    pub event_bus: &'a mut EventBus,
    /// The current tool
    pub current_tool: ToolType,
}

impl<'a> CommandContext<'a> {
    /// Create a new command context
    pub fn new(
        document: &'a mut Document,
        editor_context: &'a mut EditorContext,
        event_bus: &'a mut EventBus,
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