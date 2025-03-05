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
    selection_tool: Option<crate::tools::UnifiedSelectionTool>,
    selected_elements: Arc<[ElementType]>,
    version: u64, // Track state version for change detection
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
                selection_tool: None,
                selected_elements: Arc::new([]),
                version: 0,
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
    
    /// Get current state version
    pub fn version(&self) -> u64 {
        self.shared.version
    }
    
    pub fn update_tool<F>(&self, f: F) -> Self 
    where
        F: FnOnce(Option<&ToolType>) -> Option<ToolType>
    {
        self.builder()
            .with_active_tool(f(self.active_tool()))
            .build()
    }

   
    pub fn update_selection<F>(&self, f: F) -> Self 
    where
        F: FnOnce(&[ElementType]) -> Vec<ElementType>
    {
        self.builder()
            .with_selected_elements(f(self.selected_elements()))
            .build()
    }

    
    pub fn take_active_tool(&self) -> (Self, Option<ToolType>) {
        let mut builder = self.builder();
        let tool = builder.take_active_tool();
        let new_state = if tool.is_some() {
            // Only create a new state if we actually took a tool
            builder.build()
        } else {
            self.clone()
        };
        (new_state, tool)
    }

    /// Provides mutable access to the active tool without cloning
    /// This is used for performance-critical operations like drawing strokes
    pub fn with_tool_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Option<Arc<ToolType>>) -> R
    {
        // Create a builder to get mutable access
        let mut builder = self.builder();
        
        // Call the function with mutable access to the tool
        let result = f(&mut builder.data.active_tool);
        
        // Update the state with the modified builder
        *self = builder.build();
        
        // Return the result
        result
    }
}

impl EditorStateBuilder {
    // Builder method to update the active tool
    pub fn with_active_tool(mut self, tool: Option<ToolType>) -> Self {
        let new_tool = tool.map(Arc::new);
        let changed = match (&self.data.active_tool, &new_tool) {
            (Some(current), Some(new)) => !Arc::ptr_eq(current, new),
            (None, None) => false,
            _ => true, // One is Some and one is None
        };
        
        if changed {
            self.data.version = self.data.version.wrapping_add(1);
            self.data.active_tool = new_tool;
        }
        self
    }

    // Builder method to update selected elements
    pub fn with_selected_elements(mut self, elements: Vec<ElementType>) -> Self {
        let new_elements: Arc<[ElementType]> = elements.into();
        
        // Compare contents since Arc::ptr_eq would only check if they're the same allocation
        let elements_changed = self.data.selected_elements.len() != new_elements.len() || 
            self.data.selected_elements.iter().zip(new_elements.iter()).any(|(a, b)| !std::ptr::eq(a, b));
        
        if elements_changed {
            self.data.version = self.data.version.wrapping_add(1);
            self.data.selected_elements = new_elements;
        }
        self
    }

    // Method to take ownership of the active tool
    pub fn take_active_tool(&mut self) -> Option<ToolType> {
        self.data.active_tool.take().map(|arc| {
            Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone())
        })
    }

    // Build the final EditorState
    pub fn build(self) -> EditorState {
        EditorState {
            shared: Arc::new(self.data),
        }
    }
}

