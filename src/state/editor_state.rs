/// The core state machine for the paint application editor.
/// 
/// This module implements a strict state machine that enforces valid transitions between
/// different editor modes (idle, drawing, selecting, transforming). The state machine
/// helps prevent invalid state combinations and ensures tools properly clean up after use.
/// 
/// # State Transitions
/// 
/// The valid state transitions are:
/// ```text
///                    ┌─────────────┐
///                    │             │
///              ┌─────►   Drawing   ├─────┐
///              │     │             │     │
///              │     └─────────────┘     │
///              │                         │
///              │     ┌─────────────┐     │
/// ┌──────────┐ │     │             │     │ ┌──────────┐
/// │          ├─┼─────►  Selecting  ├─────┼─►          │
/// │   Idle   │ │     │             │     │ │   Idle   │
/// │          ◄─┼─────┤             ├─────┼─►          │
/// └──────────┘ │     └─────────┬───┘     │ └──────────┘
///              │               │          │
///              │     ┌─────────▼───┐     │
///              │     │             │     │
///              └─────► Transforming├─────┘
///                    │             │
///                    └─────────────┘
/// ```
/// 
/// # Usage
/// 
/// The state machine is primarily interacted with through the `EditorContext`, which provides
/// safe methods for state transitions. Direct manipulation of states should be avoided.
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use crate::state::{EditorContext, EditorState};
/// 
/// // States are typically transitioned through the context:
/// let mut context = EditorContext::new(document, renderer);
/// context.begin_drawing(tool);  // Transitions to Drawing state
/// context.return_to_idle();     // Returns to Idle state
/// ```
use crate::tool::types::DrawingTool;
use crate::selection::SelectionMode;
use crate::layer::LayerId;
use crate::gizmo::TransformGizmo;
use crate::stroke::Stroke;
use serde::{Serialize, Deserialize};
use eframe::egui::Pos2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectionInProgress {
    pub start: Pos2,
    pub current: Pos2,
    pub mode: SelectionMode,
    pub points: Vec<Pos2>,
}

/// The possible states of the editor.
/// 
/// Each variant represents a distinct mode of operation, with its own
/// set of allowed operations and associated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorState {
    /// No active operation
    Idle,
    /// Currently drawing with a tool
    Drawing {
        tool: DrawingTool,
        stroke: Option<Stroke>,
    },
    /// Currently making a selection
    Selecting {
        mode: SelectionMode,
        in_progress: Option<SelectionInProgress>,
    },
    /// Currently transforming a layer
    Transforming {
        layer_id: LayerId,
        #[serde(skip)]
        gizmo: TransformGizmo,
    },
}

impl PartialEq for EditorState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EditorState::Idle, EditorState::Idle) => true,
            (EditorState::Drawing { tool: t1, stroke: s1 }, EditorState::Drawing { tool: t2, stroke: s2 }) => {
                t1 == t2 && s1 == s2
            }
            (EditorState::Selecting { mode: m1, in_progress: p1 }, EditorState::Selecting { mode: m2, in_progress: p2 }) => {
                m1 == m2 && p1 == p2
            }
            (EditorState::Transforming { layer_id: l1, .. }, EditorState::Transforming { layer_id: l2, .. }) => {
                l1 == l2 // Skip comparing gizmo as it's transient
            }
            _ => false,
        }
    }
}

impl EditorState {
    /// Validates whether a transition to the new state is allowed
    pub fn can_transition_to(&self, new_state: &EditorState) -> bool {
        match (self, new_state) {
            // From Idle, we can go to any state
            (EditorState::Idle, _) => true,

            // From Drawing, we can only finish or cancel (go back to Idle)
            (EditorState::Drawing { .. }, EditorState::Idle) => true,

            // From Selecting, we can finish selection or cancel, or go to transform
            (EditorState::Selecting { .. }, EditorState::Idle) => true,
            (EditorState::Selecting { .. }, EditorState::Transforming { .. }) => true,

            // From Transforming, we can only finish or cancel
            (EditorState::Transforming { .. }, EditorState::Idle) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    /// Returns true if the editor is currently in an idle state
    pub fn is_idle(&self) -> bool {
        matches!(self, EditorState::Idle)
    }

    /// Returns true if the editor is currently in a drawing state
    pub fn is_drawing(&self) -> bool {
        matches!(self, EditorState::Drawing { .. })
    }

    /// Returns true if the editor is currently in a selecting state
    pub fn is_selecting(&self) -> bool {
        matches!(self, EditorState::Selecting { .. })
    }

    /// Returns true if the editor is currently in a transforming state
    pub fn is_transforming(&self) -> bool {
        matches!(self, EditorState::Transforming { .. })
    }

    /// Returns true if the editor can start a transform operation
    pub fn can_transform(&self) -> bool {
        matches!(self, EditorState::Idle | EditorState::Selecting { .. })
    }

    /// Returns the current drawing tool if in drawing state
    pub fn current_drawing_tool(&self) -> Option<&DrawingTool> {
        match self {
            EditorState::Drawing { tool, .. } => Some(tool),
            _ => None,
        }
    }

    /// Returns the current selection mode if in selecting state
    pub fn current_selection_mode(&self) -> Option<&SelectionMode> {
        match self {
            EditorState::Selecting { mode, .. } => Some(mode),
            _ => None,
        }
    }

    /// Returns the current transform gizmo if in transforming state
    pub fn current_transform_gizmo(&self) -> Option<&TransformGizmo> {
        match self {
            EditorState::Transforming { gizmo, .. } => Some(gizmo),
            _ => None,
        }
    }

    /// Returns a mutable reference to the current transform gizmo if in transforming state
    pub fn current_transform_gizmo_mut(&mut self) -> Option<&mut TransformGizmo> {
        match self {
            EditorState::Transforming { gizmo, .. } => Some(gizmo),
            _ => None,
        }
    }

    /// Returns the layer ID being transformed if in transforming state
    pub fn transforming_layer_id(&self) -> Option<LayerId> {
        match self {
            EditorState::Transforming { layer_id, .. } => Some(*layer_id),
            _ => None,
        }
    }

    /// Returns the transform data (layer_id and gizmo) if in transforming state
    pub fn get_transform_data(&self) -> Option<(LayerId, &TransformGizmo)> {
        match self {
            EditorState::Transforming { layer_id, gizmo } => Some((*layer_id, gizmo)),
            _ => None,
        }
    }

    /// Returns a mutable reference to the transform data if in transforming state
    pub fn get_transform_data_mut(&mut self) -> Option<(LayerId, &mut TransformGizmo)> {
        match self {
            EditorState::Transforming { layer_id, gizmo } => Some((*layer_id, gizmo)),
            _ => None,
        }
    }

    /// Creates a new transforming state
    pub fn new_transforming(layer_id: LayerId, gizmo: TransformGizmo) -> Self {
        EditorState::Transforming {
            layer_id,
            gizmo,
        }
    }
} 