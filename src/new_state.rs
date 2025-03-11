use std::collections::HashSet;
use crate::tools::ToolType;
use crate::element::ElementType;

pub type ElementId = usize;

pub struct EditorModel {
    pub content: Vec<ElementType>,
    pub version: usize,
    pub selected_element_ids: HashSet<ElementId>,
    pub active_tool: ToolType,
}

impl EditorModel {
    pub fn new() -> Self {
        // We need to create a default tool
        // Using DrawStroke as the default tool based on the ToolType enum
        let default_tool = crate::tools::new_tool("draw_stroke")
            .expect("Failed to create default draw_stroke tool");
            
        Self {
            content: Vec::new(),
            version: 0,
            selected_element_ids: HashSet::new(),
            active_tool: default_tool,
        }
    }

    pub fn mark_modified(&mut self) {
        self.version += 1;
    }

    pub fn get_element_by_id(&self, id: ElementId) -> Option<&ElementType> {
        self.content.iter().find(|element| element.get_stable_id() == id)
    }

    pub fn is_element_selected(&self, id: ElementId) -> bool {
        self.selected_element_ids.contains(&id)
    }
} 