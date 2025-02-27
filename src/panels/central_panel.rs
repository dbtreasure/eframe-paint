use crate::PaintApp;
use crate::command::{Command, CommandHistory};
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::stroke::MutableStroke;
use crate::input::InputEvent;
use crate::tools::{DrawStrokeTool, Tool};
use egui;
use std::cell::RefCell;

thread_local! {
    static DRAW_TOOL: RefCell<DrawStrokeTool> = RefCell::new(DrawStrokeTool::new());
}

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
                // Use the DrawStrokeTool to handle the pointer down event
                let mut cmd = None;
                DRAW_TOOL.with(|tool| {
                    cmd = tool.borrow_mut().on_pointer_down(location.position, document);
                });
                
                if let Some(cmd) = cmd {
                    command_history.execute(cmd, document);
                } else {
                    // If no command was returned, start drawing using the existing state system
                    let stroke = MutableStroke::new(
                        egui::Color32::BLACK,
                        2.0,
                    );
                    *state = EditorState::start_drawing(stroke);
                }
            }
            
            InputEvent::PointerMove { location, held_buttons } => {
                if held_buttons.contains(&egui::PointerButton::Primary) {
                    // First try to use the DrawStrokeTool
                    let mut cmd = None;
                    DRAW_TOOL.with(|tool| {
                        cmd = tool.borrow_mut().on_pointer_move(location.position, document);
                    });
                    
                    if let Some(cmd) = cmd {
                        command_history.execute(cmd, document);
                    } else {
                        // Fall back to the existing state system
                        if let EditorState::Drawing { current_stroke } = state {
                            current_stroke.add_point(location.position);
                            
                            // Create a StrokeRef for preview without cloning the points twice
                            let preview = current_stroke.to_stroke_ref();
                            renderer.set_preview_stroke(Some(preview));
                        }
                    }
                }
            }
            
            InputEvent::PointerUp { location, button } 
                if *button == egui::PointerButton::Primary => {
                // First try to use the DrawStrokeTool
                let mut cmd = None;
                DRAW_TOOL.with(|tool| {
                    cmd = tool.borrow_mut().on_pointer_up(location.position, document);
                });
                
                if let Some(cmd) = cmd {
                    command_history.execute(cmd, document);
                } else {
                    // Fall back to the existing state system
                    if let Some(stroke) = state.take_stroke() {
                        command_history.execute(Command::AddStroke(stroke), document);
                    }
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