use std::collections::HashSet;
use crate::tools::{ToolType, Tool};
use crate::element::{ElementType, ElementTypeMut};
use crate::stroke::StrokeRef;
use crate::image::ImageRef;
use egui;

pub type ElementId = usize;

#[derive(Clone)]
pub struct EditorModel {
    pub content: Vec<ElementType>,
    pub version: usize,
    pub selected_element_ids: HashSet<ElementId>,
    pub active_tool: ToolType,
}

impl EditorModel {
    pub fn new() -> Self {
        // Use the same approach as in PaintApp::new() for consistency
        let default_tool = ToolType::DrawStroke(crate::tools::new_draw_stroke_tool());
            
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
    
    // Tool Management methods migrated from EditorState
    
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
    
    // Selection Management methods migrated from EditorState
    
    /// Gets selected element IDs
    pub fn selected_ids(&self) -> &HashSet<ElementId> {
        &self.selected_element_ids
    }
    
    /// Gets all selected elements
    pub fn selected_elements(&self) -> Vec<ElementType> {
        self.selected_element_ids.iter()
            .filter_map(|id| self.find_element_by_id(*id).cloned())
            .collect()
    }
    
    /// Gets the first selected element (if any)
    pub fn selected_element(&self) -> Option<ElementType> {
        self.selected_element_ids.iter()
            .next()
            .and_then(|id| self.find_element_by_id(*id).cloned())
    }
    
    /// Updates the selection
    pub fn update_selection<F>(&mut self, f: F) 
    where
        F: FnOnce(&[ElementType]) -> Vec<ElementType>
    {
        // Get the current selected elements
        let current_elements = self.selected_elements();
        
        // Apply the function to get the new selection
        let new_elements = f(&current_elements);
        
        // Convert the new elements to IDs
        let new_ids: HashSet<ElementId> = new_elements.iter()
            .map(|e| e.get_stable_id())
            .collect();
        
        // Update the selection
        self.selected_element_ids = new_ids;
        
        // Mark as modified
        self.mark_modified();
    }
    
    /// Sets the selected elements
    pub fn with_selected_elements(&mut self, elements: Vec<ElementType>) {
        let ids: HashSet<ElementId> = elements.iter()
            .map(|e| e.get_stable_id())
            .collect();
        self.selected_element_ids = ids;
        self.mark_modified();
    }
    
    /// Sets a single selected element (or none)
    pub fn with_selected_element(&mut self, element: Option<ElementType>) {
        match element {
            Some(elem) => {
                let mut ids = HashSet::new();
                ids.insert(elem.get_stable_id());
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
    
    // Stroke management methods migrated from Document
    
    pub fn add_stroke(&mut self, stroke: StrokeRef) {
        self.content.push(ElementType::Stroke(stroke));
        self.mark_modified();
    }
    
    pub fn strokes(&self) -> Vec<&StrokeRef> {
        self.content.iter()
            .filter_map(|element| {
                if let ElementType::Stroke(stroke) = element {
                    Some(stroke)
                } else {
                    None
                }
            })
            .collect()
    }
    
    pub fn remove_last_stroke(&mut self) -> Option<StrokeRef> {
        // Find the index of the last stroke in the content vector
        let last_stroke_index = self.content.iter().enumerate()
            .filter_map(|(i, element)| {
                if let ElementType::Stroke(_) = element {
                    Some(i)
                } else {
                    None
                }
            })
            .last();
        
        // If a stroke was found, remove it
        if let Some(index) = last_stroke_index {
            if let ElementType::Stroke(stroke) = self.content.remove(index) {
                return Some(stroke);
            }
        }
        
        None
    }
    
    pub fn find_stroke_by_id(&self, id: usize) -> Option<&StrokeRef> {
        self.content.iter()
            .filter_map(|element| {
                if let ElementType::Stroke(stroke) = element {
                    if stroke.id() == id {
                        Some(stroke)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next()
    }
    
    pub fn replace_stroke_by_id(&mut self, id: usize, new_stroke: StrokeRef) -> bool {
        // Find the index of the stroke with the matching ID
        let mut index_to_replace = None;
        
        for (i, element) in self.content.iter().enumerate() {
            if let ElementType::Stroke(stroke) = element {
                if stroke.id() == id {
                    index_to_replace = Some(i);
                    break;
                }
            }
        }
        
        // If found, replace it at the same index
        if let Some(index) = index_to_replace {
            log::info!("Replacing stroke at index {} (ID: {})", index, id);
            
            // Replace at the same index to preserve ordering
            self.content[index] = ElementType::Stroke(new_stroke);
            
            // Mark as modified
            self.mark_modified();
            return true;
        }
        
        false
    }
    
    // Image management methods migrated from Document
    
    pub fn add_image(&mut self, image: ImageRef) {
        self.content.push(ElementType::Image(image));
        self.mark_modified();
    }
    
    pub fn images(&self) -> Vec<&ImageRef> {
        self.content.iter()
            .filter_map(|element| {
                if let ElementType::Image(image) = element {
                    Some(image)
                } else {
                    None
                }
            })
            .collect()
    }
    
    pub fn remove_last_image(&mut self) -> Option<ImageRef> {
        // Find the index of the last image in the content vector
        let last_image_index = self.content.iter().enumerate()
            .filter_map(|(i, element)| {
                if let ElementType::Image(_) = element {
                    Some(i)
                } else {
                    None
                }
            })
            .last();
        
        // If an image was found, remove it
        if let Some(index) = last_image_index {
            if let ElementType::Image(image) = self.content.remove(index) {
                return Some(image);
            }
        }
        
        None
    }
    
    pub fn find_image_by_id(&self, id: usize) -> Option<&ImageRef> {
        self.content.iter()
            .filter_map(|element| {
                if let ElementType::Image(image) = element {
                    if image.id() == id {
                        Some(image)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next()
    }
    
    pub fn replace_image_by_id(&mut self, id: usize, new_image: ImageRef) -> bool {
        // Find the index of the image with the matching ID
        let mut index_to_replace = None;
        
        for (i, element) in self.content.iter().enumerate() {
            if let ElementType::Image(image) = element {
                if image.id() == id {
                    index_to_replace = Some(i);
                    break;
                }
            }
        }
        
        // If found, replace it at the same index
        if let Some(index) = index_to_replace {
            log::info!("Replacing image ID: {}", id);
            
            // Replace at the same index to preserve ordering
            self.content[index] = ElementType::Image(new_image);
            
            // Mark document as modified
            self.mark_modified();
            
            return true;
        }
        
        log::error!("Could not find image with ID: {} to replace", id);
        false
    }
    
    // Element Management methods migrated from Document
    
    /// Find any element by ID
    pub fn find_element_by_id(&self, id: usize) -> Option<&ElementType> {
        // Changed to return a reference instead of a clone
        self.content.iter()
            .find(|element| element.get_stable_id() == id)
    }
    
    /// Check if document contains element with given ID
    pub fn contains_element(&self, id: usize) -> bool {
        self.content.iter().any(|element| element.get_stable_id() == id)
    }
    
    /// Gets mutable reference to an element
    pub fn get_element_mut(&mut self, element_id: usize) -> Option<ElementTypeMut<'_>> {
        for (i, element) in self.content.iter_mut().enumerate() {
            match element {
                ElementType::Stroke(stroke) if stroke.id() == element_id => {
                    return Some(ElementTypeMut::Stroke(stroke));
                },
                ElementType::Image(image) if image.id() == element_id => {
                    return Some(ElementTypeMut::Image(image));
                },
                _ => continue,
            }
        }
        None
    }
    
    /// Finds element at a given position
    pub fn element_at_position(&self, point: egui::Pos2) -> Option<ElementType> {
        // First check strokes (front to back)
        for element in &self.content {
            if let ElementType::Stroke(stroke) = element {
                // For simplicity, we'll check if the point is close to any line segment in the stroke
                let points = stroke.points();
                if points.len() < 2 {
                    continue;
                }

                for window in points.windows(2) {
                    let line_start = window[0];
                    let line_end = window[1];
                    
                    // Calculate distance from point to line segment
                    let distance = distance_to_line_segment(point, line_start, line_end);
                    
                    // If the distance is less than the stroke thickness plus a small margin, consider it a hit
                    if distance <= stroke.thickness() + 2.0 {
                        return Some(element.clone());
                    }
                }
            } else if let ElementType::Image(image) = element {
                let rect = image.rect();
                if rect.contains(point) {
                    return Some(element.clone());
                }
            }
        }

        // No element found at the position
        None
    }
    
    /// Gets element position in draw order
    pub fn element_draw_index(&self, id: usize) -> Option<(usize, ElementType)> {
        self.content.iter().enumerate()
            .find(|(_, element)| element.get_stable_id() == id)
            .map(|(i, element)| (i, element.clone()))
    }
    
    /// Removes an element by ID
    pub fn remove_element_by_id(&mut self, id: ElementId) -> bool {
        let index = self.content.iter().position(|e| e.get_stable_id() == id);
        if let Some(idx) = index {
            self.content.remove(idx);
            self.mark_modified();
            true
        } else {
            false
        }
    }
}

// Helper function to calculate distance from a point to a line segment
fn distance_to_line_segment(point: egui::Pos2, line_start: egui::Pos2, line_end: egui::Pos2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    
    let line_len = line_vec.length();
    if line_len == 0.0 {
        return point_vec.length();
    }
    
    let t = ((point_vec.x * line_vec.x + point_vec.y * line_vec.y) / line_len).clamp(0.0, line_len);
    let projection = line_start + (line_vec * t / line_len);
    (point - projection).length()
} 