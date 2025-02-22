use eframe::egui::{self, Color32};
use serde::{Serialize, Deserialize};
use crate::state::EditorContext;
use crate::gizmo::{TransformGizmo, GizmoHandle};
use crate::command::commands::Command;
use crate::event::{EditorEvent, TransformEvent};
use crate::layer::{Transform, LayerId};
use super::super::trait_def::{Tool, InputState};

/// Configuration options for the transform tool
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let gizmo = TransformGizmo::new(bounds, transform.clone());

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
    fn update_transform(&mut self, ctx: &mut EditorContext, new_transform: Transform) -> Result<(), String> {
        if let Some(state) = &mut self.state {
            // Apply snapping if enabled
            let snapped_transform = if self.config.snap_rotation {
                let mut t = new_transform;
                let snap_radians = self.config.rotation_snap_degrees.to_radians();
                t.rotation = (t.rotation / snap_radians).round() * snap_radians;
                t
            } else {
                new_transform
            };

            // Update the transform
            if let Err(e) = ctx.update_transform(snapped_transform.clone()) {
                return Err(format!("Failed to update transform: {:?}", e));
            }

            // Update state
            state.last_transform = snapped_transform;
            state.has_changes = true;
        }

        Ok(())
    }

    /// Complete the current transform operation
    fn complete_transform(&mut self, ctx: &mut EditorContext) -> Result<(), String> {
        if let Some(state) = self.state.take() {
            if state.has_changes {
                if let Err(e) = ctx.complete_transform() {
                    return Err(format!("Failed to complete transform: {:?}", e));
                }
            }
        }
        Ok(())
    }

    /// Cancel the current transform operation
    fn cancel_transform(&mut self, ctx: &mut EditorContext) -> Result<(), String> {
        if let Some(state) = self.state.take() {
            if let Err(e) = ctx.cancel_transform() {
                return Err(format!("Failed to cancel transform: {:?}", e));
            }
        }
        Ok(())
    }
}

impl Tool for TransformTool {
    fn on_activate(&mut self, ctx: &mut EditorContext) {
        // When activating the transform tool, try to begin transform on active layer
        if let Ok(layer_id) = ctx.active_layer_id() {
            if let Err(e) = self.begin_transform(ctx, layer_id) {
                eprintln!("Failed to begin transform: {}", e);
                return;
            }
        }

        ctx.event_bus.emit(EditorEvent::ToolActivated {
            tool_type: "Transform".to_string(),
        });
    }

    fn on_deactivate(&mut self, ctx: &mut EditorContext) {
        // Complete or cancel the transform based on whether there are changes
        if let Some(state) = &self.state {
            if state.has_changes {
                if let Err(e) = self.complete_transform(ctx) {
                    eprintln!("Failed to complete transform: {}", e);
                }
            } else {
                if let Err(e) = self.cancel_transform(ctx) {
                    eprintln!("Failed to cancel transform: {}", e);
                }
            }
        }

        ctx.event_bus.emit(EditorEvent::ToolDeactivated {
            tool_type: "Transform".to_string(),
        });
    }

    fn update(&mut self, ctx: &mut EditorContext, input: &InputState) {
        // First check if we have active state
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
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

        // If transform changed and gizmo is active, update the transform
        if state.gizmo.is_active && state.last_transform != transform {
            if let Err(e) = self.update_transform(ctx, transform) {
                eprintln!("Failed to update transform: {}", e);
            }
        }
    }

    fn render(&self, ctx: &EditorContext, painter: &egui::Painter) {
        if let Some(state) = &self.state {
            // Render transform guides if enabled
            if self.config.show_rotation_guides {
                let bounds = state.gizmo.get_bounds();
                let center = bounds.center();
                let radius = bounds.width().max(bounds.height()) / 2.0;
                
                // Draw rotation guide circle
                painter.circle_stroke(
                    center,
                    radius,
                    egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 100)),
                );

                // Draw angle markers
                if self.config.snap_rotation {
                    let steps = (360.0 / self.config.rotation_snap_degrees) as i32;
                    for i in 0..steps {
                        let angle = i as f32 * self.config.rotation_snap_degrees.to_radians();
                        let dir = egui::Vec2::angled(angle);
                        let start = center + dir * (radius - 5.0);
                        let end = center + dir * (radius + 5.0);
                        painter.line_segment(
                            [start, end],
                            egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 100)),
                        );
                    }
                }
            }

            // Show dimensions if enabled
            if self.config.show_dimensions {
                let bounds = state.gizmo.get_bounds();
                let text = format!("{:.0}x{:.0}", bounds.width(), bounds.height());
                painter.text(
                    bounds.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::default(),
                    Color32::WHITE,
                );
            }

            // Show transform origin if enabled
            if self.config.show_transform_origin {
                let bounds = state.gizmo.get_bounds();
                let center = bounds.center();
                let size = 5.0;
                painter.circle_filled(
                    center,
                    size,
                    Color32::from_rgba_premultiplied(255, 255, 255, 150),
                );
            }
        }
    }
} 