use crate::gizmo::TransformGizmo;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformTool {
    #[serde(skip)]
    pub active_gizmo: Option<TransformGizmo>,
}

impl Default for TransformTool {
    fn default() -> Self {
        Self {
            active_gizmo: None,
        }
    }
} 