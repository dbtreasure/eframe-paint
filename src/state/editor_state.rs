use crate::tool::types::{DrawingTool, SelectionMode};
use crate::layer::LayerId;
use crate::gizmo::TransformGizmo;
use crate::stroke::Stroke;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionInProgress {
    // TODO: Add selection in progress state
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorState {
    Idle,
    Drawing {
        tool: DrawingTool,
        stroke: Option<Stroke>,
    },
    Selecting {
        mode: SelectionMode,
        in_progress: Option<SelectionInProgress>,
    },
    Transforming {
        layer_id: LayerId,
        #[serde(skip)]
        gizmo: TransformGizmo,
    }
} 