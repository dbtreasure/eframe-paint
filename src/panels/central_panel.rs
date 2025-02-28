use crate::PaintApp;
use crate::command::CommandHistory;
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::input::InputEvent;
use egui;

pub struct CentralPanel {
}

impl CentralPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_input_event(
        &self,
        event: &InputEvent,
        state: &mut EditorState,
        document: &mut Document,
        command_history: &mut CommandHistory,
        renderer: &mut Renderer,
        panel_rect: egui::Rect,
    ) {
        if !self.is_event_in_panel(event, panel_rect) {
            return;
        }
        
        match event {
            InputEvent::PointerDown { location, button } 
                if *button == egui::PointerButton::Primary => {
                // Use the active tool to handle the pointer down event
                if let Some(tool) = state.active_tool_mut() {
                    if let Some(cmd) = tool.on_pointer_down(location.position, document) {
                        command_history.execute(cmd, document);
                    }
                    
                    // Update preview using the tool's trait method
                    tool.update_preview(renderer);
                }
            }
            
            InputEvent::PointerMove { location, held_buttons } => {
                if held_buttons.contains(&egui::PointerButton::Primary) {
                    // Use the active tool to handle the pointer move event
                    if let Some(tool) = state.active_tool_mut() {
                        if let Some(cmd) = tool.on_pointer_move(location.position, document) {
                            command_history.execute(cmd, document);
                        }
                        
                        // Update preview using the tool's trait method
                        tool.update_preview(renderer);
                    }
                }
            }
            
            InputEvent::PointerUp { location, button } 
                if *button == egui::PointerButton::Primary => {
                // Use the active tool to handle the pointer up event
                if let Some(tool) = state.active_tool_mut() {
                    if let Some(cmd) = tool.on_pointer_up(location.position, document) {
                        command_history.execute(cmd, document);
                    }
                    
                    // Clear preview using the tool's trait method
                    tool.clear_preview(renderer);
                }
            }
            
            _ => {}
        }
    }
    
    fn is_event_in_panel(&self, event: &InputEvent, panel_rect: egui::Rect) -> bool {
        match event {
            InputEvent::PointerDown { location, .. } |
            InputEvent::PointerUp { location, .. } |
            InputEvent::PointerMove { location, .. } |
            InputEvent::PointerEnter { location } => {
                panel_rect.contains(location.position)
            },
            InputEvent::PointerLeave { last_known_location } => {
                panel_rect.contains(last_known_location.position)
            },
        }
    }
}

pub fn central_panel(app: &mut PaintApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let canvas_rect = ui.available_rect_before_wrap();
        app.set_central_panel_rect(canvas_rect);
        
        let painter = ui.painter();
        
        // Render directly from the app, passing all needed components
        // This avoids borrowing conflicts by letting the app manage access to its components
        app.render(ctx, painter, canvas_rect);
    });
}