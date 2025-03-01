use crate::tools::ToolType;
use crate::stroke::StrokeRef;
use crate::image::ImageRef;
use std::sync::Arc;
use std::ops::Deref;

#[derive(Clone)]
pub enum ElementType {
    Stroke(StrokeRef),
    Image(ImageRef),
}

// Inner data structure that will be wrapped in Arc
#[derive(Clone)]
struct EditorStateData {
    active_tool: Option<Arc<ToolType>>,
    selected_elements: Arc<[ElementType]>,
}

// Builder pattern for EditorState mutations
pub struct EditorStateBuilder {
    data: EditorStateData,
}

#[derive(Clone)]
pub struct EditorState {
    shared: Arc<EditorStateData>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(EditorStateData {
                active_tool: None,
                selected_elements: Arc::new([]),
            }),
        }
    }

    // Create a builder for modifying the state
    pub fn builder(&self) -> EditorStateBuilder {
        EditorStateBuilder {
            data: (*self.shared).clone(),
        }
    }

    // Legacy builder methods that use the new builder pattern internally
    // These maintain backward compatibility with existing code
    
    // Builder method to update the active tool
    pub fn with_active_tool(self, tool: Option<ToolType>) -> Self {
        self.builder()
            .with_active_tool(tool)
            .build()
    }

    // Builder method to update selected elements
    pub fn with_selected_elements(self, elements: Vec<ElementType>) -> Self {
        self.builder()
            .with_selected_elements(elements)
            .build()
    }

    // Convenience method to set a single element (or none)
    pub fn with_selected_element(self, element: Option<ElementType>) -> Self {
        match element {
            Some(elem) => self.builder()
                .with_selected_elements(vec![elem])
                .build(),
            None => self.builder()
                .with_selected_elements(vec![])
                .build(),
        }
    }

    // Getters for state values
    pub fn active_tool(&self) -> Option<&ToolType> {
        self.shared.active_tool.as_ref().map(|arc| arc.deref())
    }

    // Get all selected elements
    pub fn selected_elements(&self) -> &[ElementType] {
        &self.shared.selected_elements
    }

    // Convenience method to get the first selected element (if any)
    pub fn selected_element(&self) -> Option<&ElementType> {
        self.shared.selected_elements.first()
    }
}

impl EditorStateBuilder {
    // Builder method to update the active tool
    pub fn with_active_tool(mut self, tool: Option<ToolType>) -> Self {
        self.data.active_tool = tool.map(Arc::new);
        self
    }

    // Builder method to update selected elements
    pub fn with_selected_elements(mut self, elements: Vec<ElementType>) -> Self {
        self.data.selected_elements = elements.into();
        self
    }

    // Build the final EditorState
    pub fn build(self) -> EditorState {
        EditorState {
            shared: Arc::new(self.data),
        }
    }
}
