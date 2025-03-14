use egui::Ui;
use egui::Pos2;
use crate::command::Command;
use crate::renderer::Renderer;
use crate::state::EditorModel;
use std::any::Any;

/// Tool configuration trait for persisting tool settings
pub trait ToolConfig: Send + Sync + 'static {
    /// Get the tool name this config belongs to
    fn tool_name(&self) -> &'static str;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Tool trait defines the interface for all drawing tools
pub trait Tool: Send + Sync {
    /// Return the name of the tool
    fn name(&self) -> &'static str;
    
    /// Return current selection state if applicable
    fn selection_state(&self) -> Option<&SelectionState> {
        None
    }
    
    /// Called when the tool is selected (activated).
    /// Can be used to initialize or reset tool state.
    fn activate(&mut self, _editor_model: &EditorModel) {
        // default: do nothing
    }
    
    /// Called when the tool is deselected (deactivated).
    /// Can be used to clean up state or finalize preview.
    fn deactivate(&mut self, editor_model: &EditorModel);

    /// If true, this tool operates on a selected element and should be disabled if there is no selection.
    fn requires_selection(&self) -> bool {
        false  // default: tool does not need a selection
    }

    /// Handle pointer press (e.g., mouse down) on the canvas.
    /// Return a Command to **begin** an action if applicable, or None.
    fn on_pointer_down(&mut self, pos: Pos2, editor_model: &EditorModel) -> Option<Command>;

    /// Handle pointer drag (movement) while the pointer is held down.
    /// Can update internal state or preview, and optionally return a Command for continuous actions.
    fn on_pointer_move(&mut self, pos: Pos2, editor_model: &mut EditorModel, ui: &egui::Ui, renderer: &mut Renderer) -> Option<Command>;

    /// Handle pointer release (e.g., mouse up) on the canvas.
    /// Return a Command to **finalize** an action if applicable.
    fn on_pointer_up(&mut self, pos: Pos2, editor_model: &EditorModel) -> Option<Command>;

    /// Update any preview rendering for the tool's current state
    fn update_preview(&mut self, renderer: &mut Renderer);

    /// Clear any preview rendering
    fn clear_preview(&mut self, renderer: &mut Renderer);

    /// Show any tool-specific UI controls (buttons, sliders, etc.) in the tool panel.
    /// Return a Command if the UI interaction should trigger an action.
    fn ui(&mut self, ui: &mut Ui, editor_model: &EditorModel) -> Option<Command>;
    
    /// Get the current configuration of this tool
    fn get_config(&self) -> Box<dyn ToolConfig>;
    
    /// Apply a configuration to this tool
    fn apply_config(&mut self, _config: &dyn ToolConfig);
}


// Tool implementations
mod draw_stroke_tool;
pub use draw_stroke_tool::{UnifiedDrawStrokeTool, DrawStrokeState, new_draw_stroke_tool};

mod selection_tool;
pub use selection_tool::{UnifiedSelectionTool, SelectionState, new_selection_tool};

// Re-export any tool implementations we add later
// Example: mod pencil_tool; pub use pencil_tool::PencilTool; 

/// Enum representing all available tool types
/// This allows us to avoid using Box<dyn Tool> and simplifies memory management
#[derive(Clone)]
pub enum ToolType {
    DrawStroke(UnifiedDrawStrokeTool),
    Selection(UnifiedSelectionTool),
    // Add more tools here as they are implemented
}

impl Tool for ToolType {
    fn name(&self) -> &'static str {
        match self {
            Self::DrawStroke(tool) => tool.name(),
            Self::Selection(tool) => tool.name(),
        }
    }
    
    fn selection_state(&self) -> Option<&SelectionState> {
        match self {
            Self::Selection(tool) => tool.selection_state(),
            _ => None,
        }
    }
    
    fn activate(&mut self, editor_model: &EditorModel) {
        match self {
            Self::DrawStroke(tool) => tool.activate(editor_model),
            Self::Selection(tool) => tool.activate(editor_model),
        }
    }
    
    fn deactivate(&mut self, editor_model: &EditorModel) {
        match self {
            Self::DrawStroke(tool) => tool.deactivate(editor_model),
            Self::Selection(tool) => tool.deactivate(editor_model),
        }
    }
    
    fn requires_selection(&self) -> bool {
        match self {
            Self::DrawStroke(tool) => tool.requires_selection(),
            Self::Selection(tool) => tool.requires_selection(),
        }
    }
    
    fn on_pointer_down(&mut self, pos: Pos2, editor_model: &EditorModel) -> Option<Command> {
        match self {
            Self::DrawStroke(tool) => tool.on_pointer_down(pos, editor_model),
            Self::Selection(tool) => tool.on_pointer_down(pos, editor_model),
        }
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, editor_model: &mut EditorModel, ui: &egui::Ui, renderer: &mut Renderer) -> Option<Command> {
        match self {
            Self::DrawStroke(tool) => tool.on_pointer_move(pos, editor_model, ui, renderer),
            Self::Selection(tool) => tool.on_pointer_move(pos, editor_model, ui, renderer),
        }
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, editor_model: &EditorModel) -> Option<Command> {
        match self {
            Self::DrawStroke(tool) => tool.on_pointer_up(pos, editor_model),
            Self::Selection(tool) => tool.on_pointer_up(pos, editor_model),
        }
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::DrawStroke(tool) => tool.update_preview(renderer),
            Self::Selection(tool) => tool.update_preview(renderer),
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::DrawStroke(tool) => tool.clear_preview(renderer),
            Self::Selection(tool) => tool.clear_preview(renderer),
        }
    }
    
    fn ui(&mut self, ui: &mut Ui, editor_model: &EditorModel) -> Option<Command> {
        match self {
            Self::DrawStroke(tool) => tool.ui(ui, editor_model),
            Self::Selection(tool) => tool.ui(ui, editor_model),
        }
    }
    
    fn get_config(&self) -> Box<dyn ToolConfig> {
        match self {
            Self::DrawStroke(tool) => tool.get_config(),
            Self::Selection(tool) => tool.get_config(),
        }
    }
    
    fn apply_config(&mut self, config: &dyn ToolConfig) {
        match self {
            Self::DrawStroke(tool) => {
                tool.apply_config(config);
            },
            Self::Selection(tool) => tool.apply_config(config),
        }
    }
}

// Factory function to create a new tool of the specified type
pub fn new_tool(tool_type: &str) -> Option<ToolType> {
    match tool_type {
        "DrawStroke" => Some(ToolType::DrawStroke(new_draw_stroke_tool())),
        "Selection" => Some(ToolType::Selection(new_selection_tool())),
        _ => None,
    }
}

// Helper methods for ToolType
impl ToolType {    
    pub fn current_state_name(&self) -> &'static str {
        match self {
            Self::DrawStroke(tool) => tool.current_state_name(),
            Self::Selection(tool) => tool.current_state_name(),
        }
    }
}