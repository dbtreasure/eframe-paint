use crate::stroke::StrokeRef;
use crate::image::{ImageRef, Image};
use crate::widgets::resize_handle::Corner;
use crate::element::{ElementType, Element, factory};
use crate::renderer::Renderer;
use crate::state::EditorModel;
use egui;
use log;

// Helper function to resize image data
fn resize_image_data(original_data: &[u8], original_width: usize, original_height: usize, 
                    new_width: usize, new_height: usize) -> Vec<u8> {
    // If dimensions match, return the original data
    if original_width == new_width && original_height == new_height {
        return original_data.to_vec();
    }
    
    // Create a new buffer for the resized image
    let mut new_data = Vec::with_capacity(new_width * new_height * 4);
    
    // Simple nearest-neighbor scaling
    for y in 0..new_height {
        for x in 0..new_width {
            // Map new coordinates to original image coordinates
            let orig_x = (x * original_width) / new_width;
            let orig_y = (y * original_height) / new_height;
            
            // Calculate pixel index in original data
            let orig_idx = (orig_y * original_width + orig_x) * 4;
            
            // Copy the pixel if it's within bounds
            if orig_idx + 3 < original_data.len() {
                new_data.push(original_data[orig_idx]);     // R
                new_data.push(original_data[orig_idx + 1]); // G
                new_data.push(original_data[orig_idx + 2]); // B
                new_data.push(original_data[orig_idx + 3]); // A
            } else {
                // Use a default color (blue) if out of bounds
                new_data.push(0);   // R
                new_data.push(0);   // G
                new_data.push(255); // B
                new_data.push(255); // A
            }
        }
    }
    
    new_data
}

#[derive(Clone, Debug)]
pub enum Command {
    AddStroke(StrokeRef),
    AddImage(ImageRef),
    ResizeElement {
        element_id: usize,
        corner: Corner,
        new_position: egui::Pos2,
        original_element: Option<ElementType>,
    },
    MoveElement {
        element_id: usize,
        delta: egui::Vec2,
        original_element: Option<ElementType>,
    },
    SelectElement(usize),
    DeselectElement(usize),
    ClearSelection,
    ToggleSelection(usize),
}

impl Command {
    // Add a new method to handle texture invalidation after command execution
    pub fn invalidate_textures(&self, renderer: &mut Renderer) {
        match self {
            Command::AddStroke(stroke) => {
                log::info!("🧹 Invalidating texture for new stroke {}", stroke.id());
                // Using ID-based approach to avoid ElementType variant matching
                renderer.clear_element_state(stroke.id());
            },
            Command::AddImage(image) => {
                log::info!("🧹 Invalidating texture for new image {}", image.id());
                // Using ID-based approach to avoid ElementType variant matching
                renderer.clear_element_state(image.id());
            },
            Command::ResizeElement { element_id, corner: _, new_position: _, original_element } => {
                log::info!("🧹 Invalidating texture for resized element {}", element_id);
                
                // First clear by ID to remove any stale textures
                renderer.clear_element_state(*element_id);
                
                // Also handle the element if we have it
                if let Some(element) = original_element {
                    renderer.handle_element_update(element);
                    
                    // For stroke elements, perform extra invalidation
                    if element.element_type() == "stroke" {
                        log::info!("🧹 Extra invalidation for stroke element {}", element_id);
                        renderer.clear_texture_for_element(*element_id);
                    }
                }
                
                // For resize operations, always reset all state to be safe
                renderer.clear_all_element_state();
            },
            Command::MoveElement { element_id, delta: _, original_element } => {
                log::info!("🧹 Invalidating texture for moved element {}", element_id);
                
                // First clear by ID to remove any stale textures
                renderer.clear_element_state(*element_id);
                
                // Also handle the element if we have it
                if let Some(element) = original_element {
                    renderer.handle_element_update(element);
                    
                    // For stroke elements, perform extra invalidation
                    if element.element_type() == "stroke" {
                        log::info!("🧹 Extra invalidation for stroke element {}", element_id);
                        renderer.clear_texture_for_element(*element_id);
                        
                        // Reset renderer state more completely for stroke moves to fix duplicate rendering
                        log::info!("🧹 Performing full renderer reset for stroke element {}", element_id);
                        renderer.reset_state();
                    } else {
                        // For non-stroke elements, still clear all element state
                        renderer.clear_all_element_state();
                    }
                } else {
                    // If we don't have the original element, clear all to be safe
                    renderer.clear_all_element_state();
                }
            },
            Command::SelectElement(_) | Command::DeselectElement(_) | Command::ClearSelection | Command::ToggleSelection(_) => {
                // Selection commands don't need texture invalidation
                // But we should request a repaint to ensure the UI updates
                renderer.get_ctx().request_repaint();
            }
        }
        
        // Request a repaint to ensure changes are visible
        renderer.get_ctx().request_repaint();
    }

    pub fn apply_to_editor_model(&self, editor_model: &mut EditorModel) {
        match self {
            Command::AddStroke(stroke) => {
                editor_model.add_stroke(stroke.clone());
            },
            Command::AddImage(image) => {
                editor_model.add_image(image.clone());
            },
            Command::ResizeElement { element_id, corner, new_position, original_element } => {
                log::info!("💻 Executing ResizeElement command for element {}", element_id);
                
                // Compute the new rectangle based on the corner and new position
                let element = if let Some(elem) = original_element.clone() {
                    // If we have the original element, use it
                    let original_rect = crate::element::compute_element_rect(&elem);
                    let new_rect = Renderer::compute_resized_rect(original_rect, *corner, *new_position);
                    
                    log::info!("📐 Resizing element {} from {:?} to {:?}", element_id, original_rect, new_rect);
                    
                    // Take ownership of the element
                    if let Ok(mut elem) = editor_model.resize_element(*element_id, new_rect) {
                        log::info!("✅ Successfully resized element {} using ownership transfer", element_id);
                        Some(elem)
                    } else {
                        // Resize failed, try legacy approach for backward compatibility
                        log::warn!("⚠️ Direct ownership resize failed, using backward compatibility code");
                        
                        // If the element is an image, we may need to resize the pixel data
                        if elem.element_type() == "image" {
                            if let Some(original_img) = editor_model.find_element_by_id(*element_id) {
                                if original_img.element_type() == "image" {
                                    // Remove the element so we can re-add it with the new data
                                    if let Some(removed_elem) = editor_model.remove_element_by_id(*element_id) {
                                        // Create a new image element with the resized data using factory
                                        // This is a simplified example as we don't have direct access to image data
                                        let resized_elem = factory::create_image(
                                            *element_id,
                                            vec![255, 0, 0, 255], // Placeholder data
                                            new_rect.size(),
                                            new_rect.min
                                        );
                                        
                                        // Add the resized element back
                                        editor_model.add_element(resized_elem.clone());
                                        Some(resized_elem)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                } else {
                    // Try to get the element from the editor model
                    if let Some(elem) = editor_model.find_element_by_id(*element_id).cloned() {
                        let original_rect = crate::element::compute_element_rect(&elem);
                        let new_rect = Renderer::compute_resized_rect(original_rect, *corner, *new_position);
                        
                        log::info!("📐 Resizing element {} from {:?} to {:?}", element_id, original_rect, new_rect);
                        
                        // Use the resize_element method which handles ownership transfer
                        if let Ok(()) = editor_model.resize_element(*element_id, new_rect) {
                            log::info!("✅ Successfully resized element {}", element_id);
                            editor_model.find_element_by_id(*element_id).cloned()
                        } else {
                            log::error!("❌ Resize operation failed for element {}", element_id);
                            None
                        }
                    } else {
                        log::error!("❌ Original element {} not found", element_id);
                        None
                    }
                };
                
                // The element instance is discarded, but that's okay because the editor_model now has ownership
                // of the modified element if the operation was successful
                
                editor_model.mark_modified();
            },
            Command::MoveElement { element_id, delta, original_element } => {
                log::info!("Executing MoveElement command: element={}, delta={:?}", element_id, delta);
                
                // Try to move the element using the ownership transfer pattern
                if let Err(e) = editor_model.translate_element(*element_id, *delta) {
                    log::warn!("❗ Primary translate_element failed: {}", e);
                    
                    // Fallback to using the original element if provided
                    if let Some(original) = original_element {
                        log::info!("Using provided original element for translation");
                        
                        // Create a new element with the same ID but translated
                        let id = original.id();
                        
                        // Remove the old element
                        if editor_model.remove_element_by_id(id).is_some() {
                            // Create a new element based on the type
                            let mut new_element = original.clone();
                            
                            // Translate it
                            if let Ok(()) = new_element.translate(*delta) {
                                // Add it back to the model
                                editor_model.add_element(new_element);
                                log::info!("✅ Successfully translated element {} using original", id);
                            } else {
                                log::error!("❌ Failed to translate original element {}", id);
                            }
                        } else {
                            log::error!("❌ Failed to remove element {} for replacement", id);
                        }
                    } else {
                        log::error!("❌ No original element provided and translate_element failed for {}", element_id);
                    }
                } else {
                    log::info!("✅ Successfully translated element {} using ownership transfer", element_id);
                }
                
                // Explicitly mark the model as modified
                editor_model.mark_modified();
            },
            Command::SelectElement(element_id) => {
                log::info!("Executing SelectElement command for element {}", element_id);
                editor_model.select_element(*element_id);
            },
            Command::DeselectElement(element_id) => {
                log::info!("Executing DeselectElement command for element {}", element_id);
                editor_model.deselect_element(*element_id);
            },
            Command::ClearSelection => {
                log::info!("Executing ClearSelection command");
                editor_model.clear_selection();
            },
            Command::ToggleSelection(element_id) => {
                log::info!("Executing ToggleSelection command for element {}", element_id);
                editor_model.toggle_selection(*element_id);
            },
        }
    }

    pub fn unapply_from_editor_model(&self, editor_model: &mut EditorModel) {
        match self {
            Command::AddStroke(stroke) => {
                // Remove the stroke from the editor_model
                editor_model.remove_element_by_id(stroke.id());
                editor_model.mark_modified();
            },
            Command::AddImage(image) => {
                // Remove the image from the editor_model
                editor_model.remove_element_by_id(image.id());
                editor_model.mark_modified();
            },
            Command::ResizeElement { element_id, corner: _, new_position: _, original_element } => {
                // Restore the original element if provided
                if let Some(original) = original_element {
                    match original {
                        ElementType::Image(img) => {
                            editor_model.replace_image_by_id(*element_id, img.clone());
                        },
                        ElementType::Stroke(stroke) => {
                            editor_model.replace_stroke_by_id(*element_id, stroke.clone());
                        }
                    }
                }
                
                editor_model.mark_modified();
            },
            Command::MoveElement { element_id, delta, original_element } => {
                // Restore the original element if provided
                if let Some(original) = original_element {
                    match original {
                        ElementType::Image(img) => {
                            editor_model.replace_image_by_id(*element_id, img.clone());
                        },
                        ElementType::Stroke(stroke) => {
                            editor_model.replace_stroke_by_id(*element_id, stroke.clone());
                        }
                    }
                } else {
                    // Otherwise, move the element back by negating the delta
                    if let Err(e) = editor_model.translate_element(*element_id, -*delta) {
                        log::error!("Failed to undo translation for element {}: {}", element_id, e);
                    }
                }
                
                // No need to call mark_modified() here as it's done in translate_element or replace methods
            },
            Command::SelectElement(element_id) => {
                // Undo a selection by deselecting the element
                editor_model.deselect_element(*element_id);
            },
            Command::DeselectElement(element_id) => {
                // Undo a deselection by selecting the element
                editor_model.select_element(*element_id);
            },
            Command::ClearSelection => {
                // This is harder to undo properly without storing the previous selection
                // For now, we'll just log a warning
                log::warn!("Cannot properly undo ClearSelection without storing previous selection");
            },
            Command::ToggleSelection(element_id) => {
                // Undo a toggle by toggling again
                editor_model.toggle_selection(*element_id);
            },
        }
    }
}

pub struct CommandHistory {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Execute a command on an EditorModel
    pub fn execute(&mut self, command: Command, editor_model: &mut EditorModel) {
        // Clear the redo stack when a new command is executed
        self.redo_stack.clear();
        
        // Apply the command to the editor_model using the new method
        command.apply_to_editor_model(editor_model);
        
        // Add the command to the undo stack
        self.undo_stack.push(command);
    }
    
    /// Undo a command on an EditorModel
    pub fn undo(&mut self, editor_model: &mut EditorModel) {
        if let Some(command) = self.undo_stack.pop() {
            // Unapply the command from the editor_model using the new method
            command.unapply_from_editor_model(editor_model);
            
            // Add the command to the redo stack
            self.redo_stack.push(command);
        }
    }
    
    /// Redo a command on an EditorModel
    pub fn redo(&mut self, editor_model: &mut EditorModel) {
        if let Some(command) = self.redo_stack.pop() {
            // Apply the command to the editor_model using the new method
            command.apply_to_editor_model(editor_model);
            
            // Add the command to the undo stack
            self.undo_stack.push(command);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_stack(&self) -> &[Command] {
        &self.undo_stack
    }

    pub fn redo_stack(&self) -> &[Command] {
        &self.redo_stack
    }
} 