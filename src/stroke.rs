use egui::{Color32, Pos2};
use std::sync::Arc;
use crate::id_generator;
use crate::renderer::StrokePreview;

// Immutable stroke for sharing
#[derive(Clone, Debug)]
pub struct Stroke {
    id: usize,
    points: Vec<Pos2>,
    color: Color32,
    thickness: f32,
}

// Mutable stroke for editing
#[derive(Clone)]
pub struct MutableStroke {
    id: usize,
    points: Vec<Pos2>,
    color: Color32,
    thickness: f32,
}

// Define a reference-counted type alias for Stroke
pub type StrokeRef = Arc<Stroke>;

impl Stroke {
    // Create a new immutable stroke
    pub fn new(color: Color32, thickness: f32, points: Vec<Pos2>) -> Self {
        let id = id_generator::generate_id();
        Self {
            id,
            points,
            color,
            thickness,
        }
    }

    // Create a new reference-counted Image with a specific ID
    pub fn new_ref_with_id(id: usize, points: Vec<Pos2>, color: Color32, thickness: f32) -> StrokeRef {
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

    pub fn id(&self) -> usize {
        self.id
    }

    // Add translate method to create a new stroke with translated points
    pub fn translate(&self, delta: egui::Vec2) -> Self {
        // Create a new stroke with translated points
        let translated_points = self.points.iter()
            .map(|p| *p + delta)
            .collect();
        
        Self {
            id: self.id, // Preserve the ID during translation
            points: translated_points,
            color: self.color,
            thickness: self.thickness,
        }
    }

    // Add translate_in_place method for in-place translation
    pub fn translate_in_place(&mut self, delta: egui::Vec2) {
        for point in &mut self.points {
            *point += delta;
        }
    }
    
    // Add resize_in_place method for in-place resizing
    pub fn resize_in_place(&mut self, original_rect: egui::Rect, new_rect: egui::Rect) {
        // Calculate scale factors
        let scale_x = new_rect.width() / original_rect.width();
        let scale_y = new_rect.height() / original_rect.height();
        
        // Transform each point
        for point in &mut self.points {
            // Convert to relative coordinates in the original rect
            let relative_x = (point.x - original_rect.min.x) / original_rect.width();
            let relative_y = (point.y - original_rect.min.y) / original_rect.height();
            
            // Apply to new rect
            point.x = new_rect.min.x + (relative_x * new_rect.width());
            point.y = new_rect.min.y + (relative_y * new_rect.height());
        }
        
        // Scale thickness proportionally
        self.thickness *= (scale_x + scale_y) / 2.0;
    }
    
    // Add points_mut method for direct manipulation of points
    pub fn points_mut(&mut self) -> &mut Vec<egui::Pos2> {
        &mut self.points
    }
}

impl MutableStroke {
    // Create a new mutable stroke for editing
    pub fn new(color: Color32, thickness: f32) -> Self {
        let id = id_generator::generate_id();
        Self {
            id,
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
        Stroke {
            id: self.id, // Preserve the ID during conversion
            points: self.points.clone(),
            color: self.color,
            thickness: self.thickness,
        }
    }

    // Convert to a reference-counted StrokeRef
    pub fn to_stroke_ref(&self) -> StrokeRef {
        Arc::new(self.to_stroke())
    }
    
    // Convert to a StrokePreview for the renderer
    pub fn to_stroke_preview(&self) -> StrokePreview {
        StrokePreview::new(
            self.points.clone(),
            self.thickness,
            self.color
        )
    }
    
    // Convert to an immutable Stroke by consuming self (no cloning)
    pub fn into_stroke(self) -> Stroke {
        Stroke {
            id: self.id, // Preserve the ID during conversion
            points: self.points,
            color: self.color,
            thickness: self.thickness,
        }
    }
    
    // Convert to a reference-counted StrokeRef by consuming self (no cloning)
    pub fn into_stroke_ref(self) -> StrokeRef {
        Arc::new(self.into_stroke())
    }

    // Get a reference to the points for preview
    pub fn points(&self) -> &[Pos2] {
        &self.points
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
    let new_stroke = Stroke {
        id: stroke.id(), // Preserve the ID during resize
        points: resized_points,
        color: stroke.color(),
        thickness: stroke.thickness() * ((scale_x + scale_y) / 2.0), // Scale thickness proportionally
    };
    
    std::sync::Arc::new(new_stroke)
} 