use crate::tools::ToolType;
use crate::stroke::StrokeRef;
use crate::image::ImageRef;

#[derive(Clone)]
pub enum ElementType {
    Stroke(StrokeRef),
    Image(ImageRef),
}


#[derive(Clone)]
pub struct EditorState {
    active_tool: Option<ToolType>,
    selected_elements: Vec<ElementType>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            active_tool: None,
            selected_elements: Vec::new(),
        }
    }

    // Builder method to update the active tool
    pub fn with_active_tool(self, tool: Option<ToolType>) -> Self {
        Self {
            active_tool: tool,
            ..self
        }
    }

    // Builder method to update selected elements
    pub fn with_selected_elements(self, elements: Vec<ElementType>) -> Self {
        Self {
            selected_elements: elements,
            ..self
        }
    }

    // Convenience method to set a single element (or none)
    pub fn with_selected_element(self, element: Option<ElementType>) -> Self {
        match element {
            Some(elem) => Self {
                selected_elements: vec![elem],
                ..self
            },
            None => Self {
                selected_elements: Vec::new(),
                ..self
            }
        }
    }

    // Getters for state values
    pub fn active_tool(&self) -> Option<&ToolType> {
        self.active_tool.as_ref()
    }

    // Get all selected elements
    pub fn selected_elements(&self) -> &Vec<ElementType> {
        &self.selected_elements
    }

    // Convenience method to get the first selected element (if any)
    pub fn selected_element(&self) -> Option<&ElementType> {
        self.selected_elements.first()
    }
}