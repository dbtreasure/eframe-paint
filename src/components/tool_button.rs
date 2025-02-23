use eframe::egui;
use crate::renderer::Tool;

pub struct ToolButton {
    pub tool: Tool,
    pub icon: &'static str,
    pub selected: bool,
}

impl ToolButton {
    pub fn new(tool: Tool, icon: &'static str, selected: bool) -> Self {
        Self {
            tool,
            icon,
            selected,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        let button_size = egui::vec2(32.0, 32.0);
        let (rect, mut response) = ui.allocate_exact_size(button_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let bg_color = if self.selected {
                egui::Color32::from_rgb(100, 181, 246) // Light blue when selected
            } else if response.hovered() {
                egui::Color32::from_gray(40) // Lighter gray on hover
            } else {
                egui::Color32::from_gray(30) // Dark gray by default
            };

            // Draw background
            ui.painter().rect_filled(rect, 4.0, bg_color);

            // Draw icon text centered
            let font_id = egui::FontId::proportional(24.0);
            let text_color = if self.selected {
                egui::Color32::BLACK
            } else {
                egui::Color32::WHITE
            };
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.icon,
                font_id,
                text_color,
            );

            // Draw border when selected
            if self.selected {
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(33, 150, 243)),
                );
            }
        }

        response
    }
} 