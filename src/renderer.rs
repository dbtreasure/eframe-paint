// src/renderer.rs
use eframe::egui;
use eframe::glow::HasContext; // For OpenGL context

pub struct Renderer {
    gl: Option<std::sync::Arc<eframe::glow::Context>>,
}

impl Renderer {
    /// Creates a new renderer instance with GPU resources initialized
    /// 
    /// Args:
    ///     cc (CreationContext): The eframe creation context containing GPU context
    ///
    /// Returns:
    ///     Self: Initialized renderer instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Get the glow graphics context from eframe
        let gl = cc.gl.clone();
        
        // Initialize renderer with OpenGL context
        Self { gl }
    }

    /// Renders the current frame
    ///
    /// Args:
    ///     ctx (egui::Context): The egui context for the current frame
    ///     painter (egui::Painter): The painter to draw with
    ///     rect (egui::Rect): The rectangle to draw in
    pub fn render(&mut self, ctx: &egui::Context, painter: &egui::Painter, rect: egui::Rect) {
        // Draw a semi-transparent blue rectangle
        painter.rect_filled(
            rect,
            0.0, // rounding
            egui::Color32::from_rgba_premultiplied(0, 127, 255, 200), // semi-transparent blue
        );
        
        // Request continuous rendering
        ctx.request_repaint();
    }
}