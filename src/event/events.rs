use crate::tool::ToolType;
use crate::state::EditorState;
use crate::layer::{Transform, LayerId};
use crate::selection::{Selection, SelectionMode};
use eframe::egui::{Rect, Vec2};

#[derive(Debug, Clone)]
pub enum TransformEvent {
    Started {
        layer_id: LayerId,
        initial_transform: Transform,
    },
    Updated {
        layer_id: LayerId,
        new_transform: Transform,
    },
    Completed {
        layer_id: LayerId,
        old_transform: Transform,
        new_transform: Transform,
    },
    Cancelled {
        layer_id: LayerId,
    },
}

#[derive(Debug, Clone)]
pub enum EditorEvent {
    ToolChanged { 
        old: ToolType, 
        new: ToolType 
    },
    StateChanged { 
        old: EditorState, 
        new: EditorState 
    },
    LayerChanged(LayerEvent),
    SelectionChanged(SelectionEvent),
    DocumentChanged(DocumentEvent),
    ToolActivated {
        tool_type: String,
    },
    ToolDeactivated {
        tool_type: String,
    },
    StrokeStarted {
        layer_id: LayerId,
    },
    StrokeCompleted {
        layer_id: LayerId,
    },
    TransformChanged(TransformEvent),
    ViewChanged {
        scale: f32,
        translation: Vec2,
    },
}

#[derive(Debug, Clone)]
pub enum LayerEvent {
    Added { index: usize },
    Removed { index: usize },
    Reordered { 
        old_index: usize,
        new_index: usize 
    },
    TransformChanged { 
        index: usize,
        old_transform: Transform,
        new_transform: Transform 
    },
    VisibilityChanged { index: usize, visible: bool },
    ContentChanged { index: usize },
    Transformed {
        index: usize,
        old_transform: Transform,
        new_transform: Transform
    },
}

#[derive(Debug, Clone)]
pub enum SelectionEvent {
    Created(Selection),
    Cleared,
    ModeChanged(SelectionMode),
    Started,
    InProgress { bounds: Rect },
    Modified(Selection),
}

#[derive(Debug, Clone)]
pub enum DocumentEvent {
    Modified,
    Saved,
    Loaded,
} 