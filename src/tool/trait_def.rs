use crate::state::EditorContext;
use eframe::egui;

#[derive(Debug, Clone)]
pub struct InputState {
    pub pointer_pos: Option<egui::Pos2>,
    pub pointer_pressed: bool,
    pub pointer_released: bool,
    pub modifiers: egui::Modifiers,
    /// Pressure value between 0.0 and 1.0, or None if pressure is not supported
    pub pressure: Option<f32>,
}

pub trait Tool: Send {
    fn on_activate(&mut self, ctx: &mut EditorContext);
    fn on_deactivate(&mut self, ctx: &mut EditorContext);
    fn update(&mut self, ctx: &mut EditorContext, input: &InputState);
    fn render(&self, ctx: &EditorContext, painter: &egui::Painter);
} 