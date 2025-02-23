use super::{CommandContext, CommandResult, CommandError};
use crate::layer::{LayerId, Transform};
use crate::selection::{Selection, SelectionShape, SelectionMode};
use crate::tool::ToolType;
use crate::event::{EditorEvent, SelectionEvent};
use crate::stroke::Stroke;
use crate::state::EditorState;
use eframe::egui::{self, TextureHandle, Color32};
use serde::{Serialize, Deserialize};

/// Tool property values that can be set
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum ToolPropertyValue {
    Color(Color32),
    Thickness(f32),
    SelectionMode(SelectionMode),
}

/// Commands that can be executed in the editor
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Command {
    /// Change the active tool
    SetTool(ToolType),

    /// Begin a new operation (drawing, selecting, transforming)
    BeginOperation(EditorState),

    /// End the current operation
    EndOperation,

    /// Transform a layer
    TransformLayer {
        layer_id: LayerId,
        transform: Transform,
    },

    /// Begin a new transform operation
    BeginTransform {
        layer_id: LayerId,
        initial_transform: Transform,
    },

    /// Update a transform
    UpdateTransform {
        layer_id: LayerId,
        new_transform: Transform,
    },

    /// Complete a transform
    CompleteTransform {
        layer_id: LayerId,
        old_transform: Transform,
        new_transform: Transform,
    },

    /// Add a stroke to a layer
    AddStroke {
        layer_id: LayerId,
        stroke: Stroke,
    },

    /// Set the current selection
    SetSelection {
        selection: Selection,
    },

    /// Add a new layer (blank or image)
    AddLayer {
        name: String,
        #[serde(skip)]
        texture: Option<(TextureHandle, [usize; 2])>, // None for blank layer, Some for image layer with size
    },

    /// Reorder a layer
    ReorderLayer {
        layer_id: LayerId,
        new_index: usize,
    },

    /// Clear the current selection
    ClearSelection,

    /// Rename a layer
    RenameLayer {
        layer_id: LayerId,
        old_name: String,
        new_name: String,
    },

    /// Set a tool property
    SetToolProperty {
        tool: ToolType,
        property: String,
        value: ToolPropertyValue,
    },

    /// Toggle layer visibility
    ToggleLayerVisibility {
        layer_id: LayerId,
    },

    /// Set the active layer
    SetActiveLayer {
        layer_id: LayerId,
    },

    /// Undo the last command
    Undo,

    /// Redo the last undone command
    Redo,
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::SetTool(tool) => f
                .debug_struct("SetTool")
                .field("tool", tool)
                .finish(),
            Command::BeginOperation(state) => f
                .debug_struct("BeginOperation")
                .field("state", state)
                .finish(),
            Command::EndOperation => write!(f, "EndOperation"),
            Command::TransformLayer { layer_id, transform } => f
                .debug_struct("TransformLayer")
                .field("layer_id", layer_id)
                .field("transform", transform)
                .finish(),
            Command::BeginTransform { layer_id, initial_transform } => f
                .debug_struct("BeginTransform")
                .field("layer_id", layer_id)
                .field("initial_transform", initial_transform)
                .finish(),
            Command::UpdateTransform { layer_id, new_transform } => f
                .debug_struct("UpdateTransform")
                .field("layer_id", layer_id)
                .field("new_transform", new_transform)
                .finish(),
            Command::CompleteTransform { layer_id, old_transform, new_transform } => f
                .debug_struct("CompleteTransform")
                .field("layer_id", layer_id)
                .field("old_transform", old_transform)
                .field("new_transform", new_transform)
                .finish(),
            Command::AddStroke { layer_id, stroke } => f
                .debug_struct("AddStroke")
                .field("layer_id", layer_id)
                .field("stroke", stroke)
                .finish(),
            Command::SetSelection { selection } => f
                .debug_struct("SetSelection")
                .field("selection", selection)
                .finish(),
            Command::AddLayer { name, texture } => f
                .debug_struct("AddLayer")
                .field("name", name)
                .field("has_texture", &texture.is_some())
                .finish(),
            Command::ReorderLayer { layer_id, new_index } => f
                .debug_struct("ReorderLayer")
                .field("layer_id", layer_id)
                .field("new_index", new_index)
                .finish(),
            Command::ClearSelection => write!(f, "ClearSelection"),
            Command::RenameLayer { layer_id, old_name, new_name } => f
                .debug_struct("RenameLayer")
                .field("layer_id", layer_id)
                .field("old_name", old_name)
                .field("new_name", new_name)
                .finish(),
            Command::SetToolProperty { .. } => write!(f, "SetToolProperty"),
            Command::ToggleLayerVisibility { .. } => write!(f, "ToggleLayerVisibility"),
            Command::SetActiveLayer { .. } => write!(f, "SetActiveLayer"),
            Command::Undo => write!(f, "Undo"),
            Command::Redo => write!(f, "Redo"),
        }
    }
}

impl Command {
    /// Validate if the command can be executed in the current context
    pub fn validate<'a>(&self, ctx: &CommandContext<'a>) -> CommandResult {
        match self {
            Command::SetTool(_tool) => {
                // Tool changes are always valid
                Ok(())
            }
            Command::BeginOperation(state) => {
                // Check if we can transition to the new state
                if !ctx.editor_context.current_state().can_transition_to(state) {
                    Err(CommandError::InvalidStateTransition)
                } else {
                    Ok(())
                }
            }
            Command::EndOperation => {
                // Can only end operation if not in idle state
                if ctx.editor_context.current_state().is_idle() {
                    Err(CommandError::InvalidStateTransition)
                } else {
                    Ok(())
                }
            }
            Command::TransformLayer { layer_id, transform: _ } => {
                // Validate layer exists and can be transformed
                ctx.document.get_layer(*layer_id)
                    .map(|_| ())
                    .map_err(|_| CommandError::InvalidParameters)
            }
            Command::BeginTransform { layer_id, .. } => {
                // Check if layer exists and we can start transform
                if !ctx.editor_context.current_state().can_transform() {
                    return Err(CommandError::InvalidStateTransition);
                }
                ctx.document.get_layer(*layer_id)
                    .map(|_| ())
                    .map_err(|_| CommandError::InvalidParameters)
            }
            Command::UpdateTransform { layer_id, .. } => {
                // Can only update transform if in transforming state
                if !ctx.editor_context.current_state().is_transforming() {
                    return Err(CommandError::InvalidStateTransition);
                }
                ctx.document.get_layer(*layer_id)
                    .map(|_| ())
                    .map_err(|_| CommandError::InvalidParameters)
            }
            Command::CompleteTransform { layer_id, .. } => {
                // Can only complete transform if in transforming state
                if !ctx.editor_context.current_state().is_transforming() {
                    return Err(CommandError::InvalidStateTransition);
                }
                ctx.document.get_layer(*layer_id)
                    .map(|_| ())
                    .map_err(|_| CommandError::InvalidParameters)
            }
            Command::AddStroke { layer_id, stroke } => {
                // Validate layer exists and stroke is not empty
                if stroke.points.is_empty() {
                    return Err(CommandError::InvalidParameters);
                }
                ctx.document.get_layer(*layer_id)
                    .map(|_| ())
                    .map_err(|_| CommandError::InvalidParameters)
            }
            Command::SetSelection { selection: _ } => {
                // Selection can be set in any state
                Ok(())
            }
            Command::AddLayer { name, texture } => {
                // Validate name is not empty
                if name.is_empty() {
                    Err(CommandError::InvalidParameters)
                } else {
                    Ok(())
                }
            }
            Command::ReorderLayer { layer_id, new_index } => {
                // Validate both indices are valid
                if *new_index >= ctx.document.layers.len() {
                    return Err(CommandError::InvalidParameters);
                }
                ctx.document.get_layer(*layer_id)
                    .map(|_| ())
                    .map_err(|_| CommandError::InvalidParameters)
            }
            Command::ClearSelection => {
                // Can always clear selection
                Ok(())
            }
            Command::RenameLayer { layer_id, old_name: _, new_name } => {
                // Validate layer exists and new name is not empty
                if new_name.is_empty() {
                    return Err(CommandError::InvalidParameters);
                }
                ctx.document.get_layer(*layer_id)
                    .map(|_| ())
                    .map_err(|_| CommandError::InvalidParameters)
            }
            Command::SetToolProperty { .. } => Ok(()),
            Command::ToggleLayerVisibility { .. } => Ok(()),
            Command::SetActiveLayer { .. } => Ok(()),
            Command::Undo => {
                // Can only undo if there are commands in the history
                if !ctx.history.can_undo() {
                    Err(CommandError::InvalidStateTransition)
                } else {
                    Ok(())
                }
            }
            Command::Redo => {
                // Can only redo if there are undone commands
                if !ctx.history.can_redo() {
                    Err(CommandError::InvalidStateTransition)
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Execute the command with the given context
    pub fn execute<'a>(&self, ctx: &mut CommandContext<'a>) -> CommandResult {
        match self {
            Command::SetTool(tool) => {
                let old_tool = ctx.current_tool.clone();
                ctx.current_tool = tool.clone();
                ctx.event_bus.emit(EditorEvent::ToolChanged {
                    old: old_tool,
                    new: tool.clone(),
                });
                Ok(())
            }

            Command::BeginOperation(state) => {
                ctx.editor_context.transition_to(state.clone())?;
                Ok(())
            }

            Command::EndOperation => {
                ctx.editor_context.return_to_idle()?;
                Ok(())
            }

            Command::TransformLayer { layer_id, transform: new_transform } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.transform = new_transform.clone();
                Ok(())
            }

            Command::BeginTransform { layer_id, initial_transform } => {
                ctx.editor_context.begin_transform(*layer_id)?;
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.transform = initial_transform.clone();
                Ok(())
            }

            Command::UpdateTransform { layer_id, new_transform } => {
                ctx.editor_context.update_transform(new_transform.clone())?;
                Ok(())
            }

            Command::CompleteTransform { layer_id, old_transform, new_transform } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.transform = new_transform.clone();
                ctx.editor_context.complete_transform()?;
                Ok(())
            }

            Command::AddStroke { layer_id, stroke } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.add_stroke(stroke.clone());
                Ok(())
            }

            Command::SetSelection { selection } => {
                ctx.document.set_selection(selection.clone());
                ctx.event_bus.emit(EditorEvent::SelectionChanged(
                    SelectionEvent::Modified(selection.clone())
                ));
                Ok(())
            }

            Command::AddLayer { name, texture } => {
                if let Some((texture, size)) = texture {
                    ctx.document.add_image_layer(name, texture.clone(), *size);
                } else {
                    ctx.document.add_layer(name);
                }
                Ok(())
            }

            Command::ReorderLayer { layer_id, new_index } => {
                ctx.document.reorder_layer(*layer_id, *new_index);
                Ok(())
            }

            Command::ClearSelection => {
                ctx.document.clear_selection();
                ctx.event_bus.emit(EditorEvent::SelectionChanged(
                    SelectionEvent::Cleared
                ));
                Ok(())
            }

            Command::RenameLayer { layer_id, old_name: _, new_name } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.set_name(new_name.clone());
                Ok(())
            }

            Command::SetToolProperty { tool, property, value } => {
                match (tool, property.as_str(), value) {
                    (&ToolType::Brush(_), "color", &ToolPropertyValue::Color(color)) => {
                        ctx.editor_context.renderer.set_brush_color(color);
                        Ok(())
                    }
                    (&ToolType::Brush(_), "thickness", &ToolPropertyValue::Thickness(thickness)) => {
                        ctx.editor_context.renderer.set_brush_thickness(thickness);
                        Ok(())
                    }
                    (&ToolType::Selection(_), "mode", &ToolPropertyValue::SelectionMode(mode)) => {
                        ctx.editor_context.renderer.set_selection_mode(mode);
                        Ok(())
                    }
                    _ => Err(CommandError::InvalidParameters),
                }
            }

            Command::ToggleLayerVisibility { layer_id } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.visible = !layer.visible;
                Ok(())
            }

            Command::SetActiveLayer { layer_id } => {
                ctx.document.set_active_layer(*layer_id);
                Ok(())
            }

            Command::Undo => {
                ctx.history.undo()
            }

            Command::Redo => {
                ctx.history.redo()
            }
        }
    }

    /// Returns true if the command can be undone
    pub fn can_undo(&self) -> bool {
        match self {
            Command::SetTool(_) => true,
            Command::BeginOperation(_) => false,
            Command::EndOperation => false,
            Command::TransformLayer { .. } => true,
            Command::BeginTransform { .. } => false,
            Command::UpdateTransform { .. } => false,
            Command::CompleteTransform { .. } => true,
            Command::AddStroke { .. } => true,
            Command::SetSelection { .. } => true,
            Command::AddLayer { .. } => true,
            Command::ReorderLayer { .. } => true,
            Command::ClearSelection => true,
            Command::RenameLayer { .. } => true,
            Command::SetToolProperty { .. } => true,
            Command::ToggleLayerVisibility { .. } => true,
            Command::SetActiveLayer { .. } => true,
            Command::Undo => false,
            Command::Redo => false,
        }
    }

    /// Create the inverse command for undo operations
    pub fn inverse<'a>(&self, ctx: &CommandContext<'a>) -> Option<Command> {
        match self {
            Command::SetTool(_tool) => Some(Command::SetTool(ctx.current_tool.clone())),
            
            Command::BeginOperation(_) => None,
            
            Command::EndOperation => None,
            
            Command::TransformLayer { layer_id, transform } => {
                let layer = ctx.document.get_layer(*layer_id).ok()?;
                Some(Command::TransformLayer {
                    layer_id: *layer_id,
                    transform: layer.transform.clone(),
                })
            }

            Command::BeginTransform { layer_id, initial_transform } => {
                let layer = ctx.document.get_layer(*layer_id).ok()?;
                Some(Command::BeginTransform {
                    layer_id: *layer_id,
                    initial_transform: layer.transform.clone(),
                })
            }

            Command::UpdateTransform { layer_id, new_transform } => {
                let layer = ctx.document.get_layer(*layer_id).ok()?;
                Some(Command::UpdateTransform {
                    layer_id: *layer_id,
                    new_transform: layer.transform.clone(),
                })
            }

            Command::CompleteTransform { layer_id, old_transform, new_transform } => {
                Some(Command::CompleteTransform {
                    layer_id: *layer_id,
                    old_transform: new_transform.clone(),
                    new_transform: old_transform.clone(),
                })
            }

            Command::AddStroke { layer_id, .. } => {
                Some(Command::AddStroke {
                    layer_id: *layer_id,
                    stroke: Stroke::default(),
                })
            }

            Command::SetSelection { .. } => {
                let current = ctx.document.current_selection();
                Some(Command::SetSelection {
                    selection: current.as_ref().map(Clone::clone).unwrap_or_else(|| Selection {
                        shape: SelectionShape::Rectangle(egui::Rect::NOTHING)
                    }),
                })
            }

            Command::AddLayer { .. } => None,

            Command::ReorderLayer { layer_id, new_index } => {
                let current_index = layer_id.index();
                Some(Command::ReorderLayer {
                    layer_id: LayerId::new(*new_index),
                    new_index: current_index,
                })
            }

            Command::ClearSelection => {
                let current = ctx.document.current_selection();
                current.as_ref().map(|sel| Command::SetSelection {
                    selection: sel.clone(),
                })
            }

            Command::RenameLayer { layer_id, old_name, new_name } => {
                Some(Command::RenameLayer {
                    layer_id: *layer_id,
                    old_name: new_name.clone(),
                    new_name: old_name.clone(),
                })
            }

            Command::SetToolProperty { .. } => None,

            Command::ToggleLayerVisibility { layer_id } => {
                Some(Command::ToggleLayerVisibility {
                    layer_id: *layer_id,
                })
            }

            Command::SetActiveLayer { layer_id } => {
                Some(Command::SetActiveLayer {
                    layer_id: *layer_id,
                })
            }

            Command::Undo => None,
            Command::Redo => None,
        }
    }
} 