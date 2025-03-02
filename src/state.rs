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
    
    /// Update active tool using a closure pattern
    /// 
    /// The closure receives the current tool (if any) and should return the new tool (if any).
    /// Version is only incremented if the tool actually changes.
    /// 
    /// # Example
    /// ```
    /// # use eframe_paint::state::EditorState;
    /// # use eframe_paint::tools::ToolType;
    /// # let state = EditorState::new();
    /// // Example: Set a specific tool
    /// let new_state = state.update_tool(|_| Some(ToolType::Selection(eframe_paint::tools::new_selection_tool())));
    /// ```
    pub fn update_tool<F>(&self, f: F) -> Self 
    where
        F: FnOnce(Option<&ToolType>) -> Option<ToolType>
    {
        self.builder()
            .with_active_tool(f(self.active_tool()))
            .build()
    }

    /// Modify selection using a closure pattern
    /// 
    /// The closure receives the current selection slice and should return a new selection.
    /// Version is only incremented if the selection actually changes.
    /// 
    /// # Example
    /// ```
    /// # use eframe_paint::state::EditorState;
    /// # let state = EditorState::new();
    /// // Example: Clear the selection
    /// let new_state = state.update_selection(|_| vec![]);
    /// ```
    pub fn update_selection<F>(&self, f: F) -> Self 
    where
        F: FnOnce(&[ElementType]) -> Vec<ElementType>
    {
        self.builder()
            .with_selected_elements(f(self.selected_elements()))
            .build()
    }

    /// Take ownership of the active tool, removing it from the state
    /// This is useful when you need to modify the tool in a way that can't be done through a reference
    /// Returns a tuple of (new_state, tool) where new_state has the tool removed
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_tracking() {
        // Create initial state
        let state = EditorState::new();
        let initial_version = state.version();
        
        // No change should not increment version
        let same_state = state.update_tool(|t| t.cloned());
        assert_eq!(same_state.version(), initial_version);
        
        // Changing tool should increment version
        let tool_changed = state.update_tool(|_| Some(ToolType::DrawStroke(crate::tools::new_draw_stroke_tool())));
        assert_eq!(tool_changed.version(), initial_version + 1);
        
        // Changing to same tool type should still increment version because it's a new instance
        // This is expected because we can't easily compare the internal state of tools
        let same_tool_again = tool_changed.update_tool(|_| Some(ToolType::DrawStroke(crate::tools::new_draw_stroke_tool())));
        assert_eq!(same_tool_again.version(), initial_version + 2);
        
        // Changing selection should increment version
        let with_selection = state.update_selection(|_| vec![]);
        // The empty selection is the same as the initial state's empty selection, so version shouldn't change
        assert_eq!(with_selection.version(), initial_version);
        
        // Add an element to the selection to test version change
        let stroke = crate::stroke::Stroke::new_ref(
            egui::Color32::RED, 
            1.0, 
            vec![egui::Pos2::new(0.0, 0.0), egui::Pos2::new(10.0, 10.0)]
        );
        let with_element = state.update_selection(|_| vec![ElementType::Stroke(stroke)]);
        assert_eq!(with_element.version(), initial_version + 1);
        
        // Changing back to empty selection should increment version
        let back_to_empty = with_element.update_selection(|_| vec![]);
        assert_eq!(back_to_empty.version(), initial_version + 2);
        
        // Multiple changes should increment version multiple times
        let multi_changed = state
            .update_tool(|_| Some(ToolType::DrawStroke(crate::tools::new_draw_stroke_tool())))
            .update_selection(|_| vec![/* some elements */]);
        assert_eq!(multi_changed.version(), initial_version + 1); // Only tool changed, empty selection is the same
    }
    
    #[test]
    fn test_helper_methods() {
        let state = EditorState::new();
        
        // Test update_tool
        let with_tool = state.update_tool(|_| Some(ToolType::DrawStroke(crate::tools::new_draw_stroke_tool())));
        assert!(matches!(with_tool.active_tool(), Some(ToolType::DrawStroke(_))));
        
        // Test update_selection (empty to non-empty)
        // For this test we'd need actual elements, but we can at least test the length changes
        let with_selection = state.update_selection(|sel| {
            assert_eq!(sel.len(), 0); // Initial state has empty selection
            vec![] // Return empty vec for now
        });
        assert_eq!(with_selection.selected_elements().len(), 0);
        
        // Test that update_tool preserves selection
        // Create a simple stroke for testing
        let stroke = crate::stroke::Stroke::new_ref(
            egui::Color32::RED, 
            1.0, 
            vec![egui::Pos2::new(0.0, 0.0), egui::Pos2::new(10.0, 10.0)]
        );
        let element = ElementType::Stroke(stroke);
        
        let with_element = state.with_selected_element(Some(element));
        let with_element_and_tool = with_element.update_tool(|_| Some(ToolType::DrawStroke(crate::tools::new_draw_stroke_tool())));
        
        assert!(matches!(with_element_and_tool.active_tool(), Some(ToolType::DrawStroke(_))));
        assert_eq!(with_element_and_tool.selected_elements().len(), 1);
    }
}
