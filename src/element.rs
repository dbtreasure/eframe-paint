use egui;
use std::sync::Arc;
use crate::image::Image;
use crate::stroke::Stroke;
use crate::stroke::StrokeRef;
use crate::image::ImageRef;
pub const RESIZE_HANDLE_RADIUS: f32 = 15.0;
pub const STROKE_BASE_PADDING: f32 = 10.0;
pub const IMAGE_PADDING: f32 = 10.0;
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

#[derive(Clone, Debug)]
pub enum ElementType {
    Stroke(StrokeRef),
    Image(ImageRef),
}

impl ElementType {
    pub fn get_stable_id(&self) -> usize {
        match self {
            ElementType::Stroke(stroke_ref) => stroke_ref.id(),
            ElementType::Image(image_ref) => image_ref.id(),
        }
    }
    
    /// Get the element type as a string
    pub fn element_type_str(&self) -> &'static str {
        match self {
            ElementType::Stroke(_) => "stroke",
            ElementType::Image(_) => "image",
        }
    }
    
    /// Get the stroke reference if this is a stroke
    pub fn as_stroke(&self) -> Option<&StrokeRef> {
        match self {
            ElementType::Stroke(stroke) => Some(stroke),
            _ => None,
        }
    }
    
    /// Get the image reference if this is an image
    pub fn as_image(&self) -> Option<&ImageRef> {
        match self {
            ElementType::Image(image) => Some(image),
            _ => None,
        }
    }
}

impl Element for ElementType {
    fn id(&self) -> usize {
        self.get_stable_id()
    }
    
    fn element_type(&self) -> &'static str {
        self.element_type_str()
    }
    
    fn rect(&self) -> egui::Rect {
        match self {
            ElementType::Image(img) => {
                egui::Rect::from_min_size(
                    img.position(),
                    img.size()
                )
            },
            ElementType::Stroke(stroke) => {
                stroke.rect()
            }
        }
    }
    
    fn as_element_type(&self) -> ElementType {
        self.clone()
    }
}

// New enum for mutable element references
#[derive(Debug)]
pub enum ElementTypeMut<'a> {
    Stroke(&'a mut StrokeRef),
    Image(&'a mut ImageRef),
}

impl<'a> ElementTypeMut<'a> {
    // Add method to translate element in-place
    pub fn translate(&mut self, delta: egui::Vec2) -> Result<(), String> {
        match self {
            ElementTypeMut::Stroke(stroke) => {
                if let Some(stroke_mut) = Arc::get_mut(stroke) {
                    stroke_mut.translate_in_place(delta);
                    Ok(())
                } else {
                    Err("Could not get mutable reference to stroke".to_string())
                }
            },
            ElementTypeMut::Image(image) => {
                if let Some(image_mut) = Arc::get_mut(image) {
                    image_mut.translate_in_place(delta);
                    Ok(())
                } else {
                    Err("Could not get mutable reference to image".to_string())
                }
            }
        }
    }
    
    // Add method to resize element in-place
    pub fn resize(&mut self, original_rect: egui::Rect, new_rect: egui::Rect) -> Result<(), String> {
        match self {
            ElementTypeMut::Stroke(stroke) => {
                if let Some(stroke_mut) = Arc::get_mut(stroke) {
                    stroke_mut.resize_in_place(original_rect, new_rect);
                    Ok(())
                } else {
                    Err("Could not get mutable reference to stroke".to_string())
                }
            },
            ElementTypeMut::Image(image) => {
                if let Some(image_mut) = Arc::get_mut(image) {
                    image_mut.resize_in_place(new_rect)
                } else {
                    Err("Could not get mutable reference to image".to_string())
                }
            }
        }
    }

    // Add method to get the element ID
    pub fn id(&self) -> usize {
        match self {
            ElementTypeMut::Stroke(stroke) => stroke.id(),
            ElementTypeMut::Image(image) => image.id(),
        }
    }
}

pub fn compute_element_rect(element: &ElementType) -> egui::Rect {
    // Get the base rectangle from the Element trait
    let base_rect = element.rect();
    
    // Apply padding based on element type
    match element {
        ElementType::Stroke(_stroke) => {
            // For strokes, add the base padding
            let padding = STROKE_BASE_PADDING;
            
            egui::Rect::from_min_max(
                egui::pos2(base_rect.min.x - padding, base_rect.min.y - padding),
                egui::pos2(base_rect.max.x + padding, base_rect.max.y + padding),
            )
        }
        ElementType::Image(_) => {
            // For images, add the image padding
            egui::Rect::from_min_max(
                egui::pos2(base_rect.min.x - IMAGE_PADDING, base_rect.min.y - IMAGE_PADDING),
                egui::pos2(base_rect.max.x + IMAGE_PADDING, base_rect.max.y + IMAGE_PADDING),
            )
        }
    }
}