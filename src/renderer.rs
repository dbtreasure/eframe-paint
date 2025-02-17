// src/renderer.rs
use eframe::egui::{self, Color32, Slider};

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

    pub fn render(&mut self, ctx: &egui::Context, painter: &egui::Painter, rect: egui::Rect) {
        // Draw a rectangle using the current brush color and alpha
        painter.rect_filled(
            rect,
            0.0,
            self.brush_color, // Use the selected color
        );
        
        // Request continuous rendering
        ctx.request_repaint();
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

    pub fn create_texture(&self, image: egui::ColorImage, name: &str) -> egui::TextureHandle {
        self.ctx.load_texture(
            name,
            image,
            egui::TextureOptions::default()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer {
            initialized: true,
            current_tool: Tool::Brush,
            brush_color: Color32::BLUE,
            brush_thickness: 5.0,
            ctx: egui::Context::default(),
        };
        assert!(renderer.is_initialized());
    }

    #[test]
    fn test_render_basics() {
        let mut renderer = Renderer {
            initialized: true,
            current_tool: Tool::Brush,
            brush_color: Color32::BLUE,
            brush_thickness: 5.0,
            ctx: egui::Context::default(),
        };
        let ctx = egui::Context::default();
        let layer_id = egui::LayerId::background();
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        let painter = egui::Painter::new(ctx.clone(), layer_id, rect);
        
        renderer.render(&ctx, &painter, rect);
    }

    #[test]
    fn test_tool_selection() {
        let mut renderer = Renderer {
            initialized: true,
            current_tool: Tool::Brush,
            brush_color: Color32::BLUE,
            brush_thickness: 5.0,
            ctx: egui::Context::default(),
        };
        assert_eq!(renderer.current_tool(), Tool::Brush);
        
        renderer.set_current_tool(Tool::Eraser);
        assert_eq!(renderer.current_tool(), Tool::Eraser);
    }
}