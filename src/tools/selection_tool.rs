use egui::{Pos2, Ui};
use crate::command::Command;
use crate::document::Document;
use crate::tools::Tool;
use crate::renderer::Renderer;

// State type definitions
#[derive(Clone)]
pub struct Active;

#[derive(Clone)]
pub struct SelectionTool<State = Active> {
    #[allow(dead_code)]
    state: State,
}

impl SelectionTool<Active> {
    pub fn new() -> Self {
        Self { state: Active }
    }
}

impl Tool for SelectionTool<Active> {
    fn name(&self) -> &'static str {
        "Selection"
    }

    fn deactivate(&mut self, _doc: &Document) {
        // When the selection tool is deactivated, we want to clear any selected elements
        // This will be handled in the app.rs file by modifying the state
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

impl Default for SelectionTool<Active> {
    fn default() -> Self {
        Self::new()
    }
}

// Wrapper enum to handle state transitions
#[derive(Clone)]
pub enum SelectionToolType {
    Active(SelectionTool<Active>),
}

impl Tool for SelectionToolType {
    fn name(&self) -> &'static str {
        match self {
            Self::Active(tool) => tool.name(),
        }
    }

    fn activate(&mut self, doc: &Document) {
        match self {
            Self::Active(tool) => tool.activate(doc),
        }
    }
    
    fn deactivate(&mut self, doc: &Document) {
        match self {
            Self::Active(tool) => tool.deactivate(doc),
        }
    }
    
    fn requires_selection(&self) -> bool {
        match self {
            Self::Active(tool) => tool.requires_selection(),
        }
    }

    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.on_pointer_down(pos, doc),
        }
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.on_pointer_move(pos, doc),
        }
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.on_pointer_up(pos, doc),
        }
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::Active(tool) => tool.update_preview(renderer),
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::Active(tool) => tool.clear_preview(renderer),
        }
    }
    
    fn ui(&mut self, ui: &mut Ui, doc: &Document) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.ui(ui, doc),
        }
    }
}

impl SelectionToolType {
    pub fn new() -> Self {
        SelectionToolType::Active(SelectionTool::new())
    }
    
    pub fn current_state_name(&self) -> &'static str {
        "Active"
    }
}

pub fn new_selection_tool() -> SelectionToolType {
    SelectionToolType::new()
} 