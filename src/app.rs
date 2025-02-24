use eframe::egui;
use crate::renderer::Renderer;
use crate::document::Document;
use crate::state::EditorState;
use crate::command::{Command, CommandHistory};
use crate::stroke::Stroke;
use crate::panels::central_panel;
use crate::input::{InputHandler, InputEvent};

pub struct PaintApp {
    renderer: Renderer,
    document: Document,
    state: EditorState,
    command_history: CommandHistory,
    input_handler: InputHandler,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            renderer: Renderer::new(cc),
            document: Document::new(),
            state: EditorState::default(),
            command_history: CommandHistory::new(),
            input_handler: InputHandler::new(egui::Rect::NOTHING), // Will be updated in update()
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }

    pub fn command_history(&self) -> &CommandHistory {
        &self.command_history
    }

    pub fn command_history_mut(&mut self) -> &mut CommandHistory {
        &mut self.command_history
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn handle_input(&mut self, ctx: &egui::Context, canvas_rect: egui::Rect) {
        self.input_handler.set_canvas_rect(canvas_rect);
        let events = self.input_handler.process_input(ctx);

        for event in events {
            match event {
                InputEvent::PointerDown { location, button } if button == egui::PointerButton::Primary && location.is_in_canvas => {
                    // Start a new stroke
                    let stroke = Stroke::new(
                        egui::Color32::BLACK,
                        2.0,
                    );
                    self.state = EditorState::start_drawing(stroke);
                }
                InputEvent::PointerMove { location, held_buttons } if location.is_in_canvas => {
                    // Add point to stroke if we're drawing
                    if held_buttons.contains(&egui::PointerButton::Primary) {
                        if let EditorState::Drawing { current_stroke } = &mut self.state {
                            current_stroke.add_point(location.position);
                            // Update preview
                            self.renderer.set_preview_stroke(Some(current_stroke.clone()));
                        }
                    }
                }
                InputEvent::PointerUp { location, button } if button == egui::PointerButton::Primary => {
                    // Finish the stroke and add it to command history
                    if let Some(stroke) = self.state.take_stroke() {
                        self.command_history.execute(Command::AddStroke(stroke.clone()));
                        self.document.add_stroke(stroke);
                    }
                    self.renderer.set_preview_stroke(None);
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        central_panel(self, ctx);
    }
}