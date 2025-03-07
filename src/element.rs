use egui;
use std::sync::Arc;
use crate::image::Image;
use crate::stroke::Stroke;
use crate::state::ElementType;

/// Common trait for all element types in the document
pub trait Element {
    /// Get the unique identifier for this element
    fn id(&self) -> usize;
    
    /// Get the element type as a string
    fn element_type(&self) -> &'static str;
    
    /// Get the bounding rectangle for this element
    fn rect(&self) -> egui::Rect;
    
    /// Convert this element to an ElementType enum
    fn as_element_type(&self) -> ElementType;
}

impl Element for Image {
    fn id(&self) -> usize { 
        self.id() 
    }
    
    fn element_type(&self) -> &'static str { 
        "image" 
    }
    
    fn rect(&self) -> egui::Rect { 
        egui::Rect::from_min_size(
            self.position(),
            self.size()
        ) 
    }
    
    fn as_element_type(&self) -> ElementType {
        ElementType::Image(Arc::new(self.clone()))
    }
}

impl Element for Stroke {
    fn id(&self) -> usize { 
        self.id()
    }
    
    fn element_type(&self) -> &'static str { 
        "stroke" 
    }
    
    fn rect(&self) -> egui::Rect { 
        // Calculate bounding rect from points
        let points = self.points();
        if points.is_empty() {
            return egui::Rect::NOTHING;
        }

        // Calculate the bounding box of all points
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for point in points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        // Expand by stroke thickness/2 to account for the actual visible stroke area
        let padding = self.thickness() / 2.0;
        
        egui::Rect::from_min_max(
            egui::pos2(min_x - padding, min_y - padding),
            egui::pos2(max_x + padding, max_y + padding),
        )
    }
    
    fn as_element_type(&self) -> ElementType {
        ElementType::Stroke(Arc::new(self.clone()))
    }
} 