use crate::stroke::StrokeRef;
use crate::document::Document;
use crate::image::ImageRef;
use crate::widgets::Corner;
use crate::state::ElementType;
use egui;
use log;

#[derive(Clone)]
pub enum Command {
    AddStroke(StrokeRef),
    AddImage(ImageRef),
    ResizeElement {
        element_id: usize,
        corner: Corner,
        new_position: egui::Pos2,
    },
    MoveElement {
        element_id: usize,
        delta: egui::Vec2,
    },
}

impl Command {
    pub fn apply(&self, document: &mut Document) {
        match self {
            Command::AddStroke(stroke) => document.add_stroke(stroke.clone()),
            Command::AddImage(image) => document.add_image(image.clone()),
            Command::ResizeElement { element_id, corner, new_position } => {
                // Find the element by ID in the images first
                let mut found = false;
                
                // Check images
                // First, find the index of the image with the matching ID
                let mut image_index = None;
                for (i, image) in document.images().iter().enumerate() {
                    if image.id() == *element_id {
                        image_index = Some(i);
                        found = true;
                        break;
                    }
                }
                
                if let Some(idx) = image_index {
                    // Get the image, calculate new dimensions, then replace it
                    let image = &document.images()[idx];
                    
                    // Get the current image rectangle
                    let rect = image.rect();
                    
                    // Compute the new rectangle based on the corner and new position
                    let new_rect = match corner {
                        Corner::TopLeft => egui::Rect::from_min_max(
                            *new_position,
                            rect.max,
                        ),
                        Corner::TopRight => egui::Rect::from_min_max(
                            egui::pos2(rect.min.x, new_position.y),
                            egui::pos2(new_position.x, rect.max.y),
                        ),
                        Corner::BottomLeft => egui::Rect::from_min_max(
                            egui::pos2(new_position.x, rect.min.y),
                            egui::pos2(rect.max.x, new_position.y),
                        ),
                        Corner::BottomRight => egui::Rect::from_min_max(
                            rect.min,
                            *new_position,
                        ),
                    };
                    
                    // Get the original image data and id
                    let original_id = image.id();
                    let original_data = image.data().to_vec();
                    let new_size = new_rect.size();
                    let new_position = new_rect.min;
                    
                    // Get the original dimensions
                    let original_width = rect.width() as usize;
                    let original_height = rect.height() as usize;
                    let new_width = new_size.x as usize;
                    let new_height = new_size.y as usize;

                    // Better approach: perform a proper scaling operation
                    // For simplicity, we'll use nearest-neighbor interpolation
                    let mut resized_data = Vec::with_capacity(new_width * new_height * 4);
                    
                    // We won't resize if the original dimensions are invalid
                    if original_width == 0 || original_height == 0 || original_data.len() != original_width * original_height * 4 {
                        // In this case, just create a solid color image at the new size
                        for _ in 0..new_width * new_height {
                            // White pixel (R,G,B,A)
                            resized_data.extend_from_slice(&[255, 255, 255, 255]);
                        }
                    } else {
                        // Nearest-neighbor scaling
                        for y in 0..new_height {
                            for x in 0..new_width {
                                // Map destination coordinates to source coordinates
                                let src_x = (x as f32 * original_width as f32 / new_width as f32) as usize;
                                let src_y = (y as f32 * original_height as f32 / new_height as f32) as usize;
                                
                                // Clamp to ensure we're within bounds
                                let src_x = src_x.min(original_width - 1);
                                let src_y = src_y.min(original_height - 1);
                                
                                // Calculate source pixel index
                                let src_idx = (src_y * original_width + src_x) * 4;
                                
                                // Safety check to avoid out-of-bounds access
                                if src_idx + 3 < original_data.len() {
                                    // Copy pixel (R,G,B,A)
                                    resized_data.extend_from_slice(&original_data[src_idx..src_idx + 4]);
                                } else {
                                    // Fallback to white pixel if out of bounds
                                    resized_data.extend_from_slice(&[255, 255, 255, 255]);
                                }
                            }
                        }
                    }
                    
                    // Create a new image with the same ID and proper data
                    let mutable_img = crate::image::MutableImage::new_with_id(
                        original_id,
                        resized_data,
                        new_size,
                        new_position,
                    );
                    
                    // Replace the image in the document
                    let new_image = mutable_img.to_image_ref();
                    document.images_mut()[idx] = new_image;
                }
                
                // If not found in images, check strokes
                if !found {
                    // For now, strokes aren't resizable
                    println!("Stroke resizing not fully implemented yet");
                }
            }
            Command::MoveElement { element_id, delta } => {
                // Find the element by ID in the images first
                let mut found = false;
                
                // Check images
                let mut image_index = None;
                for (i, image) in document.images().iter().enumerate() {
                    if image.id() == *element_id {
                        image_index = Some(i);
                        found = true;
                        break;
                    }
                }
                
                if let Some(idx) = image_index {
                    // Get the image, calculate new position, then replace it
                    let image = &document.images()[idx];
                    
                    // Get the current image rectangle
                    let rect = image.rect();
                    
                    // Compute the new position
                    let new_position = rect.min + *delta;
                    
                    // Get the original image data and id
                    let original_id = image.id();
                    let original_data = image.data().to_vec();
                    let size = rect.size();
                    
                    // Create a new image with the same ID and data but new position
                    let mutable_img = crate::image::MutableImage::new_with_id(
                        original_id,
                        original_data,
                        size,
                        new_position,
                    );
                    
                    // Replace the image in the document
                    let new_image = mutable_img.to_image_ref();
                    document.images_mut()[idx] = new_image;
                }
                
                // If not found in images, check strokes
                if !found {
                    // For now, strokes aren't movable
                    println!("Stroke moving not fully implemented yet");
                }
            }
        }
    }

    pub fn unapply(&self, document: &mut Document) {
        match self {
            Command::AddStroke(_) => {
                document.remove_last_stroke();
            }
            Command::AddImage(_) => {
                document.remove_last_image();
            }
            Command::ResizeElement { .. } => {
                // TODO: Implement undo for resize operations
                // This would require storing the original state
            }
            Command::MoveElement { .. } => {
                // TODO: Implement undo for move operations
                // This would require storing the original state
            }
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

    pub fn execute(&mut self, command: Command, document: &mut Document) {
        // Log the command being executed
        match &command {
            Command::AddStroke(_) => log::info!("Executing AddStroke command"),
            Command::AddImage(_) => log::info!("Executing AddImage command"),
            Command::ResizeElement { element_id, corner, new_position } => {
                log::info!("Executing ResizeElement command: element={}, corner={:?}, pos={:?}", 
                          element_id, corner, new_position);
            },
            Command::MoveElement { element_id, delta } => {
                log::info!("Executing MoveElement command: element={}, delta={:?}", 
                          element_id, delta);
            }
        }
        
        command.apply(document);
        self.undo_stack.push(command);
        self.redo_stack.clear();
        
        log::info!("Command executed. Undo stack size: {}, Redo stack size: {}", 
                 self.undo_stack.len(), self.redo_stack.len());
    }

    pub fn undo(&mut self, document: &mut Document) {
        if let Some(command) = self.undo_stack.pop() {
            command.unapply(document);
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self, document: &mut Document) {
        if let Some(command) = self.redo_stack.pop() {
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
} 