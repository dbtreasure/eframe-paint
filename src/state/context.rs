/// The main context for the paint application editor, managing state transitions
/// and coordinating between different components.
/// 
/// The `EditorContext` serves as the primary interface for state management in the
/// paint application. It ensures that all state transitions are valid and maintains
/// consistency between the editor state, document, renderer, and event system.
/// 
/// # State Management
/// 
/// The context provides safe methods for transitioning between states:
/// - `begin_drawing`: Start a new drawing operation
/// - `begin_selection`: Start a new selection operation
/// - `begin_transform`: Start a new transform operation
/// - `return_to_idle`: Return to the idle state
/// 
/// All state transitions are validated and will emit appropriate events when successful.
/// 
/// # Example
/// 
/// ```rust,no_run
/// use crate::state::EditorContext;
/// use crate::tool::types::DrawingTool;
/// 
/// let mut context = EditorContext::new(document, renderer);
/// 
/// // Start drawing
/// if let Err(e) = context.begin_drawing(DrawingTool::Brush) {
///     println!("Failed to start drawing: {:?}", e);
/// }
/// 
/// // Return to idle when done
/// context.return_to_idle();
/// ```
use crate::document::Document;
use crate::renderer::Renderer;
use crate::event::{EventBus, EditorEvent};
use crate::tool::types::{DrawingTool, SelectionMode};
use crate::layer::LayerId;
use crate::gizmo::TransformGizmo;
use super::EditorState;

/// Errors that can occur during state transitions.
#[derive(Debug)]
pub enum StateTransitionError {
    /// The requested state transition is not allowed from the current state
    InvalidTransition,
    /// The parameters provided for the state transition are invalid
    InvalidParameters,
}

/// The main context for the paint application editor.
#[derive(Debug)]
pub struct EditorContext {
    /// The current state of the editor
    pub state: EditorState,
    /// The document being edited
    pub document: Document,
    /// The renderer responsible for drawing the editor
    #[allow(dead_code)]
    pub renderer: Renderer,
    /// The event bus for broadcasting editor events
    pub event_bus: EventBus,
}

impl EditorContext {
    /// Creates a new editor context with the given document and renderer.
    /// 
    /// The context starts in the `Idle` state with a new event bus.
    pub fn new(document: Document, renderer: Renderer) -> Self {
        Self {
            state: EditorState::Idle,
            document,
            renderer,
            event_bus: EventBus::new(),
        }
    }

    /// Attempts to transition to a new state, validating the transition and emitting appropriate events.
    /// 
    /// This is the core method for state transitions, used by the more specific transition methods.
    /// It ensures that:
    /// 1. The transition is valid according to the state machine rules
    /// 2. Appropriate events are emitted for the state change
    /// 
    /// # Errors
    /// 
    /// Returns `StateTransitionError::InvalidTransition` if the requested transition
    /// is not allowed from the current state.
    pub fn transition_to(&mut self, new_state: EditorState) -> Result<(), StateTransitionError> {
        // Validate the transition
        if !self.state.can_transition_to(&new_state) {
            return Err(StateTransitionError::InvalidTransition);
        }

        // Store old state for event emission
        let old_state = self.state.clone();

        // Perform the transition
        self.state = new_state;

        // Emit state change event
        self.event_bus.emit(EditorEvent::StateChanged {
            old: old_state,
            new: self.state.clone(),
        });

        Ok(())
    }

    /// Begins a drawing operation with the specified tool.
    /// 
    /// This transitions the editor to the Drawing state. The operation must be
    /// started from the Idle state.
    pub fn begin_drawing(&mut self, tool: DrawingTool) -> Result<(), StateTransitionError> {
        self.transition_to(EditorState::Drawing {
            tool,
            stroke: None,
        })
    }

    /// Begins a selection operation with the specified mode.
    /// 
    /// This transitions the editor to the Selecting state. The operation must be
    /// started from the Idle state.
    pub fn begin_selection(&mut self, mode: SelectionMode) -> Result<(), StateTransitionError> {
        self.transition_to(EditorState::Selecting {
            mode,
            in_progress: None,
        })
    }

    /// Begins a transform operation on the specified layer.
    /// 
    /// This transitions the editor to the Transforming state. The operation can be
    /// started from either the Idle state or the Selecting state.
    pub fn begin_transform(&mut self, layer_id: LayerId, gizmo: TransformGizmo) -> Result<(), StateTransitionError> {
        self.transition_to(EditorState::Transforming {
            layer_id,
            gizmo,
        })
    }

    /// Returns the editor to the idle state.
    /// 
    /// This is the standard way to end any active operation. All operations
    /// (drawing, selecting, transforming) can transition back to idle.
    pub fn return_to_idle(&mut self) -> Result<(), StateTransitionError> {
        self.transition_to(EditorState::Idle)
    }

    /// Gets a reference to the current state.
    /// 
    /// This is useful for querying the current state without taking ownership.
    pub fn current_state(&self) -> &EditorState {
        &self.state
    }
} 