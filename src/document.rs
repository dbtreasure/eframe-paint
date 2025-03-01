use crate::stroke::StrokeRef;
use crate::image::ImageRef;

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

    pub fn remove_last_stroke(&mut self) -> Option<StrokeRef> {
        self.strokes.pop()
    }
    
    pub fn add_image(&mut self, image: ImageRef) {
        self.images.push(image);
    }
    
    pub fn images(&self) -> &[ImageRef] {
        &self.images
    }
    
    pub fn remove_last_image(&mut self) -> Option<ImageRef> {
        let image = self.images.pop();
        image
    }

    // Check if a point is over any stroke in the document
    pub fn is_point_over_stroke(&self, point: egui::Pos2) -> bool {
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
                    return true;
                }
            }
        }
        false
    }

    // Check if a point is over any image in the document
    pub fn is_point_over_image(&self, point: egui::Pos2) -> bool {
        for image in &self.images {
            let rect = image.rect();
            if rect.contains(point) {
                return true;
            }
        }
        false
    }
}

// Helper function to calculate distance from a point to a line segment
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