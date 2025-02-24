// src/renderer.rs
use eframe::egui;
use eframe::glow::HasContext; // For OpenGL context
use crate::stroke::Stroke;
use crate::document::Document;

pub struct Renderer {
    gl: Option<std::sync::Arc<eframe::glow::Context>>,
    preview_stroke: Option<Stroke>,
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
        Self {
            gl,
            preview_stroke: None,
        }
    }

    pub fn set_preview_stroke(&mut self, stroke: Option<Stroke>) {
        self.preview_stroke = stroke;
    }

    fn draw_stroke(&self, painter: &egui::Painter, stroke: &Stroke) {
        let points = stroke.points();
        if points.len() < 2 {
            return;
        }

        for points in points.windows(2) {
            painter.line_segment(
                [points[0], points[1]],
                egui::Stroke::new(stroke.thickness(), stroke.color()),
            );
        }
    }

    /// Renders the current frame
    ///
    /// Args:
    ///     ctx (egui::Context): The egui context for the current frame
    ///     painter (egui::Painter): The painter to draw with
    ///     rect (egui::Rect): The rectangle to draw in
    ///     document (Document): The document containing strokes to draw
    pub fn render(
        &self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        rect: egui::Rect,
        document: &Document,
    ) {
        // Draw background
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::WHITE,
        );

        // Draw all strokes in the document
        for stroke in document.strokes() {
            self.draw_stroke(painter, stroke);
        }

        // Draw preview stroke if any
        if let Some(preview) = &self.preview_stroke {
            self.draw_stroke(painter, preview);
        }

        // Request continuous rendering while we have a preview stroke
        if self.preview_stroke.is_some() {
            ctx.request_repaint();
        }
    }
}