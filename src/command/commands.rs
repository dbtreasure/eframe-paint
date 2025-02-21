use crate::tool::ToolType;
use crate::state::EditorState;
use crate::layer::{LayerId, Transform};
use crate::stroke::Stroke;
use crate::selection::Selection;
use serde::{Serialize, Deserialize};
use eframe::egui::TextureHandle;
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub enum Command {
    SetTool(ToolType),
    BeginOperation(EditorState),
    EndOperation,
    AddStroke {
        layer_id: LayerId,
        stroke: Stroke,
    },
    AddLayer {
        name: String,
    },
    AddImageLayer {
        name: String,
        #[serde(skip)]
        texture: Option<TextureHandle>,
        size: [usize; 2],
        initial_transform: Transform,
    },
    TransformLayer {
        layer_id: LayerId,
        old_transform: Transform,
        new_transform: Transform,
    },
    ReorderLayer {
        from_index: usize,
        to_index: usize,
    },
    RenameLayer {
        layer_id: LayerId,
        old_name: String,
        new_name: String,
    },
    SetSelection {
        selection: Selection,
    },
    // Add other commands as needed
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::SetTool(tool) => f.debug_tuple("SetTool").field(tool).finish(),
            Command::BeginOperation(state) => f.debug_tuple("BeginOperation").field(state).finish(),
            Command::EndOperation => write!(f, "EndOperation"),
            Command::AddStroke { layer_id, stroke } => f
                .debug_struct("AddStroke")
                .field("layer_id", layer_id)
                .field("stroke", stroke)
                .finish(),
            Command::AddLayer { name } => f
                .debug_struct("AddLayer")
                .field("name", name)
                .finish(),
            Command::AddImageLayer { name, texture: _, size, initial_transform } => f
                .debug_struct("AddImageLayer")
                .field("name", name)
                .field("size", size)
                .field("initial_transform", initial_transform)
                .finish(),
            Command::TransformLayer { layer_id, old_transform, new_transform } => f
                .debug_struct("TransformLayer")
                .field("layer_id", layer_id)
                .field("old_transform", old_transform)
                .field("new_transform", new_transform)
                .finish(),
            Command::ReorderLayer { from_index, to_index } => f
                .debug_struct("ReorderLayer")
                .field("from_index", from_index)
                .field("to_index", to_index)
                .finish(),
            Command::RenameLayer { layer_id, old_name, new_name } => f
                .debug_struct("RenameLayer")
                .field("layer_id", layer_id)
                .field("old_name", old_name)
                .field("new_name", new_name)
                .finish(),
            Command::SetSelection { selection } => f
                .debug_struct("SetSelection")
                .field("selection", selection)
                .finish(),
        }
    }
} 