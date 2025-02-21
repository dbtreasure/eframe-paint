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

    /// Add a stroke to a layer
    AddStroke {
        layer_id: LayerId,
        stroke: Stroke,
    },

    /// Remove the last stroke from a layer (used for undo)
    RemoveLastStroke {
        layer_id: LayerId,
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
            Command::AddStroke { layer_id, stroke } => f
                .debug_struct("AddStroke")
                .field("layer_id", layer_id)
                .field("stroke", stroke)
                .finish(),
            Command::RemoveLastStroke { layer_id } => f
                .debug_struct("RemoveLastStroke")
                .field("layer_id", layer_id)
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
    pub fn execute(&self, ctx: &mut CommandContext) -> CommandResult {
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

            Command::AddStroke { layer_id, stroke } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.add_stroke(stroke.clone());
                Ok(())
            }

            Command::RemoveLastStroke { layer_id } => {
                let layer = ctx.document.get_layer_mut(*layer_id)?;
                layer.remove_last_stroke()
                    .ok_or(CommandError::InvalidParameters)?;
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
            Command::AddStroke { .. } => true,
            Command::RemoveLastStroke { .. } => true,
            Command::SetSelection { .. } => true,
            Command::AddImageLayer { .. } => true,
            Command::ReorderLayer { .. } => true,
            Command::RenameLayer { .. } => true,
        }
    }

    /// Create the inverse command for undo operations
    pub fn inverse(&self, ctx: &CommandContext) -> Option<Command> {
        match self {
            Command::SetTool(_tool) => Some(Command::SetTool(ctx.current_tool.clone())),
            
            Command::BeginOperation(_) => None,
            
            Command::EndOperation => None,
            
            Command::TransformLayer { layer_id, transform: _ } => {
                let layer = ctx.document.get_layer(*layer_id).ok()?;
                Some(Command::TransformLayer {
                    layer_id: *layer_id,
                    transform: layer.transform.clone(),
                })
            }

            Command::AddStroke { layer_id, .. } => {
                Some(Command::RemoveLastStroke { layer_id: *layer_id })
            }

            Command::RemoveLastStroke { .. } => None,

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