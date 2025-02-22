use super::{CommandContext, CommandResult, CommandError};
use crate::layer::{LayerId, Transform};
use crate::selection::{Selection, SelectionShape};
use crate::tool::ToolType;
use crate::event::{EditorEvent, SelectionEvent};
use crate::stroke::Stroke;
use crate::state::EditorState;
use eframe::egui::{self, TextureHandle};
use serde::{Serialize, Deserialize};

/// Commands that can be executed in the editor
#[derive(Clone, Serialize, Deserialize)]
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

    /// Add a new image layer
    AddImageLayer {
        name: String,
        #[serde(skip)]
        texture: Option<TextureHandle>,
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
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::SetTool(tool) => f
                .debug_tuple("SetTool")
                .field(tool)
                .finish(),
            Command::BeginOperation(state) => f
                .debug_tuple("BeginOperation")
                .field(state)
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
            Command::AddImageLayer { name, texture: _ } => f
                .debug_struct("AddImageLayer")
                .field("name", name)
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
        }
    }
}

impl Command {
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

            Command::TransformLayer { layer_id, transform } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.set_transform(transform.clone());
                Ok(())
            }

            Command::BeginTransform { layer_id, initial_transform } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.transform = initial_transform.clone();
                Ok(())
            }

            Command::UpdateTransform { layer_id, new_transform } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.transform = new_transform.clone();
                Ok(())
            }

            Command::CompleteTransform { layer_id, old_transform, new_transform } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.transform = new_transform.clone();
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

            Command::AddImageLayer { name, texture } => {
                if let Some(texture) = texture {
                    ctx.document.add_image_layer(name, texture.clone());
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
            Command::AddImageLayer { .. } => true,
            Command::ReorderLayer { .. } => true,
            Command::ClearSelection => true,
            Command::RenameLayer { .. } => true,
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

            Command::AddImageLayer { .. } => None,

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
        }
    }
} 