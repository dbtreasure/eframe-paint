use egui::Ui;
use egui::Pos2;
use crate::command::Command;
use crate::document::Document;

pub trait Tool {
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
    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        None  // default: no action on pointer down
    }

    /// Handle pointer drag (movement) while the pointer is held down.
    /// Can update internal state or preview, and optionally return a Command for continuous actions.
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        None  // default: no action on pointer move (just update state/preview)
    }

    /// Handle pointer release (e.g., mouse up) on the canvas.
    /// Return a Command to **finalize** an action if applicable.
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        None  // default: no action on pointer up
    }

    /// Show any tool-specific UI controls (buttons, sliders, etc.) in the tool panel.
    /// This is also where instant tools can trigger their action.
    /// If an action is taken via the UI (e.g., button click or slider change), return the corresponding Command.
    fn ui(&mut self, ui: &mut Ui, doc: &Document) -> Option<Command>;
}

// Tool implementations
mod draw_stroke_tool;
pub use draw_stroke_tool::DrawStrokeTool;

// Re-export any tool implementations we add later
// Example: mod pencil_tool; pub use pencil_tool::PencilTool; 