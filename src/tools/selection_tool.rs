use egui::{Pos2, Ui, Rect};
use crate::command::Command;
use crate::document::Document;
use crate::tools::{Tool, ToolConfig};
use crate::renderer::Renderer;
use crate::state::ElementType;
use crate::geometry::hit_testing::{compute_element_rect, RESIZE_HANDLE_RADIUS};
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
        info!("Starting drag with offset: {:?}", offset);
        self.state = SelectionState::Dragging {
            element,
            offset
        };
    }
    
    // The old less verbose version will be removed to avoid duplicate definition
    
    pub fn handle_pointer_move(&mut self, pos: Pos2, _doc: &mut Document, _state: &EditorState) -> Option<Command> {
        info!("Selection tool handling pointer move at position: {:?}", pos);
        
        match &self.state {
            SelectionState::Dragging { element, offset } => {
                let new_pos = pos - *offset;
                let element_rect = element.rect();
                let delta = new_pos - element_rect.min;
                
                // Calculate new rectangle for the dragged element
                let new_rect = element_rect.translate(delta);
                
                // ONLY update preview visualization, do NOT modify the document
                self.current_preview = Some(new_rect);
                info!("Dragging preview updated: from {:?} to {:?}", element_rect, new_rect);
                
                // No longer directly updating the document element!
                // This is now handled by the command when pointer_up occurs
            }
            SelectionState::Resizing { corner, original_rect, start_pos, .. } => {
                // Calculate new rect based on resize operation and constrain to minimum size
                let delta = pos - *start_pos;
                let min_size = 10.0; // Minimum 10x10 size to prevent tiny/inverted elements
                
                // Create a new rectangle based on the corner being dragged
                let mut new_rect = match corner {
                    Corner::TopLeft => Rect::from_min_max(
                        original_rect.min + delta,
                        original_rect.max,
                    ),
                    Corner::TopRight => Rect::from_min_max(
                        Pos2::new(original_rect.min.x, original_rect.min.y + delta.y),
                        Pos2::new(original_rect.max.x + delta.x, original_rect.max.y),
                    ),
                    Corner::BottomLeft => Rect::from_min_max(
                        Pos2::new(original_rect.min.x + delta.x, original_rect.min.y),
                        Pos2::new(original_rect.max.x, original_rect.max.y + delta.y),
                    ),
                    Corner::BottomRight => Rect::from_min_max(
                        original_rect.min,
                        original_rect.max + delta,
                    ),
                };
                
                // Ensure the rect has positive width and height and meets minimum size
                if new_rect.width() < min_size {
                    // Adjust based on which side is being manipulated
                    match corner {
                        Corner::TopLeft | Corner::BottomLeft => {
                            new_rect.min.x = new_rect.max.x - min_size;
                        },
                        Corner::TopRight | Corner::BottomRight => {
                            new_rect.max.x = new_rect.min.x + min_size;
                        },
                    }
                }
                
                if new_rect.height() < min_size {
                    // Adjust based on which side is being manipulated
                    match corner {
                        Corner::TopLeft | Corner::TopRight => {
                            new_rect.min.y = new_rect.max.y - min_size;
                        },
                        Corner::BottomLeft | Corner::BottomRight => {
                            new_rect.max.y = new_rect.min.y + min_size;
                        },
                    }
                }
                
                // ONLY update preview visualization, do NOT modify the document
                self.current_preview = Some(new_rect);
                info!("Resizing preview updated: from {:?} to {:?}", original_rect, new_rect);
                
                // No longer directly updating the document element!
                // This is now handled by the command when pointer_up occurs
            }
            _ => {
                info!("No action for pointer move in current state: {:?}", self.state);
            }
        }
        
        None
    }
    
    // Enhanced function to cancel any ongoing interaction and clean all state
    pub fn cancel_interaction(&mut self) {
        info!("Force canceling all interaction state");
        self.state = SelectionState::Idle;
        self.current_preview = None;
    }
    
    pub fn handle_pointer_up(&mut self, pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        info!("Selection tool handling pointer up at position: {:?}", pos);
        
        // Store the command result 
        let command = match &self.state {
            SelectionState::Dragging { element, .. } => {
                if let Some(preview) = self.current_preview {
                    let delta = preview.min - element.rect().min;
                    info!("Creating MoveElement command with delta: {:?}", delta);
                    
                    // Determine if this is a stroke or an image and get its index
                    let (is_stroke, element_index) = match element {
                        ElementType::Stroke(_stroke) => {
                            // Find the index of the stroke in the document
                            let mut index = 0;
                            for (i, stroke) in _doc.strokes().iter().enumerate() {
                                let stroke_element = ElementType::Stroke(stroke.clone());
                                if stroke_element.get_stable_id() == element.get_stable_id() {
                                    index = i;
                                    break;
                                }
                            }
                            (true, index)
                        },
                        ElementType::Image(_image) => {
                            // Find the index of the image in the document
                            let mut index = 0;
                            for (i, image) in _doc.images().iter().enumerate() {
                                if image.id() == element.get_id() {
                                    index = i;
                                    break;
                                }
                            }
                            (false, index)
                        }
                    };
                    
                    Some(Command::MoveElement {
                        element_id: element.get_id(),
                        delta,
                        element_index,
                        is_stroke,
                        original_element: Some(element.clone()),
                    })
                } else {
                    info!("No preview available for dragging, no command created");
                    None
                }
            },
            SelectionState::Resizing { element, corner, .. } => {
                if let Some(preview) = self.current_preview {
                    let new_position = match corner {
                        Corner::TopLeft => preview.min,
                        Corner::TopRight => preview.right_top(),
                        Corner::BottomLeft => preview.left_bottom(),
                        Corner::BottomRight => preview.max,
                    };
                    info!("Creating ResizeElement command with new position: {:?}", new_position);
                    Some(Command::ResizeElement {
                        element_id: element.get_id(),
                        corner: *corner,
                        new_position,
                        original_element: Some(element.clone()),
                    })
                } else {
                    info!("No preview available for resizing, no command created");
                    None
                }
            },
            _ => {
                info!("No action for pointer up in current state: {:?}", self.state);
                None
            }
        };
        
        // ENSURE we reset state BEFORE returning
        // This is critical - even if command generation fails,
        // we must reset all interaction state
        info!("Canceling selection tool interaction");
        self.cancel_interaction();
        
        // Clear preview state explicitly
        self.current_preview = None;
        
        command
    }
    
    pub fn update_preview(&mut self, renderer: &mut Renderer) {
        if let Some(rect) = self.current_preview {
            match &self.state {
                SelectionState::Dragging { .. } => {
                    renderer.set_drag_preview(Some(rect));
                    renderer.set_resize_preview(None);
                    debug!("Updated drag preview: {:?}", rect);
                },
                SelectionState::Resizing { .. } => {
                    renderer.set_resize_preview(Some(rect));
                    renderer.set_drag_preview(None);
                    debug!("Updated resize preview: {:?}", rect);
                },
                _ => {
                    renderer.set_resize_preview(None);
                    renderer.set_drag_preview(None);
                    debug!("Cleared previews in state: {:?}", self.state);
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
        
        // Check if we're clicking on a resize handle
        let mut is_over_handle = false;
        let mut corner_to_resize = None;
        
        if let Some(element) = state.selected_element() {
            // Use compute_element_rect to get the correct bounding box with padding
            let rect = compute_element_rect(&element);
            
            // Check each corner
            for corner in &[
                Corner::TopLeft,
                Corner::TopRight,
                Corner::BottomLeft,
                Corner::BottomRight,
            ] {
                let handle_pos = match corner {
                    Corner::TopLeft => rect.min,
                    Corner::TopRight => Pos2::new(rect.max.x, rect.min.y),
                    Corner::BottomLeft => Pos2::new(rect.min.x, rect.max.y),
                    Corner::BottomRight => rect.max,
                };
                
                if is_near_handle_position(pos, handle_pos, RESIZE_HANDLE_RADIUS) {
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
            info!("Clicked on element: {:?}", element.get_id());
            
            // Check if this element is already selected
            let is_already_selected = if let Some(selected) = state.selected_element() {
                match (selected, &element) {
                    (ElementType::Image(sel_img), ElementType::Image(hit_img)) => sel_img.id() == hit_img.id(),
                    (ElementType::Stroke(sel_stroke), ElementType::Stroke(hit_stroke)) => {
                        // Use stable IDs for comparison
                        let sel_element = ElementType::Stroke(sel_stroke.clone());
                        let hit_element = ElementType::Stroke(hit_stroke.clone());
                        sel_element.get_stable_id() == hit_element.get_stable_id()
                    },
                    _ => false,
                }
            } else {
                false
            };
            
            if is_already_selected {
                // If already selected, start dragging
                let element_rect = element.rect();
                let offset = pos - element_rect.min;
                self.start_dragging(element.clone(), offset);
            } else {
                // If not already selected, just select it
                // We'll return None since we're not generating a command
                // The app will handle the selection in the next frame
                info!("Selecting new element: {:?}", element.get_id());
                
                // We need to return to idle state to avoid any ongoing interactions
                self.state = SelectionState::Idle;
                
                // Return None - the app will handle the selection through the state
                return None;
            }
        } else if let Some(element) = state.selected_element() {
            info!("No element hit, but have selected element");
            // Start a selection rectangle
            self.start_selecting(element.clone(), pos);
        } else {
            info!("No element hit or selected");
        }
        
        None
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, doc: &mut Document, state: &EditorState) -> Option<Command> {
        self.handle_pointer_move(pos, doc, state)
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        self.handle_pointer_up(pos, doc, state)
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        if let Some(rect) = self.current_preview {
            match &self.state {
                SelectionState::Dragging { .. } => {
                    renderer.set_drag_preview(Some(rect));
                    renderer.set_resize_preview(None);
                    debug!("Updated drag preview: {:?}", rect);
                },
                SelectionState::Resizing { .. } => {
                    renderer.set_resize_preview(Some(rect));
                    renderer.set_drag_preview(None);
                    debug!("Updated resize preview: {:?}", rect);
                },
                _ => {
                    renderer.set_resize_preview(None);
                    renderer.set_drag_preview(None);
                    debug!("Cleared previews in state: {:?}", self.state);
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
    
    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool");
        
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            ui.label("Handle size:");
            if ui.add(egui::Slider::new(&mut self.handle_size, 4.0..=16.0)).changed() {
                debug!("Handle size changed to: {}", self.handle_size);
            }
        });
        
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

impl ElementType {
    fn get_id(&self) -> usize {
        match self {
            ElementType::Stroke(_stroke_ref) => {
                // Use the stable ID approach instead of just the pointer address
                let element = self.clone();
                element.get_stable_id()
            },
            ElementType::Image(image_ref) => image_ref.id(),
        }
    }
    
    // Changed from private to public method
    pub fn rect(&self) -> Rect {
        // Use compute_element_rect for both stroke and image elements
        compute_element_rect(self)
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

/// Helper function to check if a point is over a resize handle
fn is_over_resize_handle(pos: Pos2, _doc: &Document, state: &crate::state::EditorState) -> bool {
    if let Some(element) = state.selected_element() {
        let rect = element.rect();
        
        // Check each corner
        for corner in &[
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomLeft,
            Corner::BottomRight,
        ] {
            let handle_pos = match corner {
                Corner::TopLeft => rect.min,
                Corner::TopRight => Pos2::new(rect.max.x, rect.min.y),
                Corner::BottomLeft => Pos2::new(rect.min.x, rect.max.y),
                Corner::BottomRight => rect.max,
            };
            
            if is_near_handle_position(pos, handle_pos, RESIZE_HANDLE_RADIUS) {
                return true;
            }
        }
    }
    
    false
}