use crate::tool::ToolType;
use crate::state::EditorState;
use crate::layer::Transform;
use crate::selection::Selection;

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
}

#[derive(Debug, Clone)]
pub enum LayerEvent {
    Added { index: usize },
    Removed { index: usize },
    Reordered { from: usize, to: usize },
    Transformed { 
        index: usize, 
        old_transform: Transform, 
        new_transform: Transform 
    },
    VisibilityChanged { index: usize, visible: bool },
}

#[derive(Debug, Clone)]
pub enum SelectionEvent {
    Created(Selection),
    Modified(Selection),
    Cleared,
}

#[derive(Debug, Clone)]
pub enum DocumentEvent {
    Saved,
    Loaded,
    Modified,
    UndoPerformed,
    RedoPerformed,
} 