use egui::{Pos2, Ui};
use crate::command::Command;
use crate::document::Document;
use crate::tools::Tool;
use crate::renderer::Renderer;
use crate::state::ElementType;

#[derive(Clone)]
pub struct SelectionTool {
    // No state needed for basic selection tool
}

impl SelectionTool {
    pub fn new() -> Self {
        Self {}
    }

    // Helper function to find element at position
    fn find_element_at_position(&self, pos: Pos2, doc: &Document) -> Option<ElementType> {
        // First check strokes
        for stroke in doc.strokes() {
            let points = stroke.points();
            if points.len() < 2 {
                continue;
            }
            
            for window in points.windows(2) {
                let line_start = window[0];
                let line_end = window[1];
                
                // Calculate distance from point to line segment
                let distance = self.distance_to_line_segment(pos, line_start, line_end);
                
                // If the distance is less than the stroke thickness plus a small margin, consider it a hit
                if distance <= stroke.thickness() + 2.0 {
                    return Some(ElementType::Stroke(stroke.clone()));
                }
            }
        }
        
        // Then check images
        for image in doc.images() {
            let rect = image.rect();
            if rect.contains(pos) {
                return Some(ElementType::Image(image.clone()));
            }
        }
        
        None
    }
    
    // Helper function to calculate distance from a point to a line segment
    fn distance_to_line_segment(&self, point: Pos2, line_start: Pos2, line_end: Pos2) -> f32 {
        let line_vec = line_end - line_start;
        let point_vec = point - line_start;
        
        let line_len_sq = line_vec.x * line_vec.x + line_vec.y * line_vec.y;
        
        // If the line segment is actually a point
        if line_len_sq == 0.0 {
            return point_vec.length();
        }
        
        // Calculate projection of point_vec onto line_vec
        let t = (point_vec.x * line_vec.x + point_vec.y * line_vec.y) / line_len_sq;
        
        if t < 0.0 {
            // Closest point is line_start
            return point_vec.length();
        } else if t > 1.0 {
            // Closest point is line_end
            return (point - line_end).length();
        } else {
            // Closest point is on the line segment
            let closest = line_start + line_vec * t;
            return (point - closest).length();
        }
    }
}

impl Tool for SelectionTool {
    fn name(&self) -> &'static str {
        "Selection"
    }

    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        // We don't return a command, but the selection will be handled in the central panel
        // by checking the active tool type and calling find_element_at_position
        None
    }

    fn update_preview(&mut self, _renderer: &mut Renderer) {
        // No preview needed for selection tool
    }

    fn clear_preview(&mut self, _renderer: &mut Renderer) {
        // No preview to clear
    }

    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool");
        ui.separator();
        ui.label("Click on elements to select them.");
        ui.label("Selected elements will be highlighted with a red box.");
        
        None  // No immediate command from UI
    }
} 