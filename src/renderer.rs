// src/renderer.rs
use eframe::egui;
use crate::stroke::{Stroke, StrokeRef};
use crate::document::Document;
use crate::image::Image;

pub struct Renderer {
    _gl: Option<std::sync::Arc<eframe::glow::Context>>,
    preview_stroke: Option<StrokeRef>,
}

impl Renderer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone();
        
        Self {
            _gl: gl,
            preview_stroke: None,
        }
    }

    pub fn set_preview_stroke(&mut self, stroke: Option<StrokeRef>) {
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

    fn draw_image(&self, ctx: &egui::Context, painter: &egui::Painter, image: &Image) {
        // Use the image's unique ID for caching instead of memory address
        let image_id = image.id();
        
        // Create a new texture from the image data every time
        let width = image.size().x as usize;
        let height = image.size().y as usize;
        
        // Create the color image from RGBA data
        let color_image = if image.data().len() == width * height * 4 {
            // Data is already in RGBA format
            egui::ColorImage::from_rgba_unmultiplied(
                [width, height],
                image.data(),
            )
        } else {
            // If data is not in the expected format, create a placeholder
            egui::ColorImage::new([width, height], egui::Color32::RED)
        };
        
        // Load the texture (this will be automatically freed at the end of the frame)
        let texture = ctx.load_texture(
            format!("image_{}", image_id),
            color_image,
            egui::TextureOptions::default(),
        );
        
        let texture_id = texture.id();
        
        // Draw the image at its position with its size
        let rect = image.rect();
        
        // Use the full texture (UV coordinates from 0,0 to 1,1)
        let uv = egui::Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 1.0)
        );
        
        painter.image(texture_id, rect, uv, egui::Color32::WHITE);
    }

    pub fn render(
        &mut self,
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

        // Draw all images in the document
        for (_i, image_ref) in document.images().iter().enumerate() {
            self.draw_image(ctx, painter, image_ref);
        }

        // Draw all strokes in the document
        for stroke_ref in document.strokes() {
            self.draw_stroke(painter, stroke_ref);
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