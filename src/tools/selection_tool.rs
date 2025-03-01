use egui::{Pos2, Ui};
use crate::command::Command;
use crate::document::Document;
use crate::tools::Tool;
use crate::renderer::Renderer;

#[derive(Clone)]
pub struct SelectionTool {
    // No state needed for basic selection tool
}

impl SelectionTool {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Returns the current state name as a string
    pub fn current_state_name(&self) -> &'static str {
        "Active" // Selection tool has only one state
    }
    
    /// Returns true if the tool is in the Ready state
    pub fn is_ready(&self) -> bool {
        true // Selection tool is always ready
    }
    
    /// Returns true if the tool is in the Drawing state
    pub fn is_drawing(&self) -> bool {
        false // Selection tool doesn't have a drawing state
    }
}

impl Tool for SelectionTool {
    fn name(&self) -> &'static str {
        "Selection"
    }

    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // We don't return a command, but the selection will be handled in the central panel
        // by checking the active tool type and calling find_element_at_position
        None
    }

    fn update_preview(&mut self, _renderer: &mut Renderer) {
        // No preview needed for selection tool
    }

    fn clear_preview(&mut self, _renderer: &mut Renderer) {
        // No preview to clear
    }

    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool");
        ui.separator();
        ui.label("Click on elements to select them.");
        ui.label("Selected elements will be highlighted with a red box.");
        
        None  // No immediate command from UI
    }
} 