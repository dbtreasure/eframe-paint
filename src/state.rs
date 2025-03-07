use crate::tools::{ToolType, Tool};
use crate::stroke::StrokeRef;
use crate::image::ImageRef;
use crate::element::Element;
use std::sync::Arc;
use std::ops::Deref;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum ElementType {
    Stroke(StrokeRef),
    Image(ImageRef),
}

impl ElementType {
    pub fn get_stable_id(&self) -> usize {
        match self {
            ElementType::Stroke(stroke_ref) => stroke_ref.id(),
            ElementType::Image(image_ref) => image_ref.id(),
        }
    }
    
    /// Check if this element matches the given ID
    pub fn has_id(&self, id: usize) -> bool {
        self.get_stable_id() == id
    }
    
    /// Get the element type as a string
    pub fn element_type_str(&self) -> &'static str {
        match self {
            ElementType::Stroke(_) => "stroke",
            ElementType::Image(_) => "image",
        }
    }
    
    /// Check if this element is a stroke
    pub fn is_stroke(&self) -> bool {
        matches!(self, ElementType::Stroke(_))
    }
    
    /// Check if this element is an image
    pub fn is_image(&self) -> bool {
        matches!(self, ElementType::Image(_))
    }
    
    /// Get the stroke reference if this is a stroke
    pub fn as_stroke(&self) -> Option<&StrokeRef> {
        match self {
            ElementType::Stroke(stroke) => Some(stroke),
            _ => None,
        }
    }
    
    /// Get the image reference if this is an image
    pub fn as_image(&self) -> Option<&ImageRef> {
        match self {
            ElementType::Image(image) => Some(image),
            _ => None,
        }
    }
}

impl Element for ElementType {
    fn id(&self) -> usize {
        self.get_stable_id()
    }
    
    fn element_type(&self) -> &'static str {
        self.element_type_str()
    }
    
    fn rect(&self) -> egui::Rect {
        match self {
            ElementType::Image(img) => {
                egui::Rect::from_min_size(
                    img.position(),
                    img.size()
                )
            },
            ElementType::Stroke(stroke) => {
                stroke.rect()
            }
        }
    }
    
    fn as_element_type(&self) -> ElementType {
        self.clone()
    }
}

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
        Vec::new() // Temporary empty return until Document::find_element is implemented
    }

    // Get selected element IDs
    pub fn selected_ids(&self) -> &HashSet<usize> {
        &self.shared.selected_element_ids
    }

    // Convenience method to get the first selected element (if any)
    pub fn selected_element(&self) -> Option<ElementType> {
        // This will be implemented by the caller using Document::find_element
        None // Temporary return until Document::find_element is implemented
    }
    
    /// Get current state version
    pub fn version(&self) -> u64 {
        self.shared.version
    }
    
    // Get the selection tool if the active tool is a selection tool
    pub fn selection_tool_mut(&mut self) -> Option<&mut crate::tools::UnifiedSelectionTool> {
        // This is a bit of a hack since EditorState is immutable
        // In a real implementation, we would need to modify the architecture
        // to properly handle mutable access to tools
        None
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
        let elements = self.selected_elements();
        let new_elements = f(&elements);
        let new_ids: HashSet<usize> = new_elements.iter()
            .map(|e| e.get_stable_id())
            .collect();
        
        self.builder()
            .with_selected_element_ids(new_ids)
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

    // Builder method to update selected element IDs
    pub fn with_selected_element_ids(mut self, ids: HashSet<usize>) -> Self {
        if self.data.selected_element_ids != ids {
            self.data.version = self.data.version.wrapping_add(1);
            self.data.selected_element_ids = ids;
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

