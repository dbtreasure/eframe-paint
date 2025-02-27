use crate::stroke::{Stroke, StrokeRef, MutableStroke};
use std::sync::Arc;

#[derive(Default)]
pub enum EditorState {
    #[default]
    Idle,
    Drawing {
        current_stroke: MutableStroke,
    },
}

impl EditorState {
    pub fn start_drawing(stroke: MutableStroke) -> Self {
        Self::Drawing { current_stroke: stroke }
    }

    pub fn is_drawing(&self) -> bool {
        matches!(self, Self::Drawing { .. })
    }

    pub fn take_stroke(&mut self) -> Option<StrokeRef> {
        if let Self::Drawing { current_stroke } = std::mem::replace(self, Self::Idle) {
            // Convert MutableStroke directly to StrokeRef without cloning
            Some(current_stroke.to_stroke_ref())
        } else {
            None
        }
    }
} 