use egui::{Id, Pos2, Response, Ui, Vec2, CursorIcon, Rect, Color32, Stroke};

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
        let id = Id::new(("resize_handle", self.element_id, self.corner.as_str()));
        
        // Create a rect for the handle
        let rect = Rect::from_center_size(
            self.position,
            Vec2::splat(self.size),
        );
        
        // Draw the handle visual FIRST to ensure it's visible
        ui.painter().rect_filled(
            rect,
            4.0, // Rounded corners
            Color32::from_rgb(30, 120, 255), // Bright blue
        );
        
        // Add a border to make it more visible
        ui.painter().rect_stroke(
            rect,
            4.0, // Rounded corners
            Stroke::new(1.0, Color32::WHITE),
        );
        
        // Allocate space and check for interactions
        let response = ui.interact(rect, id, egui::Sense::click_and_drag())
            .on_hover_cursor(self.corner.cursor_icon());
        
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
    
    /// Draw a simple visual handle without interaction
    pub fn draw_simple_handle(ui: &mut Ui, position: Pos2, size: f32) {
        // Draw a filled circle for the handle
        ui.painter().circle_filled(
            position,
            size,
            Color32::from_rgb(30, 120, 255), // Bright blue
        );
        
        // Add a white border
        ui.painter().circle_stroke(
            position,
            size,
            Stroke::new(1.0, Color32::WHITE),
        );
    }
} 