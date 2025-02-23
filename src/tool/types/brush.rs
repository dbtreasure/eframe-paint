use eframe::egui::{self, Color32};
use serde::{Serialize, Deserialize};
use crate::state::EditorContext;
use crate::stroke::Stroke;
use crate::command::commands::Command;
use crate::event::EditorEvent;
use super::super::trait_def::{Tool, InputState};

/// State for the brush tool's current drawing operation
#[derive(Debug, Clone)]
struct BrushState {
    stroke: Stroke,
    last_position: egui::Pos2,
    pressure: f32,
}

/// The brush tool for freehand drawing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushTool {
    /// Current brush color
    pub color: Color32,
    /// Base brush thickness (modified by pressure)
    pub thickness: f32,
    /// Whether pressure sensitivity is enabled
    pub pressure_sensitivity: bool,
    /// Minimum thickness multiplier for pressure
    pub min_pressure: f32,
    /// Maximum thickness multiplier for pressure
    pub max_pressure: f32,
    /// Current drawing state
    #[serde(skip)]
    current_state: Option<BrushState>,
}

impl Default for BrushTool {
    fn default() -> Self {
        Self {
            color: Color32::BLACK,
            thickness: 1.0,
            pressure_sensitivity: true,
            min_pressure: 0.1,
            max_pressure: 2.0,
            current_state: None,
        }
    }
}

impl BrushTool {
    /// Calculate the actual brush thickness based on pressure
    fn calculate_thickness(&self, base_pressure: f32) -> f32 {
        if self.pressure_sensitivity {
            let pressure = base_pressure.clamp(0.0, 1.0);
            let pressure_range = self.max_pressure - self.min_pressure;
            let thickness_multiplier = self.min_pressure + (pressure * pressure_range);
            self.thickness * thickness_multiplier
        } else {
            self.thickness
        }
    }

    /// Start a new stroke at the given position
    pub fn start_stroke(&mut self, pos: egui::Pos2, pressure: f32) {
        let thickness = self.calculate_thickness(pressure);
        let mut stroke = Stroke::new(self.color, thickness);
        stroke.add_point(pos);
        
        self.current_state = Some(BrushState {
            stroke,
            last_position: pos,
            pressure,
        });
    }

    /// Continue the current stroke to a new position
    pub fn continue_stroke(&mut self, pos: egui::Pos2, pressure: f32) {
        // First get the values we need
        let (should_update_thickness, new_thickness) = if let Some(state) = &self.current_state {
            let pressure_delta = (pressure - state.pressure).abs();
            if pressure_delta > 0.1 {
                (true, self.calculate_thickness(pressure))
            } else {
                (false, 0.0) // thickness won't be used
            }
        } else {
            (false, 0.0)
        };

        // Then update the state
        if let Some(state) = &mut self.current_state {
            if should_update_thickness {
                state.stroke.thickness = new_thickness;
                state.pressure = pressure;
            }
            state.stroke.add_point(pos);
            state.last_position = pos;
        }
    }

    /// Finish and commit the current stroke
    pub fn finish_stroke(&mut self, ctx: &mut EditorContext) {
        if let Some(state) = self.current_state.take() {
            // Only commit strokes that have more than one point
            if state.stroke.points.len() > 1 {
                if let Ok(layer_id) = ctx.active_layer_id() {
                    let command = Command::AddStroke {
                        layer_id,
                        stroke: state.stroke,
                    };
                    
                    if let Err(e) = ctx.execute_command(Box::new(command)) {
                        eprintln!("Failed to execute stroke command: {:?}", e);
                    }
                    
                    // Emit stroke completed event
                    ctx.event_bus.emit(EditorEvent::StrokeCompleted {
                        layer_id,
                    });
                }
            }
        }
    }
}

impl Tool for BrushTool {
    fn on_activate(&mut self, ctx: &mut EditorContext) {
        // Reset any existing stroke
        if let Some(_state) = self.current_state.take() {
            self.finish_stroke(ctx);
        }
        
        // Emit tool activated event
        ctx.event_bus.emit(EditorEvent::ToolActivated {
            tool_type: "Brush".to_string(),
        });
    }

    fn on_deactivate(&mut self, ctx: &mut EditorContext) {
        // If there's an ongoing stroke, commit it
        if let Some(_state) = self.current_state.take() {
            self.finish_stroke(ctx);
        }
        
        // Emit tool deactivated event
        ctx.event_bus.emit(EditorEvent::ToolDeactivated {
            tool_type: "Brush".to_string(),
        });
    }

    fn update(&mut self, ctx: &mut EditorContext, input: &InputState) {
        if let Some(pos) = input.pointer_pos {
            let pressure = input.pressure.unwrap_or(1.0);

            if input.pointer_pressed {
                // Start new stroke
                self.start_stroke(pos, pressure);
                
                // Emit stroke started event
                if let Ok(layer_id) = ctx.active_layer_id() {
                    ctx.event_bus.emit(EditorEvent::StrokeStarted {
                        layer_id,
                    });
                }
            } else if input.pointer_released {
                // Finish stroke
                self.finish_stroke(ctx);
            } else if ctx.is_drawing() {
                // Continue stroke
                self.continue_stroke(pos, pressure);
            }
        }
    }

    fn render(&self, _ctx: &EditorContext, painter: &egui::Painter) {
        // Render the current stroke preview
        if let Some(state) = &self.current_state {
            state.stroke.render(painter);
        }
    }
}

impl PartialEq for BrushTool {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color &&
        self.thickness == other.thickness &&
        self.pressure_sensitivity == other.pressure_sensitivity &&
        self.min_pressure == other.min_pressure &&
        self.max_pressure == other.max_pressure
        // Intentionally skip comparing current_state as it's transient
    }
} 