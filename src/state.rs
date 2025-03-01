use crate::tools::ToolType;
use crate::document::Document;

#[derive(Default)]
pub enum EditorState {
    #[default]
    Idle,
    // Now using ToolType instead of Box<dyn Tool>
    UsingTool {
        active_tool: ToolType,
    },
}

impl EditorState {
    pub fn set_active_tool(&mut self, tool_type: ToolType, document: &Document) {
        // First deactivate the current tool if there is one
        if let Self::UsingTool { active_tool } = self {
            active_tool.deactivate(document);
        }
        
        // Create a new instance of the tool
        let mut new_tool = tool_type.new_instance();
        
        // Activate the new tool
        new_tool.activate(document);
        
        // Set the new tool as active
        *self = Self::UsingTool {
            active_tool: new_tool,
        };
    }

    pub fn active_tool(&self) -> Option<&ToolType> {
        match self {
            Self::UsingTool { active_tool } => Some(active_tool),
            _ => None,
        }
    }

    pub fn active_tool_mut(&mut self) -> Option<&mut ToolType> {
        match self {
            Self::UsingTool { active_tool } => Some(active_tool),
            _ => None,
        }
    }

    pub fn is_using_tool(&self) -> bool {
        matches!(self, Self::UsingTool { .. })
    }
} 