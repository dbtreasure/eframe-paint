use egui::{Color32, Pos2};

#[derive(Clone)]
pub struct Stroke {
    points: Vec<Pos2>,
    color: Color32,
    thickness: f32,
}

impl Stroke {
    pub fn new(color: Color32, thickness: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            thickness,
        }
    }

    pub fn add_point(&mut self, point: Pos2) {
        self.points.push(point);
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