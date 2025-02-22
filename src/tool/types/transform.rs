use eframe::egui;
use serde::{Serialize, Deserialize};
use crate::state::EditorContext;
use crate::gizmo::{TransformGizmo, GizmoHandle};
use crate::command::commands::Command;
use crate::event::{EditorEvent, TransformEvent};
use crate::layer::Transform;
use super::super::trait_def::{Tool, InputState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformTool {
    #[serde(skip)]
    pub active_gizmo: Option<TransformGizmo>,
    #[serde(skip)]
    initial_transform: Option<Transform>,
    #[serde(skip)]
    last_transform: Option<Transform>,
}

impl Default for TransformTool {
    fn default() -> Self {
        Self {
            active_gizmo: None,
            initial_transform: None,
            last_transform: None,
        }
    }
}

impl Tool for TransformTool {
    fn on_activate(&mut self, ctx: &mut EditorContext) {
        // When activating the transform tool, try to begin transform on active layer
        if let Ok(layer_id) = ctx.active_layer_id() {
            if let Err(e) = ctx.begin_transform(layer_id) {
                eprintln!("Failed to begin transform: {:?}", e);
                return;
            }
        }

        ctx.event_bus.emit(EditorEvent::ToolActivated {
            tool_type: "transform".to_string(),
        });
    }

    fn on_deactivate(&mut self, ctx: &mut EditorContext) {
        // Complete or cancel the transform based on whether there are changes
        if let Some(gizmo) = &mut self.active_gizmo {
            if gizmo.completed_transform.is_some() {
                if let Err(e) = ctx.complete_transform() {
                    eprintln!("Failed to complete transform: {:?}", e);
                }
            } else {
                if let Err(e) = ctx.cancel_transform() {
                    eprintln!("Failed to cancel transform: {:?}", e);
                }
            }
        }

        // Clear state when deactivating
        self.active_gizmo = None;
        self.initial_transform = None;
        self.last_transform = None;

        ctx.event_bus.emit(EditorEvent::ToolDeactivated {
            tool_type: "transform".to_string(),
        });
    }

    fn update(&mut self, ctx: &mut EditorContext, _input: &InputState) {
        // First check if we're in transforming state
        let layer_id = match ctx.state.transforming_layer_id() {
            Some(id) => id,
            None => return,
        };

        // Get the layer data we need
        let (layer_transform, layer_content) = match ctx.document.get_layer(layer_id) {
            Ok(layer) => (layer.transform.clone(), layer.content.clone()),
            Err(e) => {
                eprintln!("Failed to get layer: {:?}", e);
                return;
            }
        };

        // Calculate bounds
        let bounds = match ctx.calculate_transformed_bounds(&layer_content, &layer_transform) {
            Some(bounds) => bounds,
            None => return,
        };

        // Now get the gizmo and update it
        if let Some((_, gizmo)) = ctx.state.get_transform_data_mut() {
            // Update gizmo bounds
            gizmo.update_bounds(bounds);
            
            // If transform changed and gizmo is active, update the transform
            if gizmo.is_active && self.last_transform.as_ref() != Some(&layer_transform) {
                // Update transform and store result
                if let Err(e) = ctx.update_transform(layer_transform.clone()) {
                    eprintln!("Failed to update transform: {:?}", e);
                }
                self.last_transform = Some(layer_transform);
            }
        }
    }

    fn render(&self, _ctx: &EditorContext, _painter: &egui::Painter) {
        // The gizmo is now rendered directly through the UI system
        // No need to render anything here
    }
} 