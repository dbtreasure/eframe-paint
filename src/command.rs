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
    /// Reorders a layer from one position to another
    ReorderLayer {
        /// The original index of the layer
        from_index: usize,
        /// The new index for the layer
        to_index: usize,
    },
    /// Renames a layer
    RenameLayer {
        /// The index of the layer to rename
        layer_index: usize,
        /// The old name of the layer
        old_name: String,
        /// The new name of the layer
        new_name: String,
    },
    // New variant for selection
    SetSelection { selection: crate::selection::Selection },
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
            Command::ReorderLayer { from_index, to_index } => f
                .debug_struct("ReorderLayer")
                .field("from_index", from_index)
                .field("to_index", to_index)
                .finish(),
            Command::RenameLayer { layer_index, old_name, new_name } => f
                .debug_struct("RenameLayer")
                .field("layer_index", layer_index)
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
            Command::ReorderLayer { from_index, to_index } => Command::ReorderLayer {
                from_index: *from_index,
                to_index: *to_index,
            },
            Command::RenameLayer { layer_index, old_name, new_name } => Command::RenameLayer {
                layer_index: *layer_index,
                old_name: old_name.clone(),
                new_name: new_name.clone(),
            },
            Command::SetSelection { selection } => Command::SetSelection {
                selection: selection.clone(),
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
            Command::ReorderLayer { from_index, to_index } => {
                if *from_index < document.layers.len() && *to_index < document.layers.len() {
                    let layer = document.layers.remove(*from_index);
                    document.layers.insert(*to_index, layer);
                    // Update active layer index if needed
                    if let Some(active_idx) = document.active_layer {
                        document.active_layer = Some(if active_idx == *from_index {
                            *to_index
                        } else if active_idx < *from_index && active_idx > *to_index {
                            active_idx + 1
                        } else if active_idx > *from_index && active_idx < *to_index {
                            active_idx - 1
                        } else {
                            active_idx
                        });
                    }
                }
            }
            Command::RenameLayer { layer_index, new_name, .. } => {
                if let Some(layer) = document.layers.get_mut(*layer_index) {
                    layer.name = new_name.clone();
                }
            }
            Command::SetSelection { selection } => {
                document.current_selection = Some(selection.clone());
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
            Command::ReorderLayer { from_index, to_index } => {
                if *from_index < document.layers.len() && *to_index < document.layers.len() {
                    let layer = document.layers.remove(*to_index);
                    document.layers.insert(*from_index, layer);
                    // Update active layer index if needed
                    if let Some(active_idx) = document.active_layer {
                        document.active_layer = Some(if active_idx == *to_index {
                            *from_index
                        } else if active_idx < *to_index && active_idx > *from_index {
                            active_idx - 1
                        } else if active_idx > *to_index && active_idx < *from_index {
                            active_idx + 1
                        } else {
                            active_idx
                        });
                    }
                }
            }
            Command::RenameLayer { layer_index, old_name, .. } => {
                if let Some(layer) = document.layers.get_mut(*layer_index) {
                    layer.name = old_name.clone();
                }
            }
            Command::SetSelection { selection: _ } => {
                document.current_selection = None;
            }
            _ => {}
        }
    }

    pub fn redo(&self, document: &mut crate::Document) {
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
            Command::ReorderLayer { from_index, to_index } => {
                if *from_index < document.layers.len() && *to_index < document.layers.len() {
                    let layer = document.layers.remove(*from_index);
                    document.layers.insert(*to_index, layer);
                    // Update active layer index if needed
                    if let Some(active_idx) = document.active_layer {
                        document.active_layer = Some(if active_idx == *from_index {
                            *to_index
                        } else if active_idx < *from_index && active_idx > *to_index {
                            active_idx + 1
                        } else if active_idx > *from_index && active_idx < *to_index {
                            active_idx - 1
                        } else {
                            active_idx
                        });
                    }
                }
            }
            Command::RenameLayer { layer_index, new_name, .. } => {
                if let Some(layer) = document.layers.get_mut(*layer_index) {
                    layer.name = new_name.clone();
                }
            }
            Command::SetSelection { selection } => {
                document.current_selection = Some(selection.clone());
            }
            _ => {}
        }
    }
} 