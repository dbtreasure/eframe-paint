use crate::stroke::StrokeRef;
use crate::document::Document;
use crate::image::ImageRef;
use crate::widgets::Corner;
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
            Command::ResizeElement { element_id, corner, new_position, original_element: _ } => {
                // Find the element by ID in the images first
                let mut found = false;
                
                // Check images
                for image in document.images().iter() {
                    if image.id() == *element_id {
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
                                    
                                    // Copy the pixel
                                    if src_idx + 3 < original_data.len() {
                                        resized_data.extend_from_slice(&original_data[src_idx..src_idx + 4]);
                                    } else {
                                        // Fallback if we somehow got an invalid index
                                        resized_data.extend_from_slice(&[255, 255, 255, 255]);
                                    }
                                }
                            }
                        }
                        
                        // Create a new image with the resized data
                        let new_image = crate::image::Image::new_ref(
                            resized_data,
                            new_size,
                            new_position,
                        );
                        
                        // Replace the image in the document
                        document.replace_image_by_id(original_id, new_image);
                        found = true;
                        break;
                    }
                }
                
                // If not found in images, check strokes
                if !found {
                    for stroke in document.strokes().iter() {
                        let element = ElementType::Stroke(stroke.clone());
                        // Try both the pointer ID and the stable ID
                        let stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
                        if stroke_id == *element_id || element.get_stable_id() == *element_id {
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
                            
                            // Create a new stroke with resized points
                            let new_stroke = crate::stroke::resize_stroke(stroke, original_rect, new_rect);
                            
                            // Replace the stroke in the document using the new method
                            document.replace_stroke_by_id(stroke_id, new_stroke);
                            
                            break;
                        }
                    }
                }
            },
            Command::MoveElement { element_id, delta, element_index, is_stroke, original_element: _ } => {
                let mut found = false;
                
                if *is_stroke {
                    // Find the stroke by index
                    if let Some(stroke) = document.strokes().get(*element_index) {
                        // Create a new stroke with translated points
                        let points = stroke.points().iter()
                            .map(|p| *p + *delta)
                            .collect::<Vec<_>>();
                        
                        let new_stroke = crate::stroke::Stroke::new_ref(
                            stroke.color(),
                            stroke.thickness(),
                            points,
                        );
                        
                        // Get the stroke ID for replacement
                        let stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
                        
                        // Replace the stroke in the document
                        document.replace_stroke_by_id(stroke_id, new_stroke);
                        found = true;
                    }
                    
                    // If not found by index, try to find by ID
                    if !found {
                        for stroke in document.strokes().iter() {
                            let stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
                            
                            // Create a temporary ElementType to get the stable ID
                            let temp_element = ElementType::Stroke(stroke.clone());
                            let stable_id = temp_element.get_stable_id();
                            
                            if stroke_id == *element_id || stable_id == *element_id {
                                // Create a new stroke with translated points
                                let points = stroke.points().iter()
                                    .map(|p| *p + *delta)
                                    .collect::<Vec<_>>();
                                
                                let new_stroke = crate::stroke::Stroke::new_ref(
                                    stroke.color(),
                                    stroke.thickness(),
                                    points,
                                );
                                
                                // Replace the stroke in the document using the new method
                                document.replace_stroke_by_id(stroke_id, new_stroke);
                                found = true;
                                break;
                            }
                        }
                    }
                } else {
                    // Find the image by index
                    if let Some(image) = document.images().get(*element_index) {
                        // Get the original image data and id
                        let original_id = image.id();
                        let original_data = image.data().to_vec();
                        let original_size = image.size();
                        let new_position = image.position() + *delta;
                        
                        // Create a new image with the same data but moved position
                        let new_image = crate::image::Image::new_ref(
                            original_data,
                            original_size,
                            new_position,
                        );
                        
                        // Replace the image in the document
                        document.replace_image_by_id(original_id, new_image);
                        log::debug!("Moved image at index {} with ID {}", element_index, original_id);
                        found = true;
                    }
                    
                    // If not found by index, try to find by ID
                    if !found {
                        for image in document.images().iter() {
                            if image.id() == *element_id {
                                // Get the original image data and id
                                let original_id = image.id();
                                let original_data = image.data().to_vec();
                                let original_size = image.size();
                                let new_position = image.position() + *delta;
                                
                                // Create a new image with the same data but moved position
                                let mutable_img = crate::image::MutableImage::new(
                                    original_data,
                                    original_size,
                                    new_position,
                                );
                                
                                // Convert to immutable and replace
                                let new_image = mutable_img.to_image_ref();
                                document.replace_image_by_id(original_id, new_image);
                                found = true;
                                break;
                            }
                        }
                    }
                    
                    // If still not found, check strokes as a fallback
                    if !found {
                        for stroke in document.strokes().iter() {
                            let stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
                            
                            // Create a temporary ElementType to get the stable ID
                            let temp_element = ElementType::Stroke(stroke.clone());
                            let stable_id = temp_element.get_stable_id();
                            
                            if stroke_id == *element_id || stable_id == *element_id {
                                // Create a new stroke with translated points
                                let points = stroke.points().iter()
                                    .map(|p| *p + *delta)
                                    .collect::<Vec<_>>();
                                
                                let new_stroke = crate::stroke::Stroke::new_ref(
                                    stroke.color(),
                                    stroke.thickness(),
                                    points,
                                );
                                
                                // Replace the stroke in the document using the new method
                                document.replace_stroke_by_id(stroke_id, new_stroke);
                                found = true;
                                break;
                            }
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
                // Restore the original element
                if let Some(element) = original_element {
                    match element {
                        ElementType::Stroke(stroke) => {
                            // Find the index of the stroke in the document
                            let _stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
                            for (i, doc_stroke) in document.strokes().iter().enumerate() {
                                let doc_stroke_id = std::sync::Arc::as_ptr(doc_stroke) as usize;
                                if doc_stroke_id == *element_id {
                                    // Replace with the original stroke
                                    document.strokes_mut()[i] = stroke.clone();
                                    break;
                                }
                            }
                        }
                        ElementType::Image(image) => {
                            // Find the index of the image in the document
                            let _image_id = image.id();
                            for (i, doc_image) in document.images().iter().enumerate() {
                                if doc_image.id() == *element_id {
                                    // Replace with the original image
                                    document.images_mut()[i] = image.clone();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Command::MoveElement { element_id, delta: _, element_index, is_stroke, original_element } => {
                // Restore the original element
                if let Some(element) = original_element {
                    if *is_stroke {
                        if let ElementType::Stroke(stroke) = element {
                            // Use the element_index directly if it's in bounds
                            if *element_index < document.strokes().len() {
                                document.strokes_mut()[*element_index] = stroke.clone();
                            } else {
                                // Fallback to searching by ID
                                let _stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
                                for (i, doc_stroke) in document.strokes().iter().enumerate() {
                                    let doc_stroke_id = std::sync::Arc::as_ptr(doc_stroke) as usize;
                                    if doc_stroke_id == *element_id {
                                        // Replace with the original stroke
                                        document.strokes_mut()[i] = stroke.clone();
                                        break;
                                    }
                                }
                            }
                        }
                    } else {
                        if let ElementType::Image(image) = element {
                            // Use the element_index directly if it's in bounds
                            if *element_index < document.images().len() {
                                document.images_mut()[*element_index] = image.clone();
                            } else {
                                // Fallback to searching by ID
                                let _image_id = image.id();
                                for (i, doc_image) in document.images().iter().enumerate() {
                                    if doc_image.id() == *element_id {
                                        // Replace with the original image
                                        document.images_mut()[i] = image.clone();
                                        break;
                                    }
                                }
                            }
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
            command.unapply(document);
            
            // Rebuild the document to ensure clean state
            document.rebuild();
            
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self, document: &mut Document) {
        if let Some(command) = self.redo_stack.pop() {
            command.apply(document);
            
            // Rebuild the document to ensure clean state
            document.rebuild();
            
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