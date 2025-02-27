use crate::stroke::{Stroke, StrokeRef, MutableStroke};
use crate::tools::{Tool, DrawStrokeTool};
use std::sync::Arc;
use std::any::Any;
use std::boxed::Box;

#[derive(Default)]
pub enum EditorState {
    #[default]
    Idle,
    // Instead of storing the stroke directly, we now store the active tool
    // which manages its own state
    UsingTool {
        active_tool: Box<dyn Tool>,
    },
}

impl EditorState {
    pub fn set_active_tool<T: Tool + 'static>(&mut self, tool: T) {
        *self = Self::UsingTool {
            active_tool: Box::new(tool),
        };
    }

    pub fn active_tool(&self) -> Option<&dyn Tool> {
        match self {
            Self::UsingTool { active_tool } => Some(active_tool.as_ref()),
            _ => None,
        }
    }

    pub fn active_tool_mut(&mut self) -> Option<&mut dyn Tool> {
        match self {
            Self::UsingTool { active_tool } => Some(active_tool.as_mut()),
            _ => None,
        }
    }

    pub fn is_using_tool(&self) -> bool {
        matches!(self, Self::UsingTool { .. })
    }
} 