use crate::stroke::StrokeRef;
use crate::document::Document;
use crate::image::ImageRef;
use crate::widgets::resize_handle::Corner;
use crate::state::ElementType;
use crate::renderer::Renderer;
use egui;
use log;

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
                // Get the original element if not provided
                let original = original_element.clone()
                    .or_else(|| document.find_element_by_id(*element_id));
                
                if let Some(element) = original {
                    // Get the original rect
                    let original_rect = crate::geometry::hit_testing::compute_element_rect(&element);
                    
                    // Compute the new rectangle based on the corner and new position
                    let new_rect = Renderer::compute_resized_rect(original_rect, *corner, *new_position);
                    
                    // Get mutable reference and resize in-place
                    if let Some(mut element_mut) = document.get_element_mut(*element_id) {
                        if let Err(e) = element_mut.resize(original_rect, new_rect) {
                            log::error!("Failed to resize element: {}", e);
                        }
                    }
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
                renderer.handle_element_update(&element);
            },
            Command::AddImage(image) => {
                let element = ElementType::Image(image.clone());
                renderer.handle_element_update(&element);
            },
            Command::ResizeElement { element_id, corner: _, new_position: _, original_element } => {
                if let Some(element) = original_element {
                    renderer.handle_element_update(element);
                } else {
                    // If no original element is available, clear by ID
                    renderer.clear_element_state(*element_id);
                }
            },
            Command::MoveElement { element_id, delta: _, original_element } => {
                if let Some(element) = original_element {
                    renderer.handle_element_update(element);
                } else {
                    // If no original element is available, clear by ID
                    renderer.clear_element_state(*element_id);
                }
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