use crate::stroke::Stroke;
use serde::{Serialize, Deserialize};

/// Represents actions that can be undone/redone in the drawing application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Adds a stroke to a specific layer
    AddStroke {
        /// The index of the layer to add the stroke to
        layer_index: usize,
        /// The stroke to add
        stroke: Stroke,
    },
    // Future commands can be added here as needed
} 