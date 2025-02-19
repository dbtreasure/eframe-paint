use crate::stroke::Stroke;
use serde::{Serialize, Deserialize};
use egui::TextureHandle;
use crate::layer::Transform;

/// Represents actions that can be undone/redone in the drawing application
#[derive(Serialize, Deserialize)]
pub enum Command {
    /// Adds a stroke to a specific layer
    AddStroke {
        /// The index of the layer to add the stroke to
        layer_index: usize,
        /// The stroke to add
        stroke: Stroke,
    },
    AddImageLayer {
        name: String,
        #[serde(skip)]
        #[serde(default)]
        texture: Option<TextureHandle>,
        size: [usize; 2],
        initial_transform: Transform,
    },
    AddLayer {
        name: String,
    },
    TransformLayer {
        layer_index: usize,
        old_transform: Transform,
        new_transform: Transform,
    },
    // Future commands can be added here as needed
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::AddStroke { layer_index, stroke } => f
                .debug_struct("AddStroke")
                .field("layer_index", layer_index)
                .field("stroke", stroke)
                .finish(),
            Command::AddImageLayer { name, size, .. } => f
                .debug_struct("AddImageLayer")
                .field("name", name)
                .field("size", size)
                .finish(),
            Command::AddLayer { name } => f
                .debug_struct("AddLayer")
                .field("name", name)
                .finish(),
            Command::TransformLayer { layer_index, .. } => f
                .debug_struct("TransformLayer")
                .field("layer_index", layer_index)
                .finish(),
        }
    }
}

// Implement Clone manually
impl Clone for Command {
    fn clone(&self) -> Self {
        match self {
            Command::AddStroke { layer_index, stroke } => Command::AddStroke {
                layer_index: *layer_index,
                stroke: stroke.clone(),
            },
            Command::AddImageLayer { name, size, .. } => Command::AddImageLayer {
                name: name.clone(),
                texture: None,  // Skip cloning the texture
                size: *size,
                initial_transform: Transform::default(),
            },
            Command::AddLayer { name } => Command::AddLayer {
                name: name.clone(),
            },
            Command::TransformLayer { layer_index, old_transform, new_transform } => Command::TransformLayer {
                layer_index: *layer_index,
                old_transform: old_transform.clone(),
                new_transform: new_transform.clone(),
            },
        }
    }
}

impl Command {
    pub fn execute(&self, document: &mut crate::Document) {
        match self {
            Command::AddStroke { layer_index, stroke } => {
                if let Some(layer) = document.layers.get_mut(*layer_index) {
                    layer.add_stroke(stroke.clone());
                }
            }
            Command::TransformLayer { layer_index, new_transform, .. } => {
                if let Some(layer) = document.layers.get_mut(*layer_index) {
                    layer.transform = *new_transform;
                }
            }
            _ => {}
        }
    }

    pub fn undo(&self, document: &mut crate::Document) {
        match self {
            Command::AddStroke { layer_index, .. } => {
                if let Some(layer) = document.layers.get_mut(*layer_index) {
                    layer.remove_last_stroke();
                }
            }
            Command::TransformLayer { layer_index, old_transform, .. } => {
                if let Some(layer) = document.layers.get_mut(*layer_index) {
                    layer.transform = *old_transform;
                }
            }
            _ => {}
        }
    }
} 