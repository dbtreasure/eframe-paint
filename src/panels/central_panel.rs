use crate::PaintApp;
use crate::command::{Command, CommandHistory};
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::stroke::MutableStroke;
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
            InputEvent::PointerDown { location: _, button } 
                if *button == egui::PointerButton::Primary => {
                let stroke = MutableStroke::new(
                    egui::Color32::BLACK,
                    2.0,
                );
                *state = EditorState::start_drawing(stroke);
            }
            
            InputEvent::PointerMove { location, held_buttons } => {
                if held_buttons.contains(&egui::PointerButton::Primary) {
                    if let EditorState::Drawing { current_stroke } = state {
                        current_stroke.add_point(location.position);
                        
                        // Create a StrokeRef for preview without cloning the points twice
                        let preview = current_stroke.to_stroke_ref();
                        renderer.set_preview_stroke(Some(preview));
                    }
                }
            }
            
            InputEvent::PointerUp { location: _, button } 
                if *button == egui::PointerButton::Primary => {
                if let Some(stroke) = state.take_stroke() {
                    command_history.execute(Command::AddStroke(stroke), document);
                }
                renderer.set_preview_stroke(None);
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
        let renderer = app.renderer();
        renderer.render(ctx, painter, canvas_rect, app.document());
    });
}