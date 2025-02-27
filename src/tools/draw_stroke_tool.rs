use egui::{Pos2, Ui};
use crate::stroke::MutableStroke;
use crate::command::Command;
use crate::document::Document;
use crate::tools::Tool;
use crate::renderer::Renderer;
use std::any::Any;

pub struct DrawStrokeTool {
    // Transient state: the stroke being drawn (if any)
    current_stroke: Option<MutableStroke>,
}

impl DrawStrokeTool {
    pub fn new() -> Self {
        Self { current_stroke: None }
    }
}

impl Tool for DrawStrokeTool {
    fn name(&self) -> &'static str {
        "DrawStroke"
    }

    fn activate(&mut self, _doc: &Document) {
        // Reset any in-progress stroke when activated
        self.current_stroke = None;
    }
    
    fn deactivate(&mut self, _doc: &Document) {
        // Clear any in-progress stroke when deactivated
        self.current_stroke = None;
    }

    fn on_pointer_down(&mut self, pos: Pos2, _doc: &Document) -> Option<Command> {
        // Start a new stroke at the cursor position
        let mut stroke = MutableStroke::new(egui::Color32::BLACK, 2.0);
        stroke.add_point(pos);
        self.current_stroke = Some(stroke);
        None  // No command yet (not finalized)
    }

    fn on_pointer_move(&mut self, pos: Pos2, _doc: &Document) -> Option<Command> {
        // Continue the stroke if one is in progress
        if let Some(stroke) = &mut self.current_stroke {
            stroke.add_point(pos);
            // No command returned yet, as we're still drawing
        }
        None
    }

    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // Finish the stroke and produce an AddStroke command for undo/redo
        if let Some(stroke) = self.current_stroke.take() {
            if !stroke.points().is_empty() {
                // Create a command that, when executed, adds the stroke to the document
                let stroke_ref = stroke.to_stroke_ref();
                return Some(Command::AddStroke(stroke_ref));
            }
        }
        None
    }

    fn update_preview(&mut self, renderer: &mut Renderer) {
        if let Some(stroke) = &self.current_stroke {
            let preview = stroke.to_stroke_ref();
            renderer.set_preview_stroke(Some(preview));
        } else {
            renderer.set_preview_stroke(None);
        }
    }

    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_preview_stroke(None);
    }

    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        // The drawing tool might display options like brush size or color.
        // For simplicity, assume these are handled elsewhere (e.g., a color picker tool),
        // so this tool has no extra UI controls.
        ui.label("Use the mouse to draw on the canvas.");
        None  // No immediate command from UI
    }
} 