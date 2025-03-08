use crate::stroke::StrokeRef;
use crate::document::Document;
use crate::image::{ImageRef, Image};
use crate::widgets::resize_handle::Corner;
use crate::state::ElementType;
use crate::renderer::Renderer;
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
}

impl Command {
    pub fn apply(&self, document: &mut Document) {
        match self {
            Command::AddStroke(stroke) => {
                document.add_stroke(stroke.clone());
            },
            Command::AddImage(image) => {
                document.add_image(image.clone());
            },
            Command::ResizeElement { element_id, corner, new_position, original_element } => {
                log::info!("💻 Executing ResizeElement command for element {}", element_id);
                
                // Get the original element if not provided
                let original = original_element.clone()
                    .or_else(|| document.find_element_by_id(*element_id));
                
                if let Some(element) = original {
                    // Get the original rect
                    let original_rect = crate::geometry::hit_testing::compute_element_rect(&element);
                    
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
                                let replaced = document.replace_image_by_id(*element_id, image_ref);
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
                                let replaced = document.replace_image_by_id(*element_id, placeholder);
                                log::info!("🖼️ Placeholder image replacement {}", 
                                          if replaced { "SUCCEEDED" } else { "FAILED" });
                            }
                        },
                        ElementType::Stroke(_stroke) => {
                            // For strokes, we'll try the standard resize approach
                            if let Some(mut element_mut) = document.get_element_mut(*element_id) {
                                let resize_result = element_mut.resize(original_rect, new_rect);
                                
                                match resize_result {
                                    Ok(_) => {
                                        log::info!("✅ Successfully resized stroke {}", element_id);
                                    },
                                    Err(e) => {
                                        log::error!("❌ Failed to resize stroke {}: {}", element_id, e);
                                    }
                                }
                            } else {
                                log::error!("❌ Could not get mutable reference to stroke {}", element_id);
                            }
                        }
                    }
                } else {
                    log::error!("❌ Original element {} not found", element_id);
                }
                
                // Always mark document as modified multiple times to force redraw
                for _ in 0..5 {
                    document.mark_modified();
                }
            },
            Command::MoveElement { element_id, delta, original_element: _ } => {
                log::info!("Executing MoveElement command: element={}, delta={:?}", element_id, delta);
                
                // Get mutable reference to the element
                if let Some(mut element) = document.get_element_mut(*element_id) {
                    // Translate in-place
                    if let Err(e) = element.translate(*delta) {
                        log::error!("Failed to translate element: {}", e);
                    }
                }
            }
        }
        
        // Ensure document is marked as modified
        document.mark_modified();
    }

    pub fn unapply(&self, document: &mut Document) {
        match self {
            Command::AddStroke(_) => {
                document.remove_last_stroke();
            }
            Command::AddImage(_) => {
                document.remove_last_image();
            }
            Command::ResizeElement { element_id, corner: _, new_position: _, original_element } => {
                // If we have the original element, restore it
                if let Some(original) = original_element {
                    // Use the helper methods to handle different element types
                    if let Some(img) = original.as_image() {
                        document.replace_image_by_id(*element_id, img.clone());
                    } else if let Some(stroke) = original.as_stroke() {
                        document.replace_stroke_by_id(*element_id, stroke.clone());
                    }
                }
                // Note: If we don't have the original element, we can't undo the resize
                // since we don't know the original dimensions
            }
            Command::MoveElement { element_id, delta, original_element } => {
                // If we have the original element, restore it
                if let Some(original) = original_element {
                    // Use the helper methods to handle different element types
                    if let Some(img) = original.as_image() {
                        document.replace_image_by_id(*element_id, img.clone());
                    } else if let Some(stroke) = original.as_stroke() {
                        document.replace_stroke_by_id(*element_id, stroke.clone());
                    }
                } else {
                    // Otherwise, apply the inverse translation
                    if let Some(mut element) = document.get_element_mut(*element_id) {
                        // Apply inverse translation
                        let inverse_delta = egui::Vec2::new(-delta.x, -delta.y);
                        if let Err(e) = element.translate(inverse_delta) {
                            log::error!("Failed to translate element during unapply: {}", e);
                        }
                    }
                }
            }
        }
    }

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
                }
            }
        }
        
        // Request a repaint to ensure changes are visible
        renderer.get_ctx().request_repaint();
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

    pub fn execute(&mut self, command: Command, document: &mut Document) {
        // Log document state before command execution
        log::info!("Before command - Document has {} strokes and {} images", 
                 document.strokes().len(), document.images().len());
        
        // Log the command being executed
        match &command {
            Command::AddStroke(_) => log::info!("Executing AddStroke command"),
            Command::AddImage(image) => {
                log::info!("Executing AddImage command with image ID: {}, size: {:?}", 
                           image.id(), image.size());
            },
            Command::ResizeElement { element_id, corner, new_position, original_element: _ } => {
                log::info!("Executing ResizeElement command: element={}, corner={:?}, pos={:?}", 
                          element_id, corner, new_position);
            },
            Command::MoveElement { element_id, delta, original_element: _ } => {
                log::info!("Executing MoveElement command: element={}, delta={:?}", 
                          element_id, delta);
            }
        }
        
        // Apply the command to update the document
        command.apply(document);
        
        // Force document to be marked as modified regardless of what the command did
        document.mark_modified();
        
        // Log document state after command execution
        log::info!("After command - Document has {} strokes and {} images", 
                 document.strokes().len(), document.images().len());
        
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, document: &mut Document) {
        if let Some(command) = self.undo_stack.pop() {
            // Log the command being undone
            match &command {
                Command::AddStroke(_) => log::info!("Undoing AddStroke command"),
                Command::AddImage(_) => log::info!("Undoing AddImage command"),
                Command::ResizeElement { element_id, corner, new_position, original_element: _ } => {
                    log::info!("Undoing ResizeElement command: element={}, corner={:?}, pos={:?}", 
                              element_id, corner, new_position);
                },
                Command::MoveElement { element_id, delta, original_element: _ } => {
                    log::info!("Undoing MoveElement command: element={}, delta={:?}", 
                              element_id, delta);
                }
            }
            
            command.unapply(document);
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self, document: &mut Document) {
        if let Some(command) = self.redo_stack.pop() {
            // Log the command being redone
            match &command {
                Command::AddStroke(_) => log::info!("Redoing AddStroke command"),
                Command::AddImage(_) => log::info!("Redoing AddImage command"),
                Command::ResizeElement { element_id, corner, new_position, original_element: _ } => {
                    log::info!("Redoing ResizeElement command: element={}, corner={:?}, pos={:?}", 
                              element_id, corner, new_position);
                },
                Command::MoveElement { element_id, delta, original_element: _ } => {
                    log::info!("Redoing MoveElement command: element={}, delta={:?}", 
                              element_id, delta);
                }
            }
            
            command.apply(document);
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

    pub fn peek_undo(&self) -> Option<&Command> {
        self.undo_stack.last()
    }

    pub fn peek_redo(&self) -> Option<&Command> {
        self.redo_stack.last()
    }
} 