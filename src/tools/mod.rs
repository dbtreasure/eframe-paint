use egui::Ui;
use egui::Pos2;
use crate::command::Command;
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;

/// Tool trait defines the interface for all drawing tools
pub trait Tool: Send + Sync {
    /// Name or identifier for the tool (for UI display or debugging).
    fn name(&self) -> &'static str;
    
    /// Called when the tool is selected (activated).
    /// Can be used to initialize or reset tool state.
    fn activate(&mut self, _doc: &Document) {
        // default: do nothing
    }
    
    /// Called when the tool is deselected (deactivated).
    /// Can be used to clean up state or finalize preview.
    fn deactivate(&mut self, _doc: &Document) {
        // default: do nothing
    }

    /// If true, this tool operates on a selected element and should be disabled if there is no selection.
    fn requires_selection(&self) -> bool {
        false  // default: tool does not need a selection
    }

    /// Handle pointer press (e.g., mouse down) on the canvas.
    /// Return a Command to **begin** an action if applicable, or None.
    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        None  // default: no action on pointer down
    }

    /// Handle pointer drag (movement) while the pointer is held down.
    /// Can update internal state or preview, and optionally return a Command for continuous actions.
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        None  // default: no action on pointer move (just update state/preview)
    }

    /// Handle pointer release (e.g., mouse up) on the canvas.
    /// Return a Command to **finalize** an action if applicable.
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        None  // default: no action on pointer up
    }

    /// Update any preview rendering for the tool's current state
    fn update_preview(&mut self, _renderer: &mut Renderer) {
        // Default implementation does nothing
    }

    /// Clear any preview rendering
    fn clear_preview(&mut self, _renderer: &mut Renderer) {
        // Default implementation does nothing
    }

    /// Show any tool-specific UI controls (buttons, sliders, etc.) in the tool panel.
    /// This is also where instant tools can trigger their action.
    /// If an action is taken via the UI (e.g., button click or slider change), return the corresponding Command.
    fn ui(&mut self, ui: &mut Ui, doc: &Document) -> Option<Command>;
}

// Tool implementations
mod draw_stroke_tool;
pub use draw_stroke_tool::{DrawStrokeToolType, new_draw_stroke_tool};

mod selection_tool;
pub use selection_tool::{SelectionToolType, new_selection_tool};

// Re-export any tool implementations we add later
// Example: mod pencil_tool; pub use pencil_tool::PencilTool; 

/// Enum representing all available tool types
/// This allows us to avoid using Box<dyn Tool> and simplifies memory management
#[derive(Clone)]
pub enum ToolType {
    DrawStroke(DrawStrokeToolType),
    Selection(SelectionToolType),
    // Add more tools here as they are implemented
}

impl ToolType {
    /// Get the name of the tool
    pub fn name(&self) -> &'static str {
        match self {
            Self::DrawStroke(tool) => tool.name(),
            Self::Selection(tool) => tool.name(),
            // Add more tools here as they are implemented
        }
    }

    /// Create a new instance of this tool type
    pub fn new_instance(&self) -> Self {
        match self {
            Self::DrawStroke(_) => Self::DrawStroke(new_draw_stroke_tool()),
            Self::Selection(_) => Self::Selection(new_selection_tool()),
            // Add more tools here as they are implemented
        }
    }

    /// Activate the tool
    /// Takes ownership of self and returns ownership of a potentially modified tool.
    pub fn activate(self, doc: &Document) -> Self {
        match self {
            Self::DrawStroke(mut tool) => {
                tool.activate(doc);
                Self::DrawStroke(tool)
            },
            Self::Selection(mut tool) => {
                tool.activate(doc);
                Self::Selection(tool)
            },
            // Add more tools here as they are implemented
        }
    }

    /// Deactivate the tool
    /// Takes ownership of self and returns ownership of a potentially modified tool.
    pub fn deactivate(mut self, doc: &Document) -> Self {
        match &mut self {
            Self::DrawStroke(tool) => tool.deactivate(doc),
            Self::Selection(tool) => tool.deactivate(doc),
            // Add more tools here as they are implemented
        }
        self
    }

    /// Check if the tool requires a selection
    pub fn requires_selection(&self) -> bool {
        match self {
            Self::DrawStroke(tool) => tool.requires_selection(),
            Self::Selection(tool) => tool.requires_selection(),
            // Add more tools here as they are implemented
        }
    }

    /// Handle pointer down event
    pub fn on_pointer_down(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::DrawStroke(tool) => tool.on_pointer_down(pos, doc, state),
            Self::Selection(tool) => tool.on_pointer_down(pos, doc, state),
            // Add more tools here as they are implemented
        }
    }

    /// Handle pointer move event
    pub fn on_pointer_move(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::DrawStroke(tool) => tool.on_pointer_move(pos, doc, state),
            Self::Selection(tool) => tool.on_pointer_move(pos, doc, state),
            // Add more tools here as they are implemented
        }
    }

    /// Handle pointer up event
    pub fn on_pointer_up(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::DrawStroke(tool) => tool.on_pointer_up(pos, doc, state),
            Self::Selection(tool) => tool.on_pointer_up(pos, doc, state),
            // Add more tools here as they are implemented
        }
    }

    /// Update preview rendering
    pub fn update_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::DrawStroke(tool) => tool.update_preview(renderer),
            Self::Selection(tool) => tool.update_preview(renderer),
            // Add more tools here as they are implemented
        }
    }

    /// Clear preview rendering
    pub fn clear_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::DrawStroke(tool) => tool.clear_preview(renderer),
            Self::Selection(tool) => tool.clear_preview(renderer),
            // Add more tools here as they are implemented
        }
    }

    /// Show tool-specific UI
    pub fn ui(&mut self, ui: &mut Ui, doc: &Document) -> Option<Command> {
        match self {
            Self::DrawStroke(tool) => tool.ui(ui, doc),
            Self::Selection(tool) => tool.ui(ui, doc),
            // Add more tools here as they are implemented
        }
    }

    /// Check if this is a selection tool
    pub fn is_selection_tool(&self) -> bool {
        matches!(self, Self::Selection(_))
    }

    /// Returns the current state name of the tool
    pub fn current_state_name(&self) -> &'static str {
        match self {
            Self::DrawStroke(tool) => tool.current_state_name(),
            Self::Selection(tool) => tool.current_state_name(),
            // Add more tools here as they are implemented
        }
    }
    
    /// Returns true if the tool is in a state where it can be configured
    pub fn is_configurable(&self) -> bool {
        match self {
            Self::DrawStroke(tool) => matches!(tool, DrawStrokeToolType::Ready(_)),
            Self::Selection(_) => true, // Selection tool is always configurable
            // Add more tools here as they are implemented
        }
    }
    
    /// Returns true if the tool is actively drawing or performing an operation
    pub fn is_active_operation(&self) -> bool {
        match self {
            Self::DrawStroke(tool) => matches!(tool, DrawStrokeToolType::Drawing(_)),
            Self::Selection(_) => false, // Selection tool doesn't have an active operation state
            // Add more tools here as they are implemented
        }
    }
    
    /// Check if this tool can transition to another state
    pub fn can_transition(&self) -> bool {
        match self {
            Self::DrawStroke(tool) => tool.can_transition(),
            Self::Selection(_) => true, // Selection tool transitions are validated by the tool itself
            // Add more tools here as they are implemented
        }
    }
}

/// Tool pool for reusing tool instances
/// This helps optimize tool transitions by avoiding unnecessary allocations
#[derive(Default)]
pub struct ToolPool {
    selection_tool: Option<SelectionToolType>,
    draw_stroke_tool: Option<DrawStrokeToolType>,
}

impl ToolPool {
    /// Create a new empty tool pool
    pub fn new() -> Self {
        Self {
            selection_tool: None,
            draw_stroke_tool: None,
        }
    }

    /// Get a tool from the pool by name
    /// If the tool is in the pool, it will be removed and returned
    /// Returns None if no matching tool is found
    pub fn get(&mut self, tool_name: &str) -> Option<ToolType> {
        match tool_name {
            "Selection" => self.selection_tool.take().map(ToolType::Selection),
            "Draw Stroke" => self.draw_stroke_tool.take().map(ToolType::DrawStroke),
            _ => None
        }
    }
    
    /// Return a tool to the pool
    /// The tool will be stored for future reuse
    pub fn return_tool(&mut self, tool: ToolType) {
        match tool {
            ToolType::Selection(s) => self.selection_tool = Some(s),
            ToolType::DrawStroke(d) => self.draw_stroke_tool = Some(d),
        }
    }

    /// Check if a transition to the given tool state is valid
    pub fn can_transition(&self, tool: &ToolType) -> bool {
        tool.can_transition()
    }
} 