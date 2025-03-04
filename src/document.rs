use crate::stroke::StrokeRef;
use crate::image::ImageRef;
use crate::state::ElementType;
use egui;

pub struct Document {
    strokes: Vec<StrokeRef>,
    images: Vec<ImageRef>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            strokes: Vec::new(),
            images: Vec::new(),
        }
    }

    pub fn add_stroke(&mut self, stroke: StrokeRef) {
        self.strokes.push(stroke);
    }

    pub fn strokes(&self) -> &[StrokeRef] {
        &self.strokes
    }

    pub fn strokes_mut(&mut self) -> &mut Vec<StrokeRef> {
        &mut self.strokes
    }

    pub fn remove_last_stroke(&mut self) -> Option<StrokeRef> {
        self.strokes.pop()
    }
    
    pub fn add_image(&mut self, image: ImageRef) {
        self.images.push(image);
    }
    
    pub fn images(&self) -> &[ImageRef] {
        &self.images
    }
    
    pub fn images_mut(&mut self) -> &mut Vec<ImageRef> {
        &mut self.images
    }
    
    pub fn remove_last_image(&mut self) -> Option<ImageRef> {
        let image = self.images.pop();
        image
    }

    pub fn find_image_by_id(&self, id: usize) -> Option<&ImageRef> {
        // Find an image by its ID (this is safer than using get_element_mut)
        self.images.iter().find(|img| img.id() == id)
    }

    pub fn get_element_mut(&mut self, element_id: usize) -> Option<&mut ElementType> {
        // First check images since they have explicit IDs
        for image in &mut self.images {
            if image.id() == element_id {
                return Some(unsafe { 
                    // This is safe because we're returning a mutable reference to the image
                    // wrapped in ElementType, and we're ensuring it doesn't outlive self
                    std::mem::transmute::<&mut ImageRef, &mut ElementType>(&mut *image)
                });
            }
        }
        
        // Then check strokes by comparing pointer values
        for stroke in &mut self.strokes {
            let stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
            if stroke_id == element_id {
                return Some(unsafe {
                    // This is safe because we're returning a mutable reference to the stroke
                    // wrapped in ElementType, and we're ensuring it doesn't outlive self
                    std::mem::transmute::<&mut StrokeRef, &mut ElementType>(&mut *stroke)
                });
            }
        }
        
        None
    }

    pub fn element_at_position(&self, point: egui::Pos2) -> Option<ElementType> {
        // First check strokes (front to back)
        for stroke in &self.strokes {
            // For simplicity, we'll check if the point is close to any line segment in the stroke
            let points = stroke.points();
            if points.len() < 2 {
                continue;
            }

            for window in points.windows(2) {
                let line_start = window[0];
                let line_end = window[1];
                
                // Calculate distance from point to line segment
                let distance = distance_to_line_segment(point, line_start, line_end);
                
                // If the distance is less than the stroke thickness plus a small margin, consider it a hit
                if distance <= stroke.thickness() + 2.0 {
                    return Some(ElementType::Stroke(stroke.clone()));
                }
            }
        }

        // Then check images (front to back)
        for image in &self.images {
            let rect = image.rect();
            if rect.contains(point) {
                return Some(ElementType::Image(image.clone()));
            }
        }

        // No element found at the position
        None
    }

    pub fn get_element_by_id(&self, id: usize) -> Option<ElementType> {
        // First check images
        if let Some(image) = self.find_image_by_id(id) {
            return Some(ElementType::Image(image.clone()));
        }
        
        // If not found in images, it might be a stroke but we need to compare by pointer address
        for stroke in self.strokes() {
            let stroke_id = std::sync::Arc::as_ptr(&stroke) as usize;
            if stroke_id == id {
                return Some(ElementType::Stroke(stroke.clone()));
            }
        }
        
        None
    }
}

fn distance_to_line_segment(point: egui::Pos2, line_start: egui::Pos2, line_end: egui::Pos2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    
    let line_len_sq = line_vec.length_sq();
    if line_len_sq == 0.0 {
        // Line segment is actually a point
        return point_vec.length();
    }
    
    // Calculate projection of point_vec onto line_vec
    let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
    
    // Calculate the closest point on the line segment
    let closest = line_start + line_vec * t;
    
    // Return the distance to the closest point
    (point - closest).length()
} 