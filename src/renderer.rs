// src/renderer.rs
use eframe::egui::{self, Color32, Slider};
use egui::{ColorImage, TextureHandle, TextureOptions};
use crate::stroke::Stroke;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tool {
    Brush,
    Eraser,
    Selection,
}

#[derive(Debug)]
pub struct Renderer {
    // We'll add fields here as needed for future rendering features
    initialized: bool,
    // Add new fields for tool state
    current_tool: Tool,
    brush_color: Color32,
    brush_thickness: f32,
    ctx: egui::Context,
}

impl Renderer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            initialized: true,
            current_tool: Tool::Brush,
            brush_color: Color32::BLUE,
            brush_thickness: 5.0,
            ctx: cc.egui_ctx.clone(),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn render_tools_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tools");
        ui.separator();

        // Tool selection buttons
        ui.horizontal(|ui| {
            if ui.selectable_label(self.current_tool == Tool::Brush, "🖌 Brush").clicked() {
                self.current_tool = Tool::Brush;
            }
            if ui.selectable_label(self.current_tool == Tool::Eraser, "⌫ Eraser").clicked() {
                self.current_tool = Tool::Eraser;
            }
            if ui.selectable_label(self.current_tool == Tool::Selection, "◻ Selection").clicked() {
                self.current_tool = Tool::Selection;
            }
        });

        ui.separator();

        // Color picker
        ui.horizontal(|ui| {
            ui.label("Color:");
            egui::color_picker::color_edit_button_srgba(
                ui,
                &mut self.brush_color,
                egui::color_picker::Alpha::Opaque
            );
        });

        // Brush thickness slider
        ui.horizontal(|ui| {
            ui.label("Thickness:");
            ui.add(Slider::new(&mut self.brush_thickness, 1.0..=50.0));
        });
    }

    // Add getters and setters for the new state
    pub fn current_tool(&self) -> Tool {
        self.current_tool
    }

    pub fn set_current_tool(&mut self, tool: Tool) {
        self.current_tool = tool;
    }

    pub fn brush_color(&self) -> Color32 {
        self.brush_color
    }

    pub fn set_brush_color(&mut self, color: Color32) {
        self.brush_color = color;
    }

    pub fn brush_thickness(&self) -> f32 {
        self.brush_thickness
    }

    pub fn set_brush_thickness(&mut self, thickness: f32) {
        self.brush_thickness = thickness;
    }

    pub fn create_texture(&self, image: ColorImage, name: &str) -> TextureHandle {
        self.ctx.load_texture(
            name,
            image,
            TextureOptions::default()
        )
    }

    // Add method to create texture from image data
    pub fn create_texture_from_image(&self, image: ColorImage, name: &str) -> TextureHandle {
        self.ctx.load_texture(
            name,
            image,
            TextureOptions::default()
        )
    }

    // Add method to render strokes to texture
    pub fn render_strokes_to_texture(
        &self,
        strokes: &[Stroke],
        size: [usize; 2],
        name: &str
    ) -> TextureHandle {
        let mut image = ColorImage::new(size, Color32::TRANSPARENT);
        
        // TODO: Implement stroke rasterization
        // This is where you'll convert vector strokes into raster image
        
        self.create_texture_from_image(image, name)
    }
}
