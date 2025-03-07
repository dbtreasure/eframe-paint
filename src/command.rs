use crate::stroke::StrokeRef;
use crate::document::Document;
use crate::image::{ImageRef, ImageRefExt};
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
        element_index: usize,
        is_stroke: bool,
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
                    .or_else(|| document.find_element(*element_id));
                
                if let Some(element) = original {
                    // Get the original rect
                    let original_rect = crate::geometry::hit_testing::compute_element_rect(&element);
                    
                    // Compute the new rectangle based on the corner and new position
                    let new_rect = match corner {
                        Corner::TopLeft => egui::Rect::from_min_max(
                            *new_position,
                            original_rect.max,
                        ),
                        Corner::TopRight => egui::Rect::from_min_max(
                            egui::pos2(original_rect.min.x, new_position.y),
                            egui::pos2(new_position.x, original_rect.max.y),
                        ),
                        Corner::BottomLeft => egui::Rect::from_min_max(
                            egui::pos2(new_position.x, original_rect.min.y),
                            egui::pos2(original_rect.max.x, new_position.y),
                        ),
                        Corner::BottomRight => egui::Rect::from_min_max(
                            original_rect.min,
                            *new_position,
                        ),
                    };
                    
                    match element {
                        ElementType::Image(img) => {
                            // Create a new image with the resized rect
                            let new_image = img.with_rect(new_rect);
                            
                            // Replace the image in the document
                            document.replace_image_by_id(*element_id, new_image);
                        },
                        ElementType::Stroke(stroke) => {
                            // Create a new stroke with resized points
                            let new_stroke = crate::stroke::resize_stroke(&stroke, original_rect, new_rect);
                            
                            // Replace the stroke in the document
                            document.replace_stroke_by_id(*element_id, new_stroke);
                        }
                    }
                }
            },
            Command::MoveElement { element_id, delta, element_index: _, is_stroke: _, original_element: _ } => {
                log::info!("Executing MoveElement command: element={}, delta={:?}", element_id, delta);
                
                if let Some(element) = document.find_element(*element_id) {
                    match element {
                        ElementType::Stroke(stroke) => {
                            // Create a new stroke with translated points
                            let points = stroke.points().iter()
                                .map(|p| *p + *delta)
                                .collect::<Vec<_>>();
                            
                            let new_stroke = crate::stroke::Stroke::new_ref(
                                stroke.color(),
                                stroke.thickness(),
                                points,
                            );
                            
                            // Replace the stroke in the document
                            document.replace_stroke_by_id(*element_id, new_stroke);
                        },
                        ElementType::Image(img) => {
                            // Create a new image with translated rect
                            let new_rect = img.rect().translate(*delta);
                            let new_image = img.with_rect(new_rect);
                            
                            // Replace the image in the document
                            document.replace_image_by_id(*element_id, new_image);
                        }
                    }
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
            Command::ResizeElement { element_id, corner: _, new_position: _, original_element } => {
                if let Some(original) = original_element {
                    match original {
                        ElementType::Image(img) => {
                            document.replace_image_by_id(*element_id, img.clone());
                        },
                        ElementType::Stroke(stroke) => {
                            document.replace_stroke_by_id(*element_id, stroke.clone());
                        }
                    }
                }
            }
            Command::MoveElement { element_id, delta: _, element_index: _, is_stroke: _, original_element } => {
                if let Some(original) = original_element {
                    match original {
                        ElementType::Image(img) => {
                            document.replace_image_by_id(*element_id, img.clone());
                        },
                        ElementType::Stroke(stroke) => {
                            document.replace_stroke_by_id(*element_id, stroke.clone());
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
                    // If no original element is available, invalidate by ID
                    renderer.invalidate_texture(*element_id);
                }
            },
            Command::MoveElement { element_id, delta: _, element_index: _, is_stroke: _, original_element } => {
                if let Some(element) = original_element {
                    renderer.handle_element_update(element);
                } else {
                    // If no original element is available, invalidate by ID
                    renderer.invalidate_texture(*element_id);
                }
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

    pub fn execute(&mut self, command: Command, document: &mut Document) {
        // Log the command being executed
        match &command {
            Command::AddStroke(_) => log::info!("Executing AddStroke command"),
            Command::AddImage(_) => log::info!("Executing AddImage command"),
            Command::ResizeElement { element_id, corner, new_position, original_element: _ } => {
                log::info!("Executing ResizeElement command: element={}, corner={:?}, pos={:?}", 
                          element_id, corner, new_position);
            },
            Command::MoveElement { element_id, delta, element_index, is_stroke, original_element: _ } => {
                log::info!("Executing MoveElement command: element={}, delta={:?}, index={}, is_stroke={}", 
                          element_id, delta, element_index, is_stroke);
            }
        }
        
        // Apply the command to update the document
        command.apply(document);
        
        // For ResizeElement and MoveElement commands, rebuild the document to ensure clean state
        match &command {
            Command::ResizeElement { .. } | Command::MoveElement { .. } => {
                // Rebuild the document to ensure all elements are properly recreated
                document.rebuild();
            },
            _ => {}
        }
        
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
                Command::MoveElement { element_id, delta, element_index, is_stroke, original_element: _ } => {
                    log::info!("Undoing MoveElement command: element={}, delta={:?}, index={}, is_stroke={}", 
                              element_id, delta, element_index, is_stroke);
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
                Command::MoveElement { element_id, delta, element_index, is_stroke, original_element: _ } => {
                    log::info!("Redoing MoveElement command: element={}, delta={:?}, index={}, is_stroke={}", 
                              element_id, delta, element_index, is_stroke);
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