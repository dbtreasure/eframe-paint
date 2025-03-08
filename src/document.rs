use crate::stroke::StrokeRef;
use crate::image::ImageRef;
use crate::state::ElementType;
use egui;
use std::sync::Arc;

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

pub struct Document {
    strokes: Vec<StrokeRef>,
    images: Vec<ImageRef>,
    version: u64,
}

impl Document {
    pub fn new() -> Self {
        Self {
            strokes: Vec::new(),
            images: Vec::new(),
            version: 0,
        }
    }

    // Add a method to mark the document as modified and increment the version
    pub fn mark_modified(&mut self) {
        self.version += 1;
    }
    
    // Add a method to get the current version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    // Add a method to increment the version
    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    pub fn add_stroke(&mut self, stroke: StrokeRef) {
        self.strokes.push(stroke);
        self.mark_modified();
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
        // Mark document as modified to ensure proper state update
        self.mark_modified();
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
        self.images.iter().find(|img| img.id() == id)
    }
    
    pub fn find_stroke_by_id(&self, id: usize) -> Option<&StrokeRef> {
        self.strokes.iter().find(|stroke| stroke.id() == id)
    }

    /// Find any element by ID
    pub fn find_element_by_id(&self, id: usize) -> Option<ElementType> {
        // First try images (faster lookup with direct ID)
        let image_result = self.find_image_by_id(id)
            .map(|img| ElementType::Image(img.clone()));
            
        if image_result.is_some() {
            return image_result;
        }
        
        // Then try strokes
        let stroke_result = self.find_stroke_by_id(id)
            .map(|stroke| ElementType::Stroke(stroke.clone()));
            
        if stroke_result.is_some() {
            return stroke_result;
        }
        
        None
    }
    
    /// Check if document contains element with given ID
    pub fn contains_element(&self, id: usize) -> bool {
        self.find_element_by_id(id).is_some()
    }

    pub fn find_element(&self, id: usize) -> Option<ElementType> {
        self.find_element_by_id(id)
    }

    pub fn get_element_mut(&mut self, element_id: usize) -> Option<ElementTypeMut<'_>> {
        // First check images since they have explicit IDs
        for image in self.images.iter_mut() {
            if image.id() == element_id {
                return Some(ElementTypeMut::Image(image));
            }
        }
        
        // Then check strokes by ID
        for stroke in self.strokes.iter_mut() {
            if stroke.id() == element_id {
                return Some(ElementTypeMut::Stroke(stroke));
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
        self.find_element(id)
    }
    
    // Improved method to replace a stroke by ID while preserving order
    pub fn replace_stroke_by_id(&mut self, id: usize, new_stroke: StrokeRef) -> bool {
        // Find the index of the stroke with the matching ID
        let mut index_to_remove = None;
        for (i, stroke) in self.strokes.iter().enumerate() {
            if stroke.id() == id {
                index_to_remove = Some(i);
                break;
            }
        }
        
        // If found, replace it at the same index
        if let Some(index) = index_to_remove {
            log::info!("Replacing stroke at index {} (ID: {})", index, id);
            
            // Replace at the same index to preserve ordering
            self.strokes[index] = new_stroke;
            
            // Mark document as modified
            self.mark_modified();
            return true;
        }
        false
    }
    
    // Improved method to replace an image by ID while preserving order
    pub fn replace_image_by_id(&mut self, id: usize, new_image: ImageRef) -> bool {
        // Find the index of the image with the matching ID
        let mut index_to_remove = None;
        for (i, image) in self.images.iter().enumerate() {
            if image.id() == id {
                index_to_remove = Some(i);
                break;
            }
        }
        
        // If found, replace it at the same index
        if let Some(index) = index_to_remove {
            log::info!("Replacing image ID: {}", id);
            
            // Replace at the same index to preserve ordering
            self.images[index] = new_image;
            
            // Mark document as modified multiple times to ensure update
            for _ in 0..5 {
                self.mark_modified();
            }
            
            return true;
        }
        
        log::error!("Could not find image with ID: {} to replace", id);
        false
    }

    /// Get element position in draw order
    pub fn element_draw_index(&self, id: usize) -> Option<(usize, ElementType)> {
        // Check images first
        for (i, img) in self.images.iter().enumerate() {
            if img.id() == id {
                return Some((i, ElementType::Image(img.clone())));
            }
        }
        
        // Then check strokes
        let img_count = self.images.len();
        for (i, stroke) in self.strokes.iter().enumerate() {
            if stroke.id() == id {
                return Some((img_count + i, ElementType::Stroke(stroke.clone())));
            }
        }
        
        None
    }
}

// Helper function to calculate distance from a point to a line segment
fn distance_to_line_segment(point: egui::Pos2, line_start: egui::Pos2, line_end: egui::Pos2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    
    let line_len = line_vec.length();
    if line_len == 0.0 {
        return point_vec.length();
    }
    
    let t = ((point_vec.x * line_vec.x + point_vec.y * line_vec.y) / line_len).clamp(0.0, line_len);
    let projection = line_start + (line_vec * t / line_len);
    (point - projection).length()
} 