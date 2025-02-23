use eframe::egui;
use crate::state::{EditorContext, EditorState};
use crate::command::Command;
use crate::event::{EditorEvent, SelectionEvent};
use crate::tool::types::DrawingTool;
use super::state::InputState;
use super::gestures::{GestureRecognizer, Gesture};
use crate::layer::Transform;
use crate::selection::{Selection, SelectionShape};
use crate::stroke::Stroke;

/// Routes input to appropriate handlers based on editor state
#[derive(Debug)]
pub struct InputRouter {
    gesture_recognizer: GestureRecognizer,
    /// Track if we're in the middle of a drag operation
    drag_in_progress: bool,
    /// Track the start position of a drag
    drag_start_pos: Option<egui::Pos2>,
    /// Track the last processed input state for delta calculations
    last_input: Option<InputState>,
}

impl Clone for InputRouter {
    fn clone(&self) -> Self {
        Self {
            gesture_recognizer: GestureRecognizer::new(), // Create new since it's not Clone
            drag_in_progress: self.drag_in_progress,
            drag_start_pos: self.drag_start_pos,
            last_input: self.last_input.clone(),
        }
    }
}

impl InputRouter {
    pub fn new() -> Self {
        Self {
            gesture_recognizer: GestureRecognizer::new(),
            drag_in_progress: false,
            drag_start_pos: None,
            last_input: None,
        }
    }

    /// Process input and route it to appropriate handlers
    pub fn handle_input(&mut self, ctx: &mut EditorContext, input: &mut InputState) {
        // Skip if input already consumed
        if input.is_consumed() {
            return;
        }

        // First check for global shortcuts
        if ctx.handle_global_shortcuts(input) {
            input.consume();
            return;
        }

        // Update gesture recognizer and handle any recognized gestures
        if let Some(gesture) = self.gesture_recognizer.update(input) {
            if self.handle_gesture(ctx, &gesture) {
                input.consume();
                return;
            }
        }

        // Route input based on current editor state
        match ctx.state {
            EditorState::Idle => self.handle_idle_state(ctx, input),
            EditorState::Drawing { .. } => self.handle_drawing_state(ctx, input),
            EditorState::Selecting { .. } => self.handle_selecting_state(ctx, input),
            EditorState::Transforming { .. } => self.handle_transforming_state(ctx, input),
        }

        // Store last input state for next frame
        self.last_input = Some(input.clone());
    }

    /// Handle recognized gestures
    fn handle_gesture(&mut self, ctx: &mut EditorContext, gesture: &Gesture) -> bool {
        match gesture {
            Gesture::Pinch { center, scale, velocity } => {
                // Handle zoom in transform mode or general view
                ctx.event_bus.emit(EditorEvent::ViewChanged {
                    scale: *scale,
                    translation: egui::Vec2::ZERO,
                });
                true
            }
            Gesture::Rotate { center, angle, velocity } => {
                // Handle rotation in transform mode
                if let EditorState::Transforming { .. } = ctx.state {
                    if let Some((layer_id, gizmo)) = ctx.state.get_transform_data_mut() {
                        let bounds = gizmo.get_bounds();
                        let mut transform = Transform::default();
                        transform.rotation = *angle;
                        transform.position = bounds.center().to_vec2();
                        ctx.update_transform(transform).ok();
                    }
                }
                true
            }
            Gesture::Pan { delta, velocity } => {
                // Handle pan in any mode
                ctx.event_bus.emit(EditorEvent::ViewChanged {
                    scale: 1.0,
                    translation: *delta,
                });
                true
            }
            Gesture::Tap { position, count } => {
                // Handle taps based on current state
                match ctx.state {
                    EditorState::Idle => {
                        if *count == 2 && ctx.document.current_selection.is_some() {
                            // Double tap to enter transform mode if there's a selection
                            if let Ok(layer_id) = ctx.active_layer_id() {
                                ctx.begin_transform(layer_id).ok();
                            }
                        }
                    }
                    _ => {}
                }
                true
            }
            Gesture::LongPress { position, duration } => {
                // Handle long press based on current state
                match ctx.state {
                    EditorState::Idle => {
                        // Long press to start selection
                        ctx.begin_selection(crate::selection::SelectionMode::Rectangle).ok();
                    }
                    _ => {}
                }
                true
            }
        }
    }

    /// Handle input in idle state
    fn handle_idle_state(&self, ctx: &mut EditorContext, input: &mut InputState) {
        if input.pointer_pressed {
            // Start operation based on current tool
            match &ctx.current_tool {
                crate::tool::ToolType::Brush(tool) => {
                    if let Some(pos) = input.pointer_pos {
                        ctx.begin_drawing(DrawingTool::Brush(tool.clone())).ok();
                        input.consume();
                    }
                }
                crate::tool::ToolType::Selection(tool) => {
                    if let Some(pos) = input.pointer_pos {
                        ctx.begin_selection(tool.mode).ok();
                        input.consume();
                    }
                }
                _ => {}
            }
        }
    }

    /// Handle input in drawing state
    fn handle_drawing_state(&self, ctx: &mut EditorContext, input: &mut InputState) {
        if let EditorState::Drawing { tool, stroke: _ } = &mut ctx.state {
            if let Some(pos) = input.pointer_pos {
                let pressure = input.pressure.unwrap_or(1.0);
                
                // Clone the tool to avoid borrowing ctx while using it
                let mut tool_clone = tool.clone();
                let should_return_to_idle = match &mut tool_clone {
                    DrawingTool::Brush(_) => self.handle_brush(&mut tool_clone, ctx, pos, pressure, input),
                    DrawingTool::Eraser(_) => self.handle_eraser(&mut tool_clone, ctx, pos, pressure, input),
                };
                
                // Update the tool state back in ctx
                if let EditorState::Drawing { tool: original_tool, .. } = &mut ctx.state {
                    *original_tool = tool_clone;
                }
                
                input.consume();

                // Handle state transition after tool operations
                if should_return_to_idle {
                    ctx.return_to_idle().ok();
                }
            }
        }
    }

    /// Handle brush tool input
    fn handle_brush(&self, brush: &mut DrawingTool, ctx: &mut EditorContext, pos: egui::Pos2, pressure: f32, input: &mut InputState) -> bool {
        if input.pointer_pressed {
            // Start new stroke
            if let DrawingTool::Brush(brush_tool) = brush {
                brush_tool.start_stroke(pos, pressure);
                
                // Emit stroke started event
                if let Ok(layer_id) = ctx.active_layer_id() {
                    ctx.event_bus.emit(EditorEvent::StrokeStarted {
                        layer_id,
                    });
                }
            }
            false
        } else if input.pointer_released {
            // Finish stroke
            if let DrawingTool::Brush(brush_tool) = brush {
                brush_tool.finish_stroke(ctx);
                
                // Return to idle state
                ctx.return_to_idle().ok();
            }
            true
        } else {
            // Continue stroke
            if let DrawingTool::Brush(brush_tool) = brush {
                brush_tool.continue_stroke(pos, pressure);
            }
            false
        }
    }

    /// Handle eraser tool input
    fn handle_eraser(&self, eraser: &mut DrawingTool, ctx: &mut EditorContext, pos: egui::Pos2, pressure: f32, input: &mut InputState) -> bool {
        if input.pointer_pressed {
            // Start new stroke
            if let DrawingTool::Eraser(eraser_tool) = eraser {
                eraser_tool.start_stroke(pos, pressure);
            }
            false
        } else if input.pointer_released {
            // Finish stroke
            if let DrawingTool::Eraser(eraser_tool) = eraser {
                eraser_tool.finish_stroke(ctx);
            }
            true
        } else {
            // Continue stroke
            if let DrawingTool::Eraser(eraser_tool) = eraser {
                eraser_tool.continue_stroke(pos, pressure);
            }
            false
        }
    }

    /// Handle input in selecting state
    fn handle_selecting_state(&self, ctx: &mut EditorContext, input: &mut InputState) {
        if let Some(pos) = input.pointer_pos {
            if input.pointer_released {
                // Complete selection
                if let Some(start_pos) = self.drag_start_pos {
                    let rect = egui::Rect::from_two_pos(start_pos, pos);
                    let selection = Selection {
                        shape: SelectionShape::Rectangle(rect),
                    };
                    ctx.event_bus.emit(EditorEvent::SelectionChanged(
                        SelectionEvent::Created(selection)
                    ));
                }
                ctx.return_to_idle().ok();
                input.consume();
            } else {
                // Update selection
                if let Some(start_pos) = self.drag_start_pos {
                    let rect = egui::Rect::from_two_pos(start_pos, pos);
                    ctx.event_bus.emit(EditorEvent::SelectionChanged(
                        SelectionEvent::InProgress { bounds: rect }
                    ));
                }
                input.consume();
            }
        }
    }

    /// Handle input in transforming state
    fn handle_transforming_state(&self, ctx: &mut EditorContext, input: &mut InputState) {
        if let Some(pos) = input.pointer_pos {
            if input.pointer_released {
                // Complete transform
                ctx.complete_transform().ok();
                ctx.return_to_idle().ok();
                input.consume();
            } else {
                // Update transform
                if let Some(start_pos) = self.drag_start_pos {
                    let delta = pos - start_pos;
                    let mut transform = Transform::default();
                    transform.position = delta;
                    ctx.update_transform(transform).ok();
                }
                input.consume();
            }
        }
    }
} 