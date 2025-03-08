use egui::{Pos2, Response, Ui, Vec2, CursorIcon, Rect};
use log;

/// Represents a corner of a selection box
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {    
    pub fn cursor_icon(&self) -> CursorIcon {
        match self {
            Corner::TopLeft => CursorIcon::ResizeNwSe,
            Corner::TopRight => CursorIcon::ResizeNeSw,
            Corner::BottomLeft => CursorIcon::ResizeNeSw,
            Corner::BottomRight => CursorIcon::ResizeNwSe,
        }
    }
}

/// A resize handle widget for interactive resizing of elements
pub struct ResizeHandle {
    element_id: usize,
    corner: Corner,
    position: Pos2,
    size: f32,
}

impl ResizeHandle {
    pub fn new(element_id: usize, corner: Corner, position: Pos2, size: f32) -> Self {
        Self {
            element_id,
            corner,
            position,
            size,
        }
    }

    /// Show the resize handle and return the response
    pub fn show(&self, ui: &mut Ui) -> Response {
        // Create a unique ID for this specific resize handle
        let id = ui.make_persistent_id(format!("resize_handle_{}_{:?}", self.element_id, self.corner));
        
        // Create a small invisible button at the handle position with slightly increased size
        // for better interactivity
        let rect = Rect::from_center_size(self.position, Vec2::new(self.size * 3.0, self.size * 3.0));
        let sense = egui::Sense::drag();
        let response = ui.interact(rect, id, sense);
        
        // Detailed logging of interaction state
        if response.dragged() {
            log::info!("Handle DRAGGED for element {}, corner {:?}, drag delta: {:?}", 
                       self.element_id, self.corner, response.drag_delta());
        }
        
        // Set the cursor based on the corner
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(self.corner.cursor_icon());
        }
        
        // Draw visual representation of handle - make it more visible
        if response.hovered() || response.dragged() {
            let painter = ui.painter();
            let visuals = ui.visuals().widgets.active;
            
            // Draw filled circle with border for better visibility
            painter.circle_filled(
                self.position,
                self.size * 1.5,
                if response.dragged() { egui::Color32::RED } else { visuals.bg_fill }
            );
            
            painter.circle_stroke(
                self.position,
                self.size * 1.5,
                egui::Stroke::new(2.0, egui::Color32::WHITE)
            );
        }
        
        response
    }
} 