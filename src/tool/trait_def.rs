use crate::state::EditorContext;
use eframe::egui;

pub struct InputState {
    pub pointer_pos: Option<egui::Pos2>,
    pub pointer_pressed: bool,
    pub pointer_released: bool,
    pub modifiers: egui::Modifiers,
}

pub trait Tool: Send {
    fn on_activate(&mut self, ctx: &mut EditorContext);
    fn on_deactivate(&mut self, ctx: &mut EditorContext);
    fn update(&mut self, ctx: &mut EditorContext, input: &InputState);
    fn render(&self, ctx: &EditorContext, painter: &egui::Painter);
} 