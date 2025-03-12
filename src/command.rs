use crate::stroke::StrokeRef;
use crate::image::{ImageRef, Image};
use crate::widgets::resize_handle::Corner;
use crate::element::ElementType;
use crate::renderer::Renderer;
use crate::state::EditorModel;
use egui;
use log;
use std::sync::Arc;

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
                let element = ElementType::Stroke(stroke.clone());
                log::info!("🧹 Invalidating texture for new stroke {}", stroke.id());
                renderer.handle_element_update(&element);
            },
            Command::AddImage(image) => {
                let element = ElementType::Image(image.clone());
                log::info!("🧹 Invalidating texture for new image {}", image.id());
                renderer.handle_element_update(&element);
            },
            Command::ResizeElement { element_id, corner: _, new_position: _, original_element } => {
                log::info!("🧹 Invalidating texture for resized element {}", element_id);
                
                // First clear by ID to remove any stale textures
                renderer.clear_element_state(*element_id);
                
                // Also handle the element if we have it
                if let Some(element) = original_element {
                    renderer.handle_element_update(element);
                }
                
                // For resize operations, ensure we specifically invalidate for strokes
                // since they may not directly mutate their underlying data
                if let Some(ElementType::Stroke(_)) = original_element {
                    log::info!("🧹 Extra invalidation for stroke element {}", element_id);
                    renderer.clear_texture_for_element(*element_id);
                }
                
                // For resize operations, always reset all state to be safe
                renderer.clear_all_element_state();
            },
            Command::MoveElement { element_id, delta: _, original_element: _original_element } => {
                log::info!("🧹 Invalidating texture for moved element {}", element_id);
                
                // First clear by ID to remove any stale textures
                renderer.clear_element_state(*element_id);
                
                // We don't need to use _original_element here as we're just invalidating the texture
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
                
                // Get the original element if not provided
                let original = original_element.clone()
                    .or_else(|| editor_model.find_element_by_id(*element_id).cloned());
                
                if let Some(element) = original {
                    // Get the original rect
                    let original_rect = crate::element::compute_element_rect(&element);
                    
                    // Compute the new rectangle based on the corner and new position
                    let new_rect = Renderer::compute_resized_rect(original_rect, *corner, *new_position);
                    
                    log::info!("📐 Resizing element {} from {:?} to {:?}", 
                              element_id, original_rect, new_rect);
                    
                    // Try different approach for images to ensure proper resize
                    match element {
                        ElementType::Image(image) => {
                            // For images, we'll create a new copy with the new rect
                            log::info!("🖼️ Image resize: creating new image with updated rect");
                            
                            // Log image data sizes to detect any issues
                            let image_data = image.data();
                            let data_size = image_data.len();
                            let width = image.size().x as usize;
                            let height = image.size().y as usize;
                            let expected_bytes = width * height * 4;
                            
                            log::info!("🔍 Image data check: original size {}x{}, data len: {}, expected: {}", 
                                      width, height, data_size, expected_bytes);
                            
                            // Handle different image data cases to avoid the red square
                            if data_size == expected_bytes {
                                // Data size matches dimensions, create a properly scaled image
                                log::info!("✅ Image data size matches dimensions, creating scaled copy");
                                
                                // Resize the image data to match the new dimensions
                                let new_width = new_rect.width() as usize;
                                let new_height = new_rect.height() as usize;
                                
                                log::info!("📏 Resizing image data from {}x{} to {}x{}", 
                                         width, height, new_width, new_height);
                                
                                let resized_data = resize_image_data(
                                    image_data, 
                                    width, 
                                    height, 
                                    new_width, 
                                    new_height
                                );
                                
                                log::info!("📏 Resized data length: {} (expected: {})", 
                                         resized_data.len(), new_width * new_height * 4);
                                
                                // Create a new image with updated position and size, preserving the ID and data
                                let image_ref = Image::new_ref_with_id(
                                    image.id(),
                                    resized_data,
                                    new_rect.size(),
                                    new_rect.min
                                );
                                
                                // Replace the image at the same position
                                let replaced = editor_model.replace_image_by_id(*element_id, image_ref);
                                log::info!("🖼️ Image replacement {}", if replaced { "SUCCEEDED" } else { "FAILED" });
                            } else {
                                // Data size doesn't match, create a blue placeholder to avoid the red square
                                log::warn!("⚠️ Image data size mismatch, creating blue placeholder");
                                
                                // Create a blue placeholder image
                                let new_width = (new_rect.width() as usize).max(1);
                                let new_height = (new_rect.height() as usize).max(1);
                                let pixels = new_width * new_height;
                                
                                let mut blue_data = Vec::with_capacity(pixels * 4);
                                for _ in 0..pixels {
                                    blue_data.push(0);     // R
                                    blue_data.push(0);     // G
                                    blue_data.push(255);   // B
                                    blue_data.push(255);   // A
                                }
                                
                                // Create a blue placeholder image with same ID
                                let placeholder = Image::new_ref_with_id(
                                    image.id(),
                                    blue_data,
                                    new_rect.size(),
                                    new_rect.min
                                );
                                
                                // Replace with the blue placeholder
                                let replaced = editor_model.replace_image_by_id(*element_id, placeholder);
                                log::info!("🖼️ Placeholder image replacement {}", 
                                          if replaced { "SUCCEEDED" } else { "FAILED" });
                            }
                        },
                        ElementType::Stroke(stroke) => {
                            // For strokes, we'll try the standard resize approach first
                            let mut resize_successful = false;
                            
                            // Get a mutable reference to the element
                            if let Some(mut element_mut) = editor_model.get_element_mut(*element_id) {
                                let resize_result = element_mut.resize(original_rect, new_rect);
                                
                                match resize_result {
                                    Ok(_) => {
                                        log::info!("✅ Successfully resized stroke {} using direct mutation", element_id);
                                        resize_successful = true;
                                    },
                                    Err(e) => {
                                        log::error!("❌ Direct mutation failed for stroke {}: {}", element_id, e);
                                        // Will continue to fallback approach
                                    }
                                }
                            }
                            
                            // If direct mutation failed, use fallback approach similar to images
                            if !resize_successful {
                                log::info!("🔄 Using fallback approach for stroke resize");
                                
                                // Use the resize_stroke function to create a new resized stroke
                                let resized_stroke = crate::stroke::resize_stroke(&stroke, original_rect, new_rect);
                                
                                // Replace the stroke in the document
                                let replaced = editor_model.replace_stroke_by_id(*element_id, resized_stroke);
                                log::info!("✏️ Stroke replacement {}", if replaced { "SUCCEEDED" } else { "FAILED" });
                            }
                        }
                    }
                } else {
                    log::error!("❌ Original element {} not found", element_id);
                }
                
                editor_model.mark_modified();
            },
            Command::MoveElement { element_id, delta, original_element: _original_element } => {
                log::info!("Executing MoveElement command: element={}, delta={:?}", element_id, delta);
                
                // Find the element in the editor_model
                let element_clone = editor_model.find_element_by_id(*element_id).cloned();
                
                if let Some(element) = element_clone {
                    match element {
                        ElementType::Image(img) => {
                            // For images, create a new image with the updated position
                            let new_position = img.position() + *delta;
                            let new_image = crate::image::Image::new_ref_with_id(
                                img.id(),
                                img.data().to_vec(),
                                img.size(),
                                new_position
                            );
                            
                            // Replace the image in the editor_model
                            editor_model.replace_image_by_id(*element_id, new_image);
                        },
                        ElementType::Stroke(stroke) => {
                            // For strokes, try the in-place translation first
                            let mut success = false;
                            
                            // Get a mutable reference to the element
                            if let Some(mut element_mut) = editor_model.get_element_mut(*element_id) {
                                if let Err(e) = element_mut.translate(*delta) {
                                    log::error!("Failed to translate stroke in-place: {}", e);
                                } else {
                                    success = true;
                                }
                            }
                            
                            // If direct mutation failed, use fallback approach
                            if !success {
                                // Fallback: create a new stroke with translated points
                                let new_stroke = stroke.translate(*delta);
                                editor_model.replace_stroke_by_id(*element_id, Arc::new(new_stroke));
                            }
                        }
                    }
                }
                
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
            Command::MoveElement { element_id, delta, original_element: _original_element } => {
                // Restore the original element if provided
                if let Some(original) = _original_element {
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
                    // Clone the element first to avoid borrowing conflicts
                    let element_clone = editor_model.find_element_by_id(*element_id).cloned();
                    
                    if let Some(element) = element_clone {
                        match element {
                            ElementType::Image(img) => {
                                let new_position = img.position() - *delta;
                                let new_image = crate::image::Image::new_ref_with_id(
                                    img.id(),
                                    img.data().to_vec(),
                                    img.size(),
                                    new_position
                                );
                                
                                editor_model.replace_image_by_id(*element_id, new_image);
                            },
                            ElementType::Stroke(stroke) => {
                                if let Some(mut element_mut) = editor_model.get_element_mut(*element_id) {
                                    if let Err(e) = element_mut.translate(-*delta) {
                                        log::error!("Failed to translate stroke in-place during undo: {}", e);
                                        
                                        // Fallback: create a new stroke with translated points
                                        let new_stroke = stroke.translate(-*delta);
                                        editor_model.replace_stroke_by_id(*element_id, Arc::new(new_stroke));
                                    }
                                }
                            }
                        }
                    }
                }
                
                editor_model.mark_modified();
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