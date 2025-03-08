use crate::tools::{ToolType, Tool};
use std::sync::Arc;
use std::ops::Deref;
use std::collections::HashSet;
use crate::element::ElementType;


// Inner data structure that will be wrapped in Arc
#[derive(Clone)]
struct EditorStateData {
    active_tool: Option<Arc<ToolType>>,
    selected_element_ids: HashSet<usize>, // Replace Arc<[ElementType]> with HashSet<usize>
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
                selected_element_ids: HashSet::new(),
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
        let ids: HashSet<usize> = elements.iter()
            .map(|e| e.get_stable_id())
            .collect();
        self.builder()
            .with_selected_element_ids(ids)
            .build()
    }

    // Convenience method to set a single element (or none)
    pub fn with_selected_element(self, element: Option<ElementType>) -> Self {
        match element {
            Some(elem) => self.builder()
                .with_selected_element_ids(vec![elem.get_stable_id()].into_iter().collect())
                .build(),
            None => self.builder()
                .with_selected_element_ids(HashSet::new())
                .build(),
        }
    }

    // Getters for state values
    pub fn active_tool(&self) -> Option<&ToolType> {
        self.shared.active_tool.as_ref().map(|arc| arc.deref())
    }

    // Get all selected elements
    pub fn selected_elements(&self) -> Vec<ElementType> {
        // Convert selected IDs back to elements by looking them up in the document
        // This will be implemented by the caller using Document::find_element
        log::warn!("⚠️ EditorState::selected_elements() called but not implemented - returning empty vector");
        log::warn!("⚠️ Selected IDs: {:?}", self.selected_ids());
        Vec::new() // Temporary empty return until Document::find_element is implemented
    }

    // Get selected element IDs
    pub fn selected_ids(&self) -> &HashSet<usize> {
        log::info!("🔍 selected_ids called, current value: {:?}", self.shared.selected_element_ids);
        &self.shared.selected_element_ids
    }

    // Convenience method to get the first selected element (if any)
    pub fn selected_element(&self) -> Option<ElementType> {
        // This will be implemented by the caller using Document::find_element
        log::warn!("⚠️ EditorState::selected_element() called but not implemented - returning None");
        log::warn!("⚠️ Selected IDs: {:?}", self.selected_ids());
        None // Temporary return until Document::find_element is implemented
    }
    
    /// Get current state version
    pub fn version(&self) -> u64 {
        self.shared.version
    }
    
    pub fn update_tool<F>(&self, f: F) -> Self 
    where
        F: FnOnce(Option<&ToolType>) -> Option<ToolType>
    {
        // Get the new tool from the callback
        let new_tool = f(self.active_tool());
        
        // Check if the tool actually changed (by name)
        if self.active_tool().map(|t| t.name()) == new_tool.as_ref().map(|t| t.name()) {
            // Even if the tool name is the same, we need to apply the changes
            // Create a new state with the updated tool
            return self.builder()
                .with_active_tool(new_tool)
                .build();
        }
        
        // If we have a current tool, deactivate it
        if let Some(current_tool) = self.active_tool() {
            // We can't actually call deactivate here because we don't have a mutable reference
            // and we don't have access to the document
            // This is a limitation of the current architecture
            log::info!("Deactivating tool: {}", current_tool.name());
        }
        
        // Create a new state with the new tool
        let new_state = self.builder()
            .with_active_tool(new_tool)
            .build();
        
        // If we have a new tool, log that we're activating it
        if let Some(tool) = new_state.active_tool() {
            log::info!("Activating tool: {}", tool.name());
        }
        
        new_state
    }

    pub fn update_selection<F>(&self, f: F) -> Self 
    where
        F: FnOnce(&[ElementType]) -> Vec<ElementType>
    {
        
        // WORKAROUND: Since selected_elements() is not implemented, we'll just use the function directly
        let elements = Vec::new(); // Empty vector since selected_elements() is not implemented
        
        let new_elements = f(&elements);
   
        let new_ids: HashSet<usize> = new_elements.iter()
            .map(|e| e.get_stable_id())
            .collect();
        
        self.builder()
            .with_selected_element_ids(new_ids)
            .build()
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

    // Builder method to update selected element IDs
    pub fn with_selected_element_ids(mut self, ids: HashSet<usize>) -> Self {
        if self.data.selected_element_ids != ids {
            self.data.version = self.data.version.wrapping_add(1);
            self.data.selected_element_ids = ids;
        }
        self
    }

    // Build the final EditorState
    pub fn build(self) -> EditorState {
        EditorState {
            shared: Arc::new(self.data),
        }
    }
}

