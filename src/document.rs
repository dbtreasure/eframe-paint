use crate::stroke::StrokeRef;
use crate::image::ImageRef;
use crate::state::ElementType;
use egui;

// New enum for mutable element references
#[derive(Debug)]
pub enum ElementTypeMut<'a> {
    Stroke(&'a mut StrokeRef),
    Image(&'a mut ImageRef),
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

    pub fn find_element(&self, id: usize) -> Option<ElementType> {
        // First try to find an image
        if let Some(image) = self.find_image_by_id(id) {
            return Some(ElementType::Image(image.clone()));
        }
        
        // Then try to find a stroke
        if let Some(stroke) = self.find_stroke_by_id(id) {
            return Some(ElementType::Stroke(stroke.clone()));
        }
        
        None
    }

    pub fn get_element_mut(&mut self, element_id: usize) -> Option<ElementTypeMut<'_>> {
        // First check images since they have explicit IDs
        for image in &mut self.images {
            if image.id() == element_id {
                return Some(ElementTypeMut::Image(image));
            }
        }
        
        // Then check strokes by ID
        for stroke in &mut self.strokes {
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
            log::info!("Replacing image at index {} (ID: {})", index, id);
            
            // Replace at the same index to preserve ordering
            self.images[index] = new_image;
            
            // Mark document as modified
            self.mark_modified();
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