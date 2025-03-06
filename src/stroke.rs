use egui::{Color32, Pos2};
use std::sync::Arc;

// Immutable stroke for sharing
#[derive(Clone)]
pub struct Stroke {
    points: Vec<Pos2>,
    color: Color32,
    thickness: f32,
}

// Mutable stroke for editing
#[derive(Clone)]
pub struct MutableStroke {
    points: Vec<Pos2>,
    color: Color32,
    thickness: f32,
}

// Define a reference-counted type alias for Stroke
pub type StrokeRef = Arc<Stroke>;

impl Stroke {
    // Create a new immutable stroke
    pub fn new(color: Color32, thickness: f32, points: Vec<Pos2>) -> Self {
        Self {
            points,
            color,
            thickness,
        }
    }

    // Create a new reference-counted Stroke
    pub fn new_ref(color: Color32, thickness: f32, points: Vec<Pos2>) -> StrokeRef {
        Arc::new(Self::new(color, thickness, points))
    }

    pub fn points(&self) -> &[Pos2] {
        &self.points
    }

    pub fn color(&self) -> Color32 {
        self.color
    }

    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    // Add translate method to create a new stroke with translated points
    pub fn translate(&self, delta: egui::Vec2) -> Self {
        // Create a new stroke with translated points
        let translated_points = self.points.iter()
            .map(|p| *p + delta)
            .collect();
        
        Self {
            points: translated_points,
            color: self.color,
            thickness: self.thickness,
        }
    }
}

// Add translate_ref function for StrokeRef
pub fn translate_ref(stroke_ref: &StrokeRef, delta: egui::Vec2) -> StrokeRef {
    Arc::new(stroke_ref.translate(delta))
}

impl MutableStroke {
    // Create a new mutable stroke for editing
    pub fn new(color: Color32, thickness: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            thickness,
        }
    }

    // Add a point to the mutable stroke
    pub fn add_point(&mut self, point: Pos2) {
        self.points.push(point);
    }

    // Convert to an immutable Stroke
    pub fn to_stroke(&self) -> Stroke {
        Stroke::new(self.color, self.thickness, self.points.clone())
    }

    // Convert to a reference-counted StrokeRef
    pub fn to_stroke_ref(&self) -> StrokeRef {
        Arc::new(self.to_stroke())
    }
    
    // Convert to an immutable Stroke by consuming self (no cloning)
    pub fn into_stroke(self) -> Stroke {
        Stroke::new(self.color, self.thickness, self.points)
    }
    
    // Convert to a reference-counted StrokeRef by consuming self (no cloning)
    pub fn into_stroke_ref(self) -> StrokeRef {
        Arc::new(self.into_stroke())
    }

    // Get a reference to the points for preview
    pub fn points(&self) -> &[Pos2] {
        &self.points
    }

    pub fn color(&self) -> Color32 {
        self.color
    }

    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    // Set the color
    pub fn set_color(&mut self, color: Color32) {
        self.color = color;
    }
    
    // Set the thickness
    pub fn set_thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
    }
}

// Resize a stroke based on original and new rectangles
pub fn resize_stroke(stroke: &StrokeRef, original_rect: egui::Rect, new_rect: egui::Rect) -> StrokeRef {
    // Create a new stroke with resized points
    let mut resized_points = Vec::with_capacity(stroke.points().len());
    
    // Calculate scale factors
    let scale_x = new_rect.width() / original_rect.width();
    let scale_y = new_rect.height() / original_rect.height();
    
    // Transform each point
    for point in stroke.points() {
        // Convert to relative coordinates in the original rect
        let relative_x = (point.x - original_rect.min.x) / original_rect.width();
        let relative_y = (point.y - original_rect.min.y) / original_rect.height();
        
        // Apply to new rect
        let new_x = new_rect.min.x + (relative_x * new_rect.width());
        let new_y = new_rect.min.y + (relative_y * new_rect.height());
        
        resized_points.push(egui::pos2(new_x, new_y));
    }
    
    // Create a new stroke with the resized points (color, thickness, points)
    let new_stroke = Stroke::new(
        stroke.color(),
        stroke.thickness() * ((scale_x + scale_y) / 2.0), // Scale thickness proportionally
        resized_points,
    );
    
    std::sync::Arc::new(new_stroke)
} 