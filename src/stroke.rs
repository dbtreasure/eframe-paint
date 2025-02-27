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
} 