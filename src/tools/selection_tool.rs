use egui::{Pos2, Ui, Rect};
use crate::command::Command;
use crate::document::Document;
use crate::tools::{Tool, ToolConfig};
use crate::renderer::Renderer;
use crate::element::ElementType;
use crate::element::Element;
use crate::element::{compute_element_rect, RESIZE_HANDLE_RADIUS};
use crate::state::EditorState;
use crate::widgets::Corner;
use std::any::Any;
use log::{debug, info};

// Constants
const DEFAULT_HANDLE_SIZE: f32 = 10.0;

// Config for SelectionTool
#[derive(Clone, Debug)]
pub struct SelectionToolConfig {
    pub handle_size: f32,
}

impl ToolConfig for SelectionToolConfig {
    fn tool_name(&self) -> &'static str {
        "Selection"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// New consolidated state enum for the refactored SelectionTool
#[derive(Clone)]
pub enum SelectionState {
    Idle,
    Selecting {
        element: ElementType,
        start_pos: Pos2
    },
    Resizing {
        element: ElementType,
        corner: Corner,
        original_rect: Rect,
        start_pos: Pos2,
        handle_size: f32
    },
    Dragging {
        element: ElementType,
        offset: egui::Vec2
    }
}

// Manual Debug implementation for SelectionState
impl std::fmt::Debug for SelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Selecting { start_pos, .. } => f.debug_struct("Selecting")
                .field("start_pos", start_pos)
                .finish_non_exhaustive(),
            Self::Resizing { corner, original_rect, start_pos, handle_size, .. } => f.debug_struct("Resizing")
                .field("corner", corner)
                .field("original_rect", original_rect)
                .field("start_pos", start_pos)
                .field("handle_size", handle_size)
                .finish_non_exhaustive(),
            Self::Dragging { offset, .. } => f.debug_struct("Dragging")
                .field("offset", offset)
                .finish_non_exhaustive(),
        }
    }
}

// New consolidated SelectionTool struct
#[derive(Debug, Clone)]
pub struct UnifiedSelectionTool {
    pub state: SelectionState,
    pub handle_size: f32,
    pub current_preview: Option<Rect>
}

impl UnifiedSelectionTool {
    pub fn new() -> Self {
        Self {
            state: SelectionState::Idle,
            handle_size: DEFAULT_HANDLE_SIZE,
            current_preview: None
        }
    }
    
    pub fn start_selecting(&mut self, element: ElementType, pos: Pos2) {
        info!("Starting selection at position: {:?}", pos);
        self.state = SelectionState::Selecting {
            element,
            start_pos: pos
        };
    }
    
    pub fn start_resizing(&mut self, element: ElementType, corner: Corner, original_rect: Rect, pos: Pos2) {
        info!("Starting resize at position: {:?} with corner: {:?}", pos, corner);
        self.state = SelectionState::Resizing {
            element,
            corner,
            original_rect,
            start_pos: pos,
            handle_size: self.handle_size
        };
    }
    
    pub fn start_dragging(&mut self, element: ElementType, offset: egui::Vec2) {
        info!("🔄 Starting drag operation for element ID: {}", element.id());
        
        // Get the element's current rectangle for preview
        let element_rect = compute_element_rect(&element);
        
        // Set the current state to Dragging
        self.state = SelectionState::Dragging {
            element,
            offset
        };
        
        // Initialize the preview with the current rectangle
        // This ensures we have a valid preview even before the first mouse move
        self.current_preview = Some(element_rect);
    }
    
    pub fn handle_pointer_move(&mut self, pos: Pos2, ui: &egui::Ui) -> Option<Command> {
        match &self.state {
            SelectionState::Selecting { element, start_pos } => {
                if ui.input(|i| i.pointer.primary_down()) {
                    let distance_moved = pos.distance(*start_pos);
                    let drag_threshold = 5.0;

                    if distance_moved >= drag_threshold {
                        info!("🔄 Transitioning from Selecting to Dragging state due to significant movement");

                        let element_rect = compute_element_rect(element);
                        let offset = *start_pos - element_rect.min;

                        let element_clone = element.clone();
                        self.start_dragging(element_clone, offset);

                        let new_pos = pos - offset;
                        let delta = new_pos - element_rect.min;
                        let new_rect = element_rect.translate(delta);

                        self.current_preview = Some(new_rect);
                    }
                }
                None
            },
            SelectionState::Dragging { element, offset } => {
                let new_pos = pos - *offset;
                let element_rect = compute_element_rect(element);
                let delta = new_pos - element_rect.min;

                let new_rect = element_rect.translate(delta);

                self.current_preview = Some(new_rect);
                info!("🔄 Dragging: delta={:?}", delta);

                None
            },
            _ => {
                info!("No action for pointer move in current state: {:?}", self.state);
                None
            }
        }
    }
    
    // Enhanced function to cancel any ongoing interaction and clean all state
    pub fn cancel_interaction(&mut self) {
        info!("Force canceling all interaction state");
        self.state = SelectionState::Idle;
        self.current_preview = None;
    }
    
    pub fn handle_pointer_up(&mut self, pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        match &self.state {
            SelectionState::Dragging { element, offset } => {
                // Calculate the delta from the original position
                let element_rect = compute_element_rect(element);
                let new_pos = pos - *offset;
                let delta = new_pos - element_rect.min;
                
                info!("🔄 Creating MoveElement command: element={}, delta={:?}", 
                     element.id(), delta);
                
                // Only create a command if there's actually movement
                if delta.x.abs() < 0.1 && delta.y.abs() < 0.1 {
                    info!("🔄 Ignoring tiny movement (less than 0.1 pixels)");
                    self.state = SelectionState::Idle;
                    self.current_preview = None;
                    return None;
                }
                
                // Create a move command
                let cmd = Command::MoveElement {
                    element_id: element.id(),
                    delta,
                    original_element: Some(element.clone()),
                };
                
                // Reset the state
                self.state = SelectionState::Idle;
                
                // Clear the preview
                self.current_preview = None;
                
                // Return the command to be executed
                Some(cmd)
            }
            SelectionState::Resizing { element, corner, .. } => {
                // Create a resize command
                let cmd = Command::ResizeElement {
                    element_id: element.id(),
                    corner: *corner,
                    new_position: pos,
                    original_element: Some(element.clone()),
                };
                
                // Reset the state
                self.state = SelectionState::Idle;
                self.current_preview = None;
                
                // Return the command to be executed
                Some(cmd)
            }
            _ => {
                info!("No action for pointer up in current state: {:?}", self.state);
                None
            }
        }
    }
    
    pub fn update_preview(&mut self, renderer: &mut Renderer) {
        if let Some(rect) = self.current_preview {
            match &self.state {
                SelectionState::Dragging { element, .. } => {
                    info!("🔄 Setting drag preview: element={}, rect={:?}", element.id(), rect);
                    renderer.set_drag_preview(Some(rect));
                    renderer.set_resize_preview(None);
                },
                SelectionState::Resizing { .. } => {
                    renderer.set_resize_preview(Some(rect));
                    renderer.set_drag_preview(None);
                },
                _ => {
                    renderer.set_resize_preview(None);
                    renderer.set_drag_preview(None);
                }
            }
        } else {
            renderer.set_resize_preview(None);
            renderer.set_drag_preview(None);
        }
    }
    
    pub fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_resize_preview(None);
        renderer.set_drag_preview(None);
        self.current_preview = None;
        debug!("Cleared all previews");
    }
    
    pub fn current_state_name(&self) -> &'static str {
        match self.state {
            SelectionState::Idle => "Idle",
            SelectionState::Selecting { .. } => "Selecting",
            SelectionState::Resizing { .. } => "Resizing",
            SelectionState::Dragging { .. } => "Dragging",
        }
    }
}

impl Tool for UnifiedSelectionTool {
    fn name(&self) -> &'static str {
        "Selection"
    }
    
    fn selection_state(&self) -> Option<&SelectionState> {
        Some(&self.state)
    }
    
    fn activate(&mut self, _doc: &Document) {
        // Reset to idle state when activated
        self.state = SelectionState::Idle;
        self.current_preview = None;
        info!("Selection tool activated");
    }
    
    fn deactivate(&mut self, _doc: &Document) {
        // Reset to idle state when deactivated
        self.state = SelectionState::Idle;
        self.current_preview = None;
        info!("Selection tool deactivated");
    }
    
    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        info!("Selection tool on_pointer_down at position: {:?}", pos);
        
        // First check if we're clicking on a resize handle of a selected element
        if let Some(element) = state.selected_element() {
            let rect = compute_element_rect(&element);
            
            // Check if we're clicking on a resize handle
            let mut is_over_handle = false;
            let mut corner_to_resize = None;
            
            // Check each corner
            let corners = [
                (rect.left_top(), Corner::TopLeft),
                (rect.right_top(), Corner::TopRight),
                (rect.left_bottom(), Corner::BottomLeft),
                (rect.right_bottom(), Corner::BottomRight),
            ];
            
            for (handle_pos, corner) in &corners {
                if is_near_handle_position(pos, *handle_pos, RESIZE_HANDLE_RADIUS) {
                    is_over_handle = true;
                    corner_to_resize = Some(*corner);
                    break;
                }
            }
            
            if is_over_handle && corner_to_resize.is_some() {
                info!("Clicked on resize handle: {:?}", corner_to_resize);
                // Start resizing
                self.start_resizing(
                    element.clone(),
                    corner_to_resize.unwrap(),
                    rect,
                    pos
                );
                return None;
            }
        }
        
        // Check if we're clicking on an element
        let hit_element = doc.element_at_position(pos);
        
        if let Some(element) = hit_element {
            info!("Clicked on element: {:?}", element.id());
            
            // Check if this is already the selected element
            let is_already_selected = if let Some(selected) = state.selected_element() {
                selected.id() == element.id()
            } else {
                false
            };
            
            if is_already_selected {
                // If already selected, start dragging
                info!("Element is already selected, starting drag operation");
                let element_rect = compute_element_rect(&element);
                let offset = pos - element_rect.min;
                self.start_dragging(element.clone(), offset);
                
                // Return None to indicate we're just starting a drag operation
                return None;
            } else {
                // If not already selected, select it
                info!("Selecting new element: {:?}", element.id());
                info!("Element type: {}", if let ElementType::Image(_) = &element { "Image" } else { "Stroke" });
                
                // Log more details about the element
                match &element {
                    ElementType::Image(img) => {
                        info!("Image details: ID={}, size={:?}, pos={:?}", 
                             img.id(), img.size(), img.position());
                    },
                    ElementType::Stroke(stroke) => {
                        info!("Stroke details: ID={}, points={}, thickness={}", 
                             stroke.id(), stroke.points().len(), stroke.thickness());
                    }
                }
                
                self.start_selecting(element.clone(), pos);
                // We don't have a SelectElement command, so we'll return None
                // and let the app handle the selection through the state
                return None;
            }
        } else {
            // Clicked on empty space, clear selection
            info!("Clicked on empty space, clearing selection");
            self.state = SelectionState::Idle;
            // We don't have a ClearSelection command, so we'll return None
            // and let the app handle clearing the selection through the state
            return None;
        }
        
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, _doc: &mut Document, _state: &EditorState, ui: &egui::Ui) -> Option<Command> {
        self.handle_pointer_move(pos, ui)
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        self.handle_pointer_up(pos, doc, state)
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        if let Some(rect) = self.current_preview {
            match &self.state {
                SelectionState::Dragging { element, .. } => {
                    info!("🔄 Setting drag preview: element={}, rect={:?}", element.id(), rect);
                    renderer.set_drag_preview(Some(rect));
                    renderer.set_resize_preview(None);
                },
                SelectionState::Resizing { .. } => {
                    renderer.set_resize_preview(Some(rect));
                    renderer.set_drag_preview(None);
                },
                _ => {
                    renderer.set_resize_preview(None);
                    renderer.set_drag_preview(None);
                }
            }
        } else {
            renderer.set_resize_preview(None);
            renderer.set_drag_preview(None);
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_resize_preview(None);
        renderer.set_drag_preview(None);
        self.current_preview = None;
        debug!("Cleared all previews");
    }
    
    fn ui(&mut self, ui: &mut Ui, doc: &Document) -> Option<Command> {
        ui.label("Selection Tool");
        
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            ui.label("Handle size:");
            if ui.add(egui::Slider::new(&mut self.handle_size, 4.0..=16.0)).changed() {
                debug!("Handle size changed to: {}", self.handle_size);
            }
        });
        
        // Add a debug section
        ui.separator();
        ui.label("Debug Tools");
        
        // Add a debug button to test image resizing directly
        if ui.button("🔧 DEBUG: Force Resize First Image").clicked() {
            // Find the first image in the document
            if !doc.images().is_empty() {
                let image = &doc.images()[0];
                // We found an image, try to resize it manually
                info!("🛠️ DEBUG: Found image {}, original size: {:?}, pos: {:?}", 
                     image.id(), image.size(), image.position());
                
                // Create an explicit resize command with a very different size
                let new_rect = egui::Rect::from_min_size(
                    image.position() + egui::vec2(50.0, 50.0), // Move it 50px
                    image.size() * 0.5 // Make it half the original size
                );
                
                info!("🛠️ DEBUG: New rect for image: {:?}", new_rect);
                
                // Use a direct image replacement approach instead of a resize command
                // This seems to be more reliable
                let new_image = crate::image::Image::new_ref_with_id(
                    image.id(),
                    image.data().to_vec(),  // Preserve original image data
                    new_rect.size(),        // New size
                    new_rect.min           // New position
                );
                
                // Create a custom command that will just do the replacement
                info!("🛠️ DEBUG: Creating debug resize command");
                
                // Just use AddImage command directly
                return Some(Command::AddImage(new_image));
            } else {
                info!("🛠️ DEBUG: No images found in document");
            }
        }
        
        // Add a button to test direct image replacement
        if ui.button("🔨 DEBUG: Direct Image Replacement").clicked() {
            if !doc.images().is_empty() {
                let image = &doc.images()[0];
                info!("🔨 DEBUG: Found image to replace, ID: {}", image.id());
                
                // Create a new image with different dimensions
                let new_size = image.size() * 0.7; // 70% of original size
                let new_pos = image.position() + egui::vec2(30.0, 30.0);
                
                info!("🔨 DEBUG: Creating new image: size={:?}, pos={:?}", new_size, new_pos);
                
                return Some(Command::AddImage(
                    crate::image::Image::new_ref(
                        image.data().to_vec(),
                        new_size,
                        new_pos
                    )
                ));
            }
        }
        
        None
    }
    
    fn get_config(&self) -> Box<dyn ToolConfig> {
        Box::new(SelectionToolConfig {
            handle_size: self.handle_size,
        })
    }
    
    fn apply_config(&mut self, config: &dyn ToolConfig) {
        if let Some(selection_config) = config.as_any().downcast_ref::<SelectionToolConfig>() {
            self.handle_size = selection_config.handle_size;
            debug!("Applied selection tool config with handle_size: {}", self.handle_size);
        }
    }
}

/// Create a new selection tool
pub fn new_selection_tool() -> UnifiedSelectionTool {
    UnifiedSelectionTool::new()
}

/// Helper function to check if a point is near a handle position
fn is_near_handle_position(pos: Pos2, handle_pos: Pos2, radius: f32) -> bool {
    pos.distance(handle_pos) <= radius
}