use eframe::egui::{self, Color32, Stroke, Rect, Pos2, Shape};
use serde::{Serialize, Deserialize};
use crate::state::EditorContext;
use crate::selection::{Selection, SelectionShape, SelectionMode};
use crate::command::commands::Command;
use crate::event::{EditorEvent, SelectionEvent};
use super::super::trait_def::{Tool, InputState};

/// State for the selection tool's current operation
#[derive(Debug, Clone)]
struct SelectionState {
    /// Starting position of the selection
    start_pos: Pos2,
    /// Current points in the selection (used for freeform)
    points: Vec<Pos2>,
    /// Whether we're currently dragging
    is_dragging: bool,
    /// Last known position
    last_position: Pos2,
}

/// The selection tool for selecting regions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionTool {
    /// Current selection mode
    pub mode: SelectionMode,
    /// Minimum distance between freeform points
    min_point_distance: f32,
    /// Whether to show measurements
    pub show_measurements: bool,
    /// Whether to snap to grid
    pub snap_to_grid: bool,
    /// Grid size for snapping
    pub grid_size: f32,
    /// Current selection operation state
    #[serde(skip)]
    current_state: Option<SelectionState>,
}

impl Default for SelectionTool {
    fn default() -> Self {
        Self {
            mode: SelectionMode::Rectangle,
            min_point_distance: 5.0, // Minimum pixels between freeform points
            show_measurements: false,
            snap_to_grid: false,
            grid_size: 10.0,
            current_state: None,
        }
    }
}

impl SelectionTool {
    /// Set the selection mode
    pub fn set_mode(&mut self, ctx: &mut EditorContext, mode: SelectionMode) {
        if self.mode != mode {
            self.mode = mode;
            // Cancel any ongoing selection when mode changes
            if self.current_state.is_some() {
                self.cancel_selection(ctx);
            }
            // Emit mode changed event
            ctx.event_bus.emit(EditorEvent::SelectionChanged(
                SelectionEvent::ModeChanged(mode)
            ));
        }
    }

    /// Start a new selection operation
    fn start_selection(&mut self, ctx: &mut EditorContext, pos: Pos2) {
        self.current_state = Some(SelectionState {
            start_pos: pos,
            points: vec![pos],
            is_dragging: true,
            last_position: pos,
        });

        // Emit selection started event
        ctx.event_bus.emit(EditorEvent::SelectionChanged(
            SelectionEvent::Started
        ));
    }

    /// Update the current selection
    fn update_selection(&mut self, ctx: &mut EditorContext, pos: Pos2) {
        if let Some(state) = &mut self.current_state {
            match self.mode {
                SelectionMode::Rectangle => {
                    // For rectangle selection, we just need to track the current position
                    state.last_position = pos;
                    
                    // Emit in progress event with current bounds
                    let bounds = Rect::from_two_pos(state.start_pos, pos);
                    ctx.event_bus.emit(EditorEvent::SelectionChanged(
                        SelectionEvent::InProgress { bounds }
                    ));
                },
                SelectionMode::Freeform => {
                    // For freeform, we need to add points when they're far enough apart
                    let last_point = state.points.last().unwrap();
                    let distance = (pos - *last_point).length();
                    
                    if distance >= self.min_point_distance {
                        state.points.push(pos);
                        state.last_position = pos;
                        
                        // Emit in progress event with current bounds
                        if state.points.len() >= 2 {
                            let bounds = Rect::from_points(&state.points);
                            ctx.event_bus.emit(EditorEvent::SelectionChanged(
                                SelectionEvent::InProgress { bounds }
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Finish the current selection operation
    fn finish_selection(&mut self, ctx: &mut EditorContext, current_pos: Pos2) {
        if let Some(state) = self.current_state.take() {
            let selection = match self.mode {
                SelectionMode::Rectangle => {
                    let rect = Rect::from_two_pos(state.start_pos, current_pos);
                    Selection {
                        shape: SelectionShape::Rectangle(rect),
                    }
                },
                SelectionMode::Freeform => {
                    // Only create selection if we have enough points
                    if state.points.len() >= 3 {
                        let mut points = state.points;
                        // Close the path by adding the first point again
                        points.push(points[0]);
                        Selection {
                            shape: SelectionShape::Freeform(points),
                        }
                    } else {
                        // Fall back to rectangle for short freeform selections
                        let rect = Rect::from_two_pos(state.start_pos, current_pos);
                        Selection {
                            shape: SelectionShape::Rectangle(rect),
                        }
                    }
                }
            };

            // Create and execute the selection command
            let command = Command::SetSelection {
                selection: selection.clone(),
            };
            ctx.execute_command(Box::new(command));

            // Emit selection completed event
            ctx.event_bus.emit(EditorEvent::SelectionChanged(
                SelectionEvent::Created(selection)
            ));
        }
    }

    /// Cancel the current selection operation
    fn cancel_selection(&mut self, ctx: &mut EditorContext) {
        self.current_state = None;
        
        // Clear any existing selection
        let command = Command::ClearSelection;
        ctx.execute_command(Box::new(command));
        
        // Emit selection cleared event
        ctx.event_bus.emit(EditorEvent::SelectionChanged(
            SelectionEvent::Cleared
        ));
    }

    /// Render the selection preview
    fn render_preview(&self, painter: &egui::Painter, current_pos: Pos2) {
        if let Some(state) = &self.current_state {
            let stroke = Stroke::new(1.0, Color32::WHITE);
            let fill = Color32::from_rgba_unmultiplied(255, 255, 255, 32);
            
            match self.mode {
                SelectionMode::Rectangle => {
                    // Draw preview rectangle with fill and outline
                    let rect = Rect::from_two_pos(state.start_pos, current_pos);
                    painter.add(Shape::rect_filled(rect, 0.0, fill));
                    painter.add(Shape::rect_stroke(rect, 0.0, stroke));
                    
                    // Draw handles at corners
                    let corners = [
                        rect.left_top(),
                        rect.right_top(),
                        rect.left_bottom(),
                        rect.right_bottom(),
                    ];
                    for &corner in &corners {
                        let handle_rect = Rect::from_center_size(
                            corner,
                            egui::vec2(6.0, 6.0),
                        );
                        painter.add(Shape::rect_filled(handle_rect, 0.0, Color32::WHITE));
                        painter.add(Shape::rect_stroke(handle_rect, 0.0, stroke));
                    }
                },
                SelectionMode::Freeform => {
                    // Draw lines between consecutive points with fill
                    if state.points.len() >= 2 {
                        let mut points = state.points.clone();
                        points.push(current_pos);
                        
                        // Draw fill
                        painter.add(Shape::convex_polygon(
                            points.clone(),
                            fill,
                            stroke,
                        ));
                        
                        // Draw outline
                        for points in points.windows(2) {
                            painter.add(Shape::line_segment(
                                [points[0], points[1]],
                                stroke,
                            ));
                        }
                        
                        // Draw start point indicator
                        let start_rect = Rect::from_center_size(
                            state.start_pos,
                            egui::vec2(6.0, 6.0),
                        );
                        painter.add(Shape::rect_filled(start_rect, 0.0, Color32::WHITE));
                        painter.add(Shape::rect_stroke(start_rect, 0.0, stroke));
                    }
                }
            }
        }
    }
}

impl Tool for SelectionTool {
    fn on_activate(&mut self, ctx: &mut EditorContext) {
        // Clear any existing selection state
        self.current_state = None;
        
        // Emit tool activated event
        ctx.event_bus.emit(EditorEvent::ToolActivated {
            tool_type: "Selection".to_string(),
        });
    }

    fn on_deactivate(&mut self, ctx: &mut EditorContext) {
        // If there's an ongoing selection, cancel it
        if self.current_state.is_some() {
            self.cancel_selection(ctx);
        }
        
        // Emit tool deactivated event
        ctx.event_bus.emit(EditorEvent::ToolDeactivated {
            tool_type: "Selection".to_string(),
        });
    }

    fn update(&mut self, ctx: &mut EditorContext, input: &InputState) {
        if let Some(pos) = input.pointer_pos {
            if input.pointer_pressed {
                // Start new selection
                self.start_selection(ctx, pos);
            } else if input.pointer_released {
                // Finish selection
                self.finish_selection(ctx, pos);
            } else if let Some(state) = &self.current_state {
                if state.is_dragging {
                    // Update selection
                    self.update_selection(ctx, pos);
                }
            }

            // Handle escape key to cancel selection
            if input.modifiers.command && input.modifiers.alt {
                self.cancel_selection(ctx);
            }
        }
    }

    fn render(&self, _ctx: &EditorContext, painter: &egui::Painter) {
        // Just render the preview with the current state
        // The pointer position will be handled in the update method
        if let Some(state) = &self.current_state {
            self.render_preview(painter, state.last_position);
        }
    }
}

impl PartialEq for SelectionTool {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode &&
        self.min_point_distance == other.min_point_distance &&
        self.show_measurements == other.show_measurements &&
        self.snap_to_grid == other.snap_to_grid &&
        self.grid_size == other.grid_size
        // Intentionally skip comparing current_state as it's transient
    }
} 