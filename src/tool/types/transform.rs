use eframe::egui::{self, Color32};
use serde::{Serialize, Deserialize};
use crate::state::EditorContext;
use crate::gizmo::{TransformGizmo, GizmoHandle, SnapMode};
use crate::command::commands::Command;
use crate::event::EditorEvent;
use crate::layer::{Transform, LayerId};
use super::super::trait_def::{Tool, InputState};

/// Configuration options for the transform tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformConfig {
    /// Whether to show rotation guides
    pub show_rotation_guides: bool,
    /// Whether to snap to common angles (0, 45, 90 degrees)
    pub snap_rotation: bool,
    /// Rotation snap increment in degrees
    pub rotation_snap_degrees: f32,
    /// Whether to maintain aspect ratio during scaling
    pub maintain_aspect_ratio: bool,
    /// Whether to show transform dimensions
    pub show_dimensions: bool,
    /// Whether to show transform origin
    pub show_transform_origin: bool,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            show_rotation_guides: true,
            snap_rotation: true,
            rotation_snap_degrees: 45.0,
            maintain_aspect_ratio: false,
            show_dimensions: true,
            show_transform_origin: true,
        }
    }
}

/// State for the transform tool's current operation
#[derive(Debug, Clone)]
struct TransformState {
    /// The layer being transformed
    layer_id: LayerId,
    /// The initial transform before any changes
    initial_transform: Transform,
    /// The last known transform state
    last_transform: Transform,
    /// The active gizmo
    gizmo: TransformGizmo,
    /// Whether changes have been made
    has_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformTool {
    /// Tool configuration
    pub config: TransformConfig,
    /// Current transform operation state
    #[serde(skip)]
    state: Option<TransformState>,
}

impl Default for TransformTool {
    fn default() -> Self {
        Self {
            config: TransformConfig::default(),
            state: None,
        }
    }
}

impl PartialEq for TransformTool {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
        // Intentionally skip comparing state as it's transient
    }
}

impl TransformTool {
    /// Begin a new transform operation
    fn begin_transform(&mut self, ctx: &mut EditorContext, layer_id: LayerId) -> Result<(), String> {
        // Get the layer data
        let (content, transform) = {
            let layer = ctx.document.get_layer(layer_id)
                .map_err(|e| format!("Failed to get layer: {:?}", e))?;
            (layer.content.clone(), layer.transform.clone())
        };

        // Calculate bounds for the gizmo
        let bounds = ctx.calculate_transformed_bounds(&content, &transform)
            .ok_or_else(|| "Failed to calculate bounds".to_string())?;

        // Create the gizmo
        let mut gizmo = TransformGizmo::new(bounds, transform.clone());

        // Sync gizmo preferences with tool config
        let mut preferences = gizmo.get_preferences().clone();
        preferences.show_measurements = self.config.show_dimensions;
        preferences.maintain_aspect_ratio = self.config.maintain_aspect_ratio;
        preferences.angle_snap_degrees = self.config.rotation_snap_degrees;
        preferences.snap_mode = if self.config.snap_rotation {
            crate::gizmo::SnapMode::Angle
        } else {
            crate::gizmo::SnapMode::None
        };
        gizmo.set_preferences(preferences);

        // Store the initial state
        self.state = Some(TransformState {
            layer_id,
            initial_transform: transform.clone(),
            last_transform: transform,
            gizmo,
            has_changes: false,
        });

        Ok(())
    }

    /// Update the current transform
    fn update_transform(&mut self, ctx: &mut EditorContext, transform: Transform) -> Result<(), String> {
        let state = self.state.as_mut()
            .ok_or_else(|| "No active transform".to_string())?;

        // Update the transform
        state.last_transform = transform.clone();
        state.has_changes = true;

        // Create command
        let command = Command::UpdateTransform {
            layer_id: state.layer_id,
            new_transform: transform,
        };

        // Execute command
        ctx.execute_command(Box::new(command));

        Ok(())
    }

    /// Complete the current transform operation
    fn complete_transform(&mut self, ctx: &mut EditorContext) -> Result<(), String> {
        let state = self.state.take()
            .ok_or_else(|| "No active transform".to_string())?;

        if state.has_changes {
            // Create command
            let command = Command::CompleteTransform {
                layer_id: state.layer_id,
                old_transform: state.initial_transform,
                new_transform: state.last_transform,
            };

            // Execute command
            ctx.execute_command(Box::new(command));
        }

        Ok(())
    }
}

impl Tool for TransformTool {
    fn on_activate(&mut self, ctx: &mut EditorContext) {
        // Reset any existing transform
        if let Some(state) = self.state.take() {
            self.complete_transform(ctx).ok();
        }
        
        // Emit tool activated event
        ctx.event_bus.emit(EditorEvent::ToolActivated {
            tool_type: "Transform".to_string(),
        });
    }

    fn on_deactivate(&mut self, ctx: &mut EditorContext) {
        // If there's an ongoing transform, complete it
        if let Some(state) = self.state.take() {
            self.complete_transform(ctx).ok();
        }
        
        // Emit tool deactivated event
        ctx.event_bus.emit(EditorEvent::ToolDeactivated {
            tool_type: "Transform".to_string(),
        });
    }

    fn update(&mut self, ctx: &mut EditorContext, input: &InputState) {
        // First check if we have active state
        let state = match &mut self.state {
            Some(state) => state,
            None => {
                // If we don't have state and the user clicked, try to start a transform
                if input.pointer_pressed {
                    if let Some(layer_id) = ctx.state.transforming_layer_id() {
                        if let Err(e) = self.begin_transform(ctx, layer_id) {
                            eprintln!("Failed to begin transform: {}", e);
                        }
                    }
                }
                return;
            }
        };

        // Get the layer data we need
        let (content, transform) = match ctx.document.get_layer(state.layer_id) {
            Ok(layer) => (layer.content.clone(), layer.transform.clone()),
            Err(e) => {
                eprintln!("Failed to get layer: {:?}", e);
                return;
            }
        };

        // Calculate bounds
        let bounds = match ctx.calculate_transformed_bounds(&content, &transform) {
            Some(bounds) => bounds,
            None => return,
        };

        // Update gizmo bounds
        state.gizmo.update_bounds(bounds);

        // Handle pointer input
        if input.pointer_pressed {
            // Check if we clicked a handle
            if let Some(pointer_pos) = input.pointer_pos {
                for handle in [
                    GizmoHandle::Move,
                    GizmoHandle::ScaleTopLeft,
                    GizmoHandle::ScaleTopRight,
                    GizmoHandle::ScaleBottomLeft,
                    GizmoHandle::ScaleBottomRight,
                    GizmoHandle::Rotate,
                ] {
                    let handle_bounds = state.gizmo.get_handle_bounds(handle);
                    if handle_bounds.contains(pointer_pos) {
                        state.gizmo.begin_transform(handle, pointer_pos);
                        break;
                    }
                }
            }
        } else if input.pointer_released {
            // End transform if we were transforming
            if state.gizmo.is_active {
                state.gizmo.end_transform();
                if let Err(e) = self.complete_transform(ctx) {
                    eprintln!("Failed to complete transform: {}", e);
                }
            }
        } else if let Some(current_pos) = input.pointer_pos {
            // Update transform if we're active
            if state.gizmo.is_active {
                if let Some(initial_pos) = state.gizmo.initial_pointer_pos {
                    let delta = current_pos - initial_pos;
                    state.gizmo.handle_pointer_move(delta);
                    // Get the transform before updating
                    let new_transform = state.gizmo.get_current_transform();
                    if let Err(e) = self.update_transform(ctx, new_transform) {
                        eprintln!("Failed to update transform: {}", e);
                    }
                }
            }
        }
    }

    fn render(&self, ctx: &EditorContext, painter: &egui::Painter) {
        if let Some(state) = &self.state {
            // Render the gizmo
            state.gizmo.render(painter);
        }
    }
} 