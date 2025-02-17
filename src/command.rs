use crate::stroke::Stroke;
use serde::{Serialize, Deserialize};
use egui::TextureHandle;

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
            },
        }
    }
} 