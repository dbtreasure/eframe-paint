use crate::stroke::{Stroke, StrokeRef};

pub struct Document {
    strokes: Vec<StrokeRef>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            strokes: Vec::new(),
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
} 