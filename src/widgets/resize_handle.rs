use egui::{Id, Pos2, Response, Ui, Vec2, CursorIcon, Rect};

/// Represents a corner of a selection box
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub fn as_str(&self) -> &'static str {
        match self {
            Corner::TopLeft => "top_left",
            Corner::TopRight => "top_right",
            Corner::BottomLeft => "bottom_left",
            Corner::BottomRight => "bottom_right",
        }
    }
    
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
        // Create a unique ID for this handle
        let _id = Id::new(("resize_handle", self.element_id, self.corner.as_str()));
        
        // Create a rect for the handle
        let rect = Rect::from_center_size(
            self.position,
            Vec2::splat(self.size),
        );
        
        // Allocate space and check for interactions
        let response = ui.allocate_rect(rect, egui::Sense::drag())
            .on_hover_cursor(self.corner.cursor_icon());
            
        // Draw the handle
        if response.hovered() || response.dragged() {
            ui.painter().rect_filled(
                rect,
                0.0, // no rounding
                ui.style().visuals.selection.bg_fill,
            );
        } else {
            ui.painter().rect_filled(
                rect,
                0.0, // no rounding
                ui.style().visuals.widgets.active.bg_fill,
            );
        }
        
        // Add a border
        ui.painter().rect_stroke(
            rect,
            0.0, // no rounding
            egui::Stroke::new(1.0, ui.style().visuals.widgets.active.fg_stroke.color),
        );
        
        response
    }
    
    /// Get the corner this handle represents
    pub fn corner(&self) -> Corner {
        self.corner
    }
    
    /// Get the element ID this handle is associated with
    pub fn element_id(&self) -> usize {
        self.element_id
    }
} 