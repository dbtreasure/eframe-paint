use std::collections::HashSet;
use crate::tools::{ToolType, Tool};
use crate::element::{Element, ElementType};
use crate::element::factory;
use crate::stroke::StrokeRef;
use crate::image::ImageRef;
use egui;
use log;

pub type ElementId = usize;

#[derive(Clone)]
pub struct EditorModel {
    pub elements: Vec<ElementType>,
    pub version: usize,
    pub selected_element_ids: HashSet<ElementId>,
    pub active_tool: ToolType,
}

impl EditorModel {
    pub fn new() -> Self {
        // Use the same approach as in PaintApp::new() for consistency
        let default_tool = ToolType::DrawStroke(crate::tools::new_draw_stroke_tool());
            
        Self {
            elements: Vec::new(),
            version: 0,
            selected_element_ids: HashSet::new(),
            active_tool: default_tool,
        }
    }

    pub fn mark_modified(&mut self) {
        self.version += 1;
    }

    // Element management with new ownership transfer pattern
    
    /// Add an element to the document
    pub fn add_element(&mut self, element: ElementType) {
        self.elements.push(element);
        self.mark_modified();
    }
    
    /// Take ownership of an element from the document
    pub fn take_element_by_id(&mut self, id: ElementId) -> Option<ElementType> {
        let pos = self.elements.iter().position(|e| e.id() == id)?;
        let element = self.elements.swap_remove(pos);
        self.mark_modified();
        Some(element)
    }
    
    /// Get a reference to an element by ID
    pub fn find_element_by_id(&self, id: ElementId) -> Option<&ElementType> {
        self.elements.iter().find(|e| e.id() == id)
    }
    
    /// Get a mutable reference to an element by ID
    pub fn get_element_mut(&mut self, id: ElementId) -> Option<&mut ElementType> {
        self.elements.iter_mut().find(|e| e.id() == id)
    }
    
    /// Check if document contains element with given ID
    pub fn contains_element(&self, id: ElementId) -> bool {
        self.elements.iter().any(|e| e.id() == id)
    }

    /// Translate an element by the given delta
    pub fn translate_element(&mut self, element_id: ElementId, delta: egui::Vec2) -> Result<(), String> {
        // Take ownership of the element
        let mut element = self.take_element_by_id(element_id)
            .ok_or_else(|| format!("Element with id {} not found", element_id))?;
        
        // Modify the element
        element.translate(delta)?;
        
        // Return ownership to the model
        self.add_element(element);
        
        Ok(())
    }
    
    /// Resize an element to the given rectangle
    pub fn resize_element(&mut self, element_id: ElementId, new_rect: egui::Rect) -> Result<(), String> {
        // Take ownership of the element
        let mut element = self.take_element_by_id(element_id)
            .ok_or_else(|| format!("Element with id {} not found", element_id))?;
        
        // Modify the element
        element.resize(new_rect)?;
        
        // Return ownership to the model
        self.add_element(element);
        
        Ok(())
    }
    
    /// Removes an element by ID
    pub fn remove_element_by_id(&mut self, id: ElementId) -> Option<ElementType> {
        let element = self.take_element_by_id(id);
        if element.is_some() {
            // If the element was selected, deselect it
            self.selected_element_ids.remove(&id);
        }
        element
    }
    
    // Tool Management methods
    
    /// Gets the active tool
    pub fn active_tool(&self) -> &ToolType {
        &self.active_tool
    }
    
    /// Gets a mutable reference to the active tool
    pub fn active_tool_mut(&mut self) -> &mut ToolType {
        &mut self.active_tool
    }
    
    /// Updates the active tool
    pub fn update_tool<F>(&mut self, f: F) 
    where
        F: FnOnce(&ToolType) -> ToolType
    {
        // Get the new tool from the callback
        let new_tool = f(&self.active_tool);
        
        // Check if the tool actually changed (by name)
        if self.active_tool.name() == new_tool.name() {
            // Even if the tool name is the same, we need to apply the changes
            self.active_tool = new_tool;
            return;
        }
        
        // If we have a current tool, deactivate it
        log::info!("Deactivating tool: {}", self.active_tool.name());
        
        // Set the new tool
        self.active_tool = new_tool;
        
        // Log that we're activating the new tool
        log::info!("Activating tool: {}", self.active_tool.name());
        
        // Mark as modified
        self.mark_modified();
    }
    
    // Selection Management methods
    
    /// Gets selected element IDs
    pub fn selected_ids(&self) -> &HashSet<ElementId> {
        &self.selected_element_ids
    }
    
    /// Gets all selected elements
    pub fn selected_elements(&self) -> Vec<&ElementType> {
        self.selected_element_ids.iter()
            .filter_map(|id| self.find_element_by_id(*id))
            .collect()
    }
    
    /// Gets the first selected element (if any)
    pub fn selected_element(&self) -> Option<&ElementType> {
        self.selected_element_ids.iter()
            .next()
            .and_then(|id| self.find_element_by_id(*id))
    }
    
    /// Updates the selection
    pub fn update_selection<F>(&mut self, f: F) 
    where
        F: FnOnce(&[&ElementType]) -> Vec<ElementId>
    {
        // Get the current selected elements
        let current_elements = self.selected_elements();
        
        // Apply the function to get the new selection IDs
        let new_ids = f(&current_elements);
        
        // Convert to a HashSet
        let new_ids_set: HashSet<ElementId> = new_ids.into_iter().collect();
        
        // Update the selection
        self.selected_element_ids = new_ids_set;
        
        // Mark as modified
        self.mark_modified();
    }
    
    /// Sets the selected elements by ID
    pub fn with_selected_elements_by_id(&mut self, ids: Vec<ElementId>) {
        self.selected_element_ids = ids.into_iter().collect();
        self.mark_modified();
    }
    
    /// Sets a single selected element by ID (or none)
    pub fn with_selected_element_id(&mut self, id: Option<ElementId>) {
        match id {
            Some(element_id) => {
                let mut ids = HashSet::new();
                ids.insert(element_id);
                self.selected_element_ids = ids;
            },
            None => {
                self.selected_element_ids.clear();
            }
        }
        self.mark_modified();
    }
    
    /// Selects an element by ID
    pub fn select_element(&mut self, id: ElementId) {
        self.selected_element_ids.insert(id);
        self.mark_modified();
    }
    
    /// Deselects an element by ID
    pub fn deselect_element(&mut self, id: ElementId) {
        self.selected_element_ids.remove(&id);
        self.mark_modified();
    }
    
    /// Clears all selection
    pub fn clear_selection(&mut self) {
        self.selected_element_ids.clear();
        self.mark_modified();
    }
    
    /// Toggles selection of an element by ID
    pub fn toggle_selection(&mut self, id: ElementId) {
        if self.selected_element_ids.contains(&id) {
            self.selected_element_ids.remove(&id);
        } else {
            self.selected_element_ids.insert(id);
        }
        self.mark_modified();
    }
    
    /// Gets current version
    pub fn version(&self) -> usize {
        self.version
    }
    
    /// Finds element at a given position
    pub fn element_at_position(&self, point: egui::Pos2) -> Option<&ElementType> {
        // Check all elements (front to back)
        for element in self.elements.iter().rev() {
            if element.hit_test(point) {
                return Some(element);
            }
        }
        None
    }
    
    // Legacy compatibility methods
    
    /// LEGACY: Check if an element is selected
    pub fn is_element_selected(&self, id: ElementId) -> bool {
        self.selected_element_ids.contains(&id)
    }
    
    /// LEGACY: Get all strokes in the document
    pub fn strokes(&self) -> Vec<&StrokeRef> {
        // This is a temporary function to provide backwards compatibility
        // It returns empty because we can't easily recreate the StrokeRef type from our new structure
        Vec::new()
    }
    
    /// LEGACY: Get all images in the document
    pub fn images(&self) -> Vec<&ImageRef> {
        // This is a temporary function to provide backwards compatibility
        // It returns empty because we can't easily recreate the ImageRef type from our new structure
        Vec::new()
    }
    
    /// Get element by ID 
    pub fn get_element_by_id(&self, id: ElementId) -> Option<&ElementType> {
        self.find_element_by_id(id)
    }
    
    /// Get mutable element by ID (needed for texture regeneration)
    pub fn get_element_mut_by_id(&mut self, id: ElementId) -> Option<&mut ElementType> {
        self.get_element_mut(id)
    }
    
    /// Get all element IDs
    pub fn all_element_ids(&self) -> Vec<ElementId> {
        self.elements.iter().map(|e| e.id()).collect()
    }
    
    /// LEGACY: Add a stroke
    pub fn add_stroke(&mut self, _stroke: StrokeRef) {
        // Cannot implement properly without converting from StrokeRef to our new structure
        log::warn!("add_stroke is deprecated in the new element model");
    }
    
    /// LEGACY: Add an image
    pub fn add_image(&mut self, _image: ImageRef) {
        // Cannot implement properly without converting from ImageRef to our new structure
        log::warn!("add_image is deprecated in the new element model");
    }
    
    /// LEGACY: Replace image by ID
    pub fn replace_image_by_id(&mut self, _id: ElementId, _new_image: ImageRef) -> bool {
        // Cannot implement properly without converting from ImageRef to our new structure
        log::warn!("replace_image_by_id is deprecated in the new element model - all calls should be updated to use the ownership transfer pattern");
        false
    }
    
    /// LEGACY: Replace image by ID (overload for element::image::Image for transition period)
    #[cfg(test)]
    pub fn replace_image_by_id_element(&mut self, _id: ElementId, _new_image: crate::element::image::Image) -> bool {
        log::warn!("replace_image_by_id_element is a compatibility bridge - use ownership transfer pattern instead");
        false
    }
    
    /// LEGACY: Replace stroke by ID
    pub fn replace_stroke_by_id(&mut self, _id: ElementId, _new_stroke: StrokeRef) -> bool {
        // Cannot implement properly without converting from StrokeRef to our new structure
        log::warn!("replace_stroke_by_id is deprecated in the new element model - all calls should be updated to use the ownership transfer pattern");
        false
    }
    
    /// LEGACY: Replace stroke by ID (overload for element::stroke::Stroke for transition period)
    #[cfg(test)]
    pub fn replace_stroke_by_id_element(&mut self, _id: ElementId, _new_stroke: crate::element::stroke::Stroke) -> bool {
        log::warn!("replace_stroke_by_id_element is a compatibility bridge - use ownership transfer pattern instead");
        false
    }
    
    /// LEGACY: Set selected element
    pub fn with_selected_element(&mut self, element: Option<ElementType>) {
        match element {
            Some(elem) => {
                let mut ids = HashSet::new();
                ids.insert(elem.id());
                self.selected_element_ids = ids;
            },
            None => {
                self.selected_element_ids.clear();
            }
        }
        self.mark_modified();
    }
}

// Define a test module to test the model
#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::factory;
    use egui::{Color32, Pos2, Vec2};
    
    fn create_test_model() -> EditorModel {
        let mut model = EditorModel::new();
        
        // Add a stroke
        let points = vec![Pos2::new(10.0, 10.0), Pos2::new(30.0, 30.0)];
        let stroke = factory::create_stroke(1, points, 2.0, Color32::RED);
        model.add_element(stroke);
        
        // Add an image
        let data = vec![0u8; 100]; // Dummy data
        let size = Vec2::new(100.0, 100.0);
        let position = Pos2::new(50.0, 50.0);
        let image = factory::create_image(2, data, size, position);
        model.add_element(image);
        
        model
    }
    
    #[test]
    fn test_element_management() {
        let mut model = create_test_model();
        
        // Check element count
        assert_eq!(model.elements.len(), 2);
        
        // Find element by ID
        let element = model.find_element_by_id(1);
        assert!(element.is_some());
        assert_eq!(element.unwrap().id(), 1);
        
        // Take element ownership
        let element = model.take_element_by_id(1);
        assert!(element.is_some());
        assert_eq!(element.unwrap().id(), 1);
        
        // Check element count after removal
        assert_eq!(model.elements.len(), 1);
        
        // Check that the element is gone
        assert!(model.find_element_by_id(1).is_none());
    }
    
    #[test]
    fn test_selection() {
        let mut model = create_test_model();
        
        // Initially no selection
        assert!(model.selected_element_ids.is_empty());
        
        // Select an element
        model.select_element(1);
        assert_eq!(model.selected_element_ids.len(), 1);
        assert!(model.selected_element_ids.contains(&1));
        
        // Toggle selection should deselect
        model.toggle_selection(1);
        assert!(model.selected_element_ids.is_empty());
        
        // Select multiple elements
        model.select_element(1);
        model.select_element(2);
        assert_eq!(model.selected_element_ids.len(), 2);
        
        // Clear selection
        model.clear_selection();
        assert!(model.selected_element_ids.is_empty());
    }
    
    #[test]
    fn test_translate_element() {
        let mut model = create_test_model();
        
        // Get initial position
        let element = model.find_element_by_id(1).unwrap();
        let initial_rect = element.rect();
        
        // Translate
        let delta = Vec2::new(10.0, 20.0);
        let result = model.translate_element(1, delta);
        assert!(result.is_ok());
        
        // Check new position
        let element = model.find_element_by_id(1).unwrap();
        let new_rect = element.rect();
        
        assert!(
            (new_rect.min.x - initial_rect.min.x - 10.0).abs() < 0.001 &&
            (new_rect.min.y - initial_rect.min.y - 20.0).abs() < 0.001
        );
    }
}