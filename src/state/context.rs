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
use crate::event::{EventBus, EditorEvent, TransformEvent, UndoRedoEventHandler};
use crate::tool::types::DrawingTool;
use crate::selection::SelectionMode;
use crate::layer::{LayerId, Transform, LayerContent};
use crate::gizmo::TransformGizmo;
use crate::command::{Command, CommandContext, CommandResult};
use crate::command::history::CommandHistory;
use crate::tool::ToolType;
use crate::input::{InputState, InputRouter};
use super::EditorState;
use super::persistence::{StatePersistence, PersistenceResult, EditorSnapshot};
use eframe::egui::{Rect, Pos2};
use thiserror::Error;
use std::time::Duration;
use crate::util::time;

/// Errors that can occur during state transitions or operations
#[derive(Debug, Error)]
pub enum EditorError {
    #[error("The requested state transition is not allowed from the current state")]
    InvalidStateTransition,
    
    #[error("The parameters provided for the operation are invalid")]
    InvalidParameters,
    
    #[error("The specified layer {0} does not exist")]
    LayerNotFound(LayerId),
    
    #[error("No layer is currently active")]
    NoActiveLayer,
    
    #[error("Failed to calculate bounds for transform operation")]
    TransformBoundsError,
    
    #[error("Transform operation failed: {0}")]
    TransformError(String),
}

pub type EditorResult<T> = Result<T, EditorError>;

/// Feedback message for user notifications
#[derive(Debug, Clone)]
pub struct Feedback {
    pub message: String,
    pub level: FeedbackLevel,
    pub timestamp: f32,
    pub duration: f32,
}

/// Level of feedback message
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeedbackLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl Feedback {
    /// Create a new feedback message
    pub fn new(message: impl Into<String>, level: FeedbackLevel) -> Self {
        Self {
            message: message.into(),
            level,
            timestamp: time::current_time(),
            duration: 3.0, // Default duration in seconds
        }
    }

    /// Check if the feedback message has expired
    pub fn is_expired(&self) -> bool {
        time::current_time() - self.timestamp >= self.duration
    }
}

/// The main context for the paint application editor.
#[derive(Debug)]
pub struct EditorContext {
    /// The current state of the editor
    pub state: EditorState,
    /// The document being edited
    pub document: Document,
    /// The renderer responsible for drawing the editor
    pub renderer: Renderer,
    /// The event bus for broadcasting editor events
    pub event_bus: EventBus,
    /// The current tool
    pub current_tool: ToolType,
    /// State persistence manager
    persistence: StatePersistence,
    /// Command history for undo/redo
    pub history: CommandHistory,
    /// Current feedback message
    feedback: Option<Feedback>,
    /// Last processed input state
    last_input: Option<InputState>,
    /// Input router for handling input events
    input_router: InputRouter,
}

impl EditorContext {
    /// Creates a new editor context with the given document and renderer.
    /// 
    /// The context starts in the `Idle` state with a new event bus.
    pub fn new(document: Document, renderer: Renderer) -> Self {
        let event_bus = EventBus::new();
        event_bus.subscribe(Box::new(UndoRedoEventHandler::new()));

        Self {
            state: EditorState::Idle,
            document,
            renderer,
            event_bus,
            current_tool: ToolType::default(),
            persistence: StatePersistence::new(String::from("./state")),
            history: CommandHistory::new(),
            feedback: None,
            last_input: None,
            input_router: InputRouter::new(),
        }
    }

    /// Helper method to emit transform events
    fn emit_transform_event(&mut self, event: TransformEvent) {
        self.event_bus.emit(EditorEvent::TransformChanged(event));
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
    /// Returns `EditorError::InvalidStateTransition` if the requested transition
    /// is not allowed from the current state.
    pub fn transition_to(&mut self, new_state: EditorState) -> EditorResult<()> {
        println!("[DEBUG] Attempting state transition from {:?} to {:?}", self.state, new_state);
        if !self.state.can_transition_to(&new_state) {
            println!("[DEBUG] State transition not allowed: current state: {:?}, attempted new state: {:?}", self.state, new_state);
            return Err(EditorError::InvalidStateTransition);
        }

        let old_state = self.state.clone();
        self.state = new_state;
        println!("[DEBUG] Successfully transitioned from {:?} to {:?}", old_state, self.state);

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
    pub fn begin_drawing(&mut self, tool: DrawingTool) -> Result<(), EditorError> {
        // Ensure we're in a valid state to start drawing
        if !self.state.is_idle() {
            return Err(EditorError::InvalidStateTransition);
        }

        // Ensure we have an active layer
        if self.document.active_layer.is_none() {
            return Err(EditorError::NoActiveLayer);
        }

        // Transition to drawing state
        self.transition_to(EditorState::Drawing {
            tool,
            stroke: None,
        })?;

        // Emit drawing started event
        if let Ok(layer_id) = self.active_layer_id() {
            self.event_bus.emit(EditorEvent::DrawingStarted {
                layer_id,
                tool_type: "Brush".to_string(),
            });
        }

        Ok(())
    }

    /// Begins a selection operation with the specified mode.
    /// 
    /// This transitions the editor to the Selecting state. The operation must be
    /// started from the Idle state.
    pub fn begin_selection(&mut self, mode: SelectionMode) -> Result<(), EditorError> {
        self.transition_to(EditorState::Selecting {
            mode,
            in_progress: None,
        })?;
        Ok(())
    }

    /// Begin a transform operation on the specified layer.
    /// 
    /// This transitions to the Transforming state. The operation can be
    /// started from either the Idle state or the Selecting state.
    pub fn begin_transform(&mut self, layer_id: LayerId) -> EditorResult<()> {
        // Verify we can start a transform
        if !self.state.can_transform() {
            return Err(EditorError::InvalidStateTransition);
        }

        // Get the layer data we need
        let (content, transform) = {
            let layer = self.document.get_layer(layer_id)
                .map_err(|_| EditorError::LayerNotFound(layer_id))?;
            (layer.content.clone(), layer.transform.clone())
        };

        // Calculate bounds for the gizmo
        let bounds = self.calculate_transformed_bounds(&content, &transform)
            .ok_or(EditorError::TransformBoundsError)?;

        // Create the gizmo and prepare the state transition
        let gizmo = TransformGizmo::new(bounds, transform.clone());
        let new_state = EditorState::new_transforming(layer_id, gizmo);

        // Perform the state transition
        self.transition_to(new_state)?;

        // Emit the transform started event
        self.emit_transform_event(TransformEvent::Started {
            layer_id,
            initial_transform: transform,
        });

        Ok(())
    }

    /// Internal method to update layer transform
    fn update_layer_transform(&mut self, layer_id: LayerId, new_transform: Transform) -> EditorResult<()> {
        let layer = self.document.get_layer_mut(layer_id)
            .map_err(|_| EditorError::LayerNotFound(layer_id))?;
        layer.transform = new_transform;
        Ok(())
    }

    /// Update the current transform operation
    pub fn update_transform(&mut self, new_transform: Transform) -> EditorResult<()> {
        // Get the layer ID from the current state
        let layer_id = match self.state.get_transform_data() {
            Some((id, _)) => id,
            None => return Err(EditorError::TransformError("Not in transform state".to_string())),
        };

        // Update the layer transform
        {
            let layer = self.document.get_layer_mut(layer_id)
                .map_err(|_| EditorError::LayerNotFound(layer_id))?;
            layer.transform = new_transform.clone();
        }

        // Clone the event bus to avoid borrow checker issues
        let event_bus = self.event_bus.clone();
        
        // Emit the event using the cloned event bus
        event_bus.emit(EditorEvent::TransformChanged(TransformEvent::Updated {
            layer_id,
            new_transform,
        }));

        Ok(())
    }

    /// Execute a command with validation and feedback
    pub fn execute_command(&mut self, command: Box<Command>) -> CommandResult {
        // Create command context
        let mut document = Document::default();
        let mut history = CommandHistory::new();
        let event_bus = self.event_bus.clone();
        
        std::mem::swap(&mut self.document, &mut document);
        std::mem::swap(&mut self.history, &mut history);
        
        let mut ctx = CommandContext::new(
            &mut document,
            self,
            &event_bus,
            self.current_tool.clone(),
            &mut history,
        );

        // Validate command
        if let Err(e) = command.validate(&ctx) {
            self.set_feedback(format!("Invalid command: {}", e), FeedbackLevel::Error);
            return Err(e);
        }

        // Execute command
        let result = command.execute(&mut ctx);
        
        // Restore document and history
        std::mem::swap(&mut self.document, &mut document);
        std::mem::swap(&mut self.history, &mut history);

        // Set feedback based on result
        match &result {
            Ok(_) => {
                self.set_feedback("Command executed successfully", FeedbackLevel::Success);
            }
            Err(e) => {
                self.set_feedback(format!("Command failed: {}", e), FeedbackLevel::Error);
            }
        }

        result
    }

    /// Complete the current transform operation
    pub fn complete_transform(&mut self) -> EditorResult<()> {
        // Get transform data first
        let transform_data = match self.state.get_transform_data() {
            Some((layer_id, gizmo)) => {
                if let Some((old_transform, new_transform)) = gizmo.completed_transform.clone() {
                    if old_transform != new_transform {
                        Some((layer_id, old_transform, new_transform))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        // Execute command if transform changed
        if let Some((layer_id, old_transform, new_transform)) = transform_data {
            // Create and execute the transform command
            let command = Command::CompleteTransform {
                layer_id,
                old_transform: old_transform.clone(),
                new_transform: new_transform.clone(),
            };

            // Take ownership of document and event_bus temporarily
            let mut document = Document::default();
            let mut history = CommandHistory::new();
            let event_bus = self.event_bus.clone();
            
            std::mem::swap(&mut self.document, &mut document);
            std::mem::swap(&mut self.history, &mut history);
            
            let current_tool = self.current_tool.clone();

            // Create command context
            let mut ctx = CommandContext::new(
                &mut document,
                self,
                &event_bus,
                current_tool,
                &mut history,
            );

            // Execute command
            let result = command.execute(&mut ctx)
                .map_err(|e| EditorError::TransformError(e.to_string()));

            // Restore document and event_bus
            std::mem::swap(&mut self.document, &mut document);
            std::mem::swap(&mut self.history, &mut history);

            // Check result
            result?;

            // Emit transform completed event
            self.emit_transform_event(TransformEvent::Completed {
                layer_id,
                old_transform,
                new_transform,
            });
        }

        // Return to idle state
        self.return_to_idle()
    }

    /// Cancel the current transform operation
    pub fn cancel_transform(&mut self) -> EditorResult<()> {
        if let Some((layer_id, _)) = self.state.get_transform_data() {
            // Emit transform cancelled event
            self.emit_transform_event(TransformEvent::Cancelled {
                layer_id,
            });
        }

        // Return to idle state
        self.return_to_idle()
    }

    /// Returns the editor to the idle state.
    /// 
    /// This is the standard way to end any active operation. All operations
    /// (drawing, selecting, transforming) can transition back to idle.
    pub fn return_to_idle(&mut self) -> EditorResult<()> {
        self.transition_to(EditorState::Idle)?;
        Ok(())
    }

    /// Gets a reference to the current state.
    /// 
    /// This is useful for querying the current state without taking ownership.
    pub fn current_state(&self) -> &EditorState {
        &self.state
    }

    /// Get the active layer ID
    pub fn active_layer_id(&self) -> EditorResult<LayerId> {
        self.document.active_layer
            .map(LayerId::new)
            .ok_or(EditorError::NoActiveLayer)
    }

    /// Check if the editor is currently in a drawing state
    pub fn is_drawing(&self) -> bool {
        matches!(self.state, EditorState::Drawing { .. })
    }

    /// Calculate the bounds of a layer's content
    pub fn calculate_layer_bounds(&self, content: &LayerContent) -> Option<Rect> {
        match content {
            LayerContent::Strokes(strokes) => {
                if strokes.is_empty() {
                    return None;
                }
                
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                
                for stroke in strokes {
                    for pos in &stroke.points {
                        min_x = min_x.min(pos.x);
                        min_y = min_y.min(pos.y);
                        max_x = max_x.max(pos.x);
                        max_y = max_y.max(pos.y);
                    }
                }
                
                Some(Rect::from_min_max(
                    Pos2::new(min_x, min_y),
                    Pos2::new(max_x, max_y),
                ))
            }
            LayerContent::Image { size, .. } => {
                // Use a fixed starting position for images (0,0)
                let width = size[0] as f32;
                let height = size[1] as f32;
                Some(Rect::from_min_max(
                    Pos2::new(0.0, 0.0),
                    Pos2::new(width, height),
                ))
            }
        }
    }

    /// Calculate the transformed bounds of a layer's content
    pub fn calculate_transformed_bounds(&self, content: &LayerContent, transform: &Transform) -> Option<Rect> {
        let original_bounds = self.calculate_layer_bounds(content)?;
        let pivot = original_bounds.center();
        let matrix = transform.to_matrix_with_pivot(pivot.to_vec2());
        
        // Transform all corners of the original bounds
        let corners = [
            original_bounds.left_top(),
            original_bounds.right_top(),
            original_bounds.right_bottom(),
            original_bounds.left_bottom(),
        ];
        
        let transformed_corners: Vec<Pos2> = corners.iter().map(|&pos| {
            let x_transformed = matrix[0][0] * pos.x + matrix[0][1] * pos.y + matrix[0][2];
            let y_transformed = matrix[1][0] * pos.x + matrix[1][1] * pos.y + matrix[1][2];
            Pos2::new(x_transformed, y_transformed)
        }).collect();
        
        Some(Rect::from_points(&transformed_corners))
    }

    /// Save the current state with a given name
    pub fn save_state(&self, name: &str) -> PersistenceResult<()> {
        self.persistence.save_snapshot(self, name)
    }

    /// Load a state by name
    pub fn load_state(&mut self, name: &str) -> PersistenceResult<()> {
        let snapshot = self.persistence.load_snapshot(name)?;
        snapshot.restore(self)
    }

    /// Try to perform auto-save if needed
    pub fn try_autosave(&mut self) -> PersistenceResult<()> {
        let snapshot = EditorSnapshot::new(self);
        self.persistence.save_snapshot(self, &format!("autosave_{}", snapshot.timestamp))
    }

    /// Attempt to recover from the most recent auto-save
    pub fn try_recover(&mut self) -> PersistenceResult<bool> {
        if let Some(name) = self.persistence.find_latest_autosave()? {
            self.load_state(&name)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Set a feedback message
    pub fn set_feedback(&mut self, message: impl Into<String>, level: FeedbackLevel) {
        self.feedback = Some(Feedback::new(message, level));
    }

    /// Get the current feedback message if not expired
    pub fn current_feedback(&mut self) -> Option<&Feedback> {
        self.feedback.as_ref().filter(|f| !f.is_expired())
    }

    /// Clear the current feedback message
    pub fn clear_feedback(&mut self) {
        self.feedback = None;
    }

    /// Process new input state and update editor accordingly
    pub fn process_input(&mut self, mut input: InputState) {
        // Skip if input hasn't changed significantly
        if let Some(last) = &self.last_input {
            if !self.has_significant_input_change(last, &input) {
                return;
            }
        }

        // Create a clone of the router to avoid borrow checker issues
        let mut router = self.input_router.clone();
        
        // Let the router handle the input
        router.handle_input(self, &mut input);

        // Store the updated router and input state
        self.input_router = router;
        self.last_input = Some(input);
    }

    /// Check if there's a significant change between two input states
    fn has_significant_input_change(&self, old: &InputState, new: &InputState) -> bool {
        // Always process if pointer state changed
        if old.pointer_pressed != new.pointer_pressed 
            || old.pointer_released != new.pointer_released
            || old.pointer_double_clicked != new.pointer_double_clicked {
            return true;
        }

        // Process if pointer moved significantly
        if let (Some(old_pos), Some(new_pos)) = (old.pointer_pos, new.pointer_pos) {
            if (old_pos - new_pos).length() > 1.0 {
                return true;
            }
        }

        // Process if modifiers changed
        if old.modifiers != new.modifiers {
            return true;
        }

        // Process if pressure changed significantly
        if let (Some(old_pressure), Some(new_pressure)) = (old.pressure, new.pressure) {
            if (old_pressure - new_pressure).abs() > 0.01 {
                return true;
            }
        }

        // Process if scroll changed
        if new.scroll_delta != Default::default() {
            return true;
        }

        false
    }

    /// Handle global shortcuts that work in any state
    pub fn handle_global_shortcuts(&mut self, input: &InputState) -> bool {
        // Undo/Redo
        if input.modifiers.command {
            if input.is_key_pressed(egui::Key::Z) {
                if input.modifiers.shift {
                    let command = Command::Redo;
                    self.execute_command(Box::new(command));
                } else {
                    let command = Command::Undo;
                    self.execute_command(Box::new(command));
                }
                return true;
            }
        }

        // Cancel current operation (Escape)
        if input.is_key_pressed(egui::Key::Escape) {
            match self.state {
                EditorState::Drawing { .. } => {
                    self.return_to_idle().ok();
                    return true;
                }
                EditorState::Selecting { .. } => {
                    self.return_to_idle().ok();
                    return true;
                }
                EditorState::Transforming { .. } => {
                    self.cancel_transform().ok();
                    return true;
                }
                _ => {}
            }
        }

        false
    }
}

// Add Clone implementation for EditorContext
impl Clone for EditorContext {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            document: self.document.clone(),
            renderer: self.renderer.clone(),
            event_bus: EventBus::new(), // Create a fresh event bus for the clone
            current_tool: self.current_tool.clone(),
            persistence: self.persistence.clone(),
            history: self.history.clone(),
            feedback: self.feedback.clone(),
            last_input: self.last_input.clone(),
            input_router: self.input_router.clone(),
        }
    }
} 