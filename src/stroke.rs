use serde::{Serialize, Deserialize};
use eframe::egui::{self, Pos2, Color32, Stroke as EguiStroke, Shape};

/// A basic stroke representation for painting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stroke {
    pub points: Vec<Pos2>,
    pub thickness: f32,
    pub color: Color32,
}

impl Stroke {
    pub fn new(color: Color32, thickness: f32) -> Self {
        Self {
            points: Vec::new(),
            thickness,
            color,
        }
    }

    pub fn add_point(&mut self, pos: Pos2) {
        self.points.push(pos);
    }

    pub fn render(&self, painter: &egui::Painter) {
        if self.points.len() < 2 {
            return;
        }

        let stroke = EguiStroke::new(self.thickness, self.color);
        
        // Draw lines between consecutive points
        for points in self.points.windows(2) {
            let line = Shape::line_segment(
                [points[0], points[1]],
                stroke,
            );
            painter.add(line);
        }
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            thickness: 2.0,
            color: Color32::BLACK,
        }
    }
} 