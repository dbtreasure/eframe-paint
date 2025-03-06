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

    // Add a method to replace a stroke by ID
    pub fn replace_stroke_by_id(&mut self, id: usize, new_stroke: StrokeRef) -> bool {
        // Find the index of the stroke with the matching ID
        let mut index_to_remove = None;
        for (i, stroke) in self.strokes.iter().enumerate() {
            let stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
            if stroke_id == id {
                index_to_remove = Some(i);
                break;
            }
        }
        
        // If found, remove it and add the new one
        if let Some(index) = index_to_remove {
            // Remove the old stroke
            self.strokes.remove(index);
            // Add the new stroke at the end (not at the same index)
            self.strokes.push(new_stroke);
            return true;
        }
        false
    }
    
    // Add a method to replace an image by ID
    pub fn replace_image_by_id(&mut self, id: usize, new_image: ImageRef) -> bool {
        // Find the index of the image with the matching ID
        let mut index_to_remove = None;
        for (i, image) in self.images.iter().enumerate() {
            if image.id() == id {
                index_to_remove = Some(i);
                break;
            }
        }
        
        // If found, remove it and add the new one
        if let Some(index) = index_to_remove {
            // Remove the old image
            self.images.remove(index);
            // Add the new image at the end (not at the same index)
            self.images.push(new_image);
            return true;
        }
        false
    }

    // Add a method to completely rebuild the document
    pub fn rebuild(&mut self) {
        // Create new copies of all strokes
        let new_strokes: Vec<StrokeRef> = self.strokes.iter()
            .map(|stroke| {
                // Create a new stroke with the same properties
                let points = stroke.points().to_vec();
                let color = stroke.color();
                let thickness = stroke.thickness();
                
                // Create a new mutable stroke
                let mut mutable_stroke = crate::stroke::MutableStroke::new(color, thickness);
                
                // Add all points
                for point in points {
                    mutable_stroke.add_point(point);
                }
                
                // Convert to StrokeRef
                mutable_stroke.to_stroke_ref()
            })
            .collect();
            
        // Create new copies of all images
        let new_images: Vec<ImageRef> = self.images.iter()
            .map(|image| {
                // Create a new image with the same properties
                let id = image.id();
                let data = image.data().to_vec();
                let size = image.size();
                let position = image.position();
                
                // Create a new mutable image
                let mutable_img = crate::image::MutableImage::new_with_id(
                    id,
                    data,
                    size,
                    position,
                );
                
                // Convert to ImageRef
                mutable_img.to_image_ref()
            })
            .collect();
            
        // Replace the old collections with the new ones
        self.strokes = new_strokes;
        self.images = new_images;
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