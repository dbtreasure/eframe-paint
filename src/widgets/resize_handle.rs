use egui::{Id, Pos2, Response, Ui, Vec2, CursorIcon, Rect, Color32, Stroke, Rounding};
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
        // Create a unique ID for this specific resize handle
        let id = ui.make_persistent_id(format!("resize_handle_{}_{:?}", self.element_id, self.corner));
        
        log::debug!("Showing resize handle for element {} corner {:?}", self.element_id, self.corner);
        
        // Create a small invisible button at the handle position
        let rect = Rect::from_center_size(self.position, Vec2::new(self.size * 2.0, self.size * 2.0));
        let response = ui.interact(rect, id, egui::Sense::drag());
        
        // Set the cursor based on the corner
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(self.corner.cursor_icon());
        }
        
        // Draw visual representation of handle if hovered or dragged
        if response.hovered() || response.dragged() {
            let painter = ui.painter();
            let visuals = ui.visuals().widgets.active;
            
            painter.rect_filled(
                rect,
                0.0,
                visuals.bg_fill,
            );
            
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, visuals.fg_stroke.color),
            );
        }
        
        // Log detailed drag events to help debug the issue
        if response.drag_started() {
            log::info!("DRAG STARTED for handle of element {} corner {:?}", self.element_id, self.corner);
        }
        
        if response.dragged() {
            log::info!("DRAGGING handle of element {} corner {:?}, delta: {:?}", 
                self.element_id, self.corner, response.drag_delta());
        }
        
        if response.drag_released() {
            log::info!("DRAG RELEASED for handle of element {} corner {:?}", self.element_id, self.corner);
        }
        
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