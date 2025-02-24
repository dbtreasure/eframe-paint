use crate::stroke::Stroke;

pub struct Document {
    strokes: Vec<Stroke>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            strokes: Vec::new(),
        }
    }

    pub fn add_stroke(&mut self, stroke: Stroke) {
        self.strokes.push(stroke);
    }

    pub fn strokes(&self) -> &[Stroke] {
        &self.strokes
    }

    pub fn remove_last_stroke(&mut self) -> Option<Stroke> {
        self.strokes.pop()
    }
} 