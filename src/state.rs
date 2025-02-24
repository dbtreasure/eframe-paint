use crate::stroke::Stroke;

#[derive(Default)]
pub enum EditorState {
    #[default]
    Idle,
    Drawing {
        current_stroke: Stroke,
    },
}

impl EditorState {
    pub fn start_drawing(stroke: Stroke) -> Self {
        Self::Drawing { current_stroke: stroke }
    }

    pub fn is_drawing(&self) -> bool {
        matches!(self, Self::Drawing { .. })
    }

    pub fn take_stroke(&mut self) -> Option<Stroke> {
        if let Self::Drawing { current_stroke } = self {
            let stroke = current_stroke.clone();
            *self = Self::Idle;
            Some(stroke)
        } else {
            None
        }
    }
} 