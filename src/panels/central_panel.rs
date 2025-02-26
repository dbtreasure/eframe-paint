use crate::PaintApp;
use crate::command::{Command, CommandHistory};
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::stroke::Stroke;
use crate::input::{InputEvent, PanelKind};
use egui;

pub struct CentralPanel {
}

impl CentralPanel {
    pub fn new() -> Self {
        Self {
            // No more rect field
        }
    }
    
    // Removed rect() and set_rect() methods
    
    /// Handle input events specific to the central panel
    pub fn handle_input_event(
        &self,
        event: &InputEvent,
        state: &mut EditorState,
        document: &mut Document,
        command_history: &mut CommandHistory,
        renderer: &mut Renderer,
        panel_rect: egui::Rect, // Added panel_rect parameter
    ) {
        // Only process events that are in this panel
        if !self.is_event_in_panel(event, panel_rect) { // Pass panel_rect to this method
            return;
        }
        
        match event {
            // Process pointer down events
            InputEvent::PointerDown { location: _, button } 
                if *button == egui::PointerButton::Primary => {
                // Start a new stroke
                let stroke = Stroke::new(
                    egui::Color32::BLACK,
                    2.0,
                );
                *state = EditorState::start_drawing(stroke);
            }
            
            // Process pointer move events
            InputEvent::PointerMove { location, held_buttons } => {
                // Add point to stroke if we're drawing
                if held_buttons.contains(&egui::PointerButton::Primary) {
                    if let EditorState::Drawing { current_stroke } = state {
                        current_stroke.add_point(location.position);
                        // Update preview
                        renderer.set_preview_stroke(Some(current_stroke.clone()));
                    }
                }
            }
            
            // Process pointer up events
            InputEvent::PointerUp { location: _, button } 
                if *button == egui::PointerButton::Primary => {
                // Finish the stroke and add it to command history
                if let Some(stroke) = state.take_stroke() {
                    command_history.execute(Command::AddStroke(stroke), document);
                }
                renderer.set_preview_stroke(None);
            }
            
            // Ignore all other events
            _ => {}
        }
    }
    
    /// Check if an event is in this panel
    fn is_event_in_panel(&self, event: &InputEvent, panel_rect: egui::Rect) -> bool {
        match event {
            InputEvent::PointerDown { location, .. } |
            InputEvent::PointerUp { location, .. } |
            InputEvent::PointerMove { location, .. } |
            InputEvent::PointerEnter { location } => {
                // Check if the event position is within the panel rectangle
                panel_rect.contains(location.position)
            },
            InputEvent::PointerLeave { last_known_location } => {
                // For leave events, check the last known position
                panel_rect.contains(last_known_location.position)
            },
        }
    }
}

// For backward compatibility, provide a function that creates and shows a central panel
pub fn central_panel(app: &mut PaintApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Get the available rect for the central panel
        // This will automatically account for the space taken by the side panel
        let canvas_rect = ui.available_rect_before_wrap();
        
        // Set the central panel rect in the app
        app.set_central_panel_rect(canvas_rect);
        
        // No longer need to update the central panel's rect
        
        let painter = ui.painter();
        let renderer = app.renderer();
        renderer.render(ctx, painter, canvas_rect, app.document());
    });
}