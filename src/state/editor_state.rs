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
use crate::tool::types::{DrawingTool, SelectionMode};
use crate::layer::LayerId;
use crate::gizmo::TransformGizmo;
use crate::stroke::Stroke;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionInProgress {
    // TODO: Add selection in progress state
}

/// The possible states of the editor.
/// 
/// Each variant represents a distinct mode of operation, with its own
/// set of allowed operations and associated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorState {
    /// The default state when no tool is actively being used.
    /// All new operations must start from this state.
    Idle,

    /// Active when a drawing tool is in use.
    /// Contains the current tool and optional in-progress stroke.
    Drawing {
        /// The active drawing tool
        tool: DrawingTool,
        /// The current stroke being drawn, if any
        stroke: Option<Stroke>,
    },

    /// Active during selection operations.
    /// Tracks the selection mode and in-progress selection state.
    Selecting {
        /// The current selection mode (e.g., rectangle, lasso)
        mode: SelectionMode,
        /// The in-progress selection, if any
        in_progress: Option<SelectionInProgress>,
    },

    /// Active when transforming a selected layer.
    /// Contains the target layer and associated transform gizmo.
    Transforming {
        /// The ID of the layer being transformed
        layer_id: LayerId,
        /// The transform gizmo controlling the transformation
        #[serde(skip)]
        gizmo: TransformGizmo,
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
} 