use crate::tools::Tool;
use crate::document::Document;
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
    pub fn set_active_tool<T: Tool + 'static>(&mut self, tool: T, document: &Document) {
        // First deactivate the current tool if there is one
        if let Self::UsingTool { active_tool } = self {
            active_tool.deactivate(document);
        }
        
        // Create the new tool
        let mut new_tool = Box::new(tool);
        
        // Activate the new tool
        new_tool.activate(document);
        
        // Set the new tool as active
        *self = Self::UsingTool {
            active_tool: new_tool,
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