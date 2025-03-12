use egui::{Pos2, Ui, Rect};
use crate::command::Command;
use crate::tools::{Tool, ToolConfig};
use crate::renderer::Renderer;
use crate::element::ElementType;
use crate::element::Element;
use crate::element::{compute_element_rect, RESIZE_HANDLE_RADIUS};
use crate::state::EditorModel;
use crate::widgets::resize_handle::Corner;
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
            Self::Selecting { element, start_pos } => f.debug_struct("Selecting")
                .field("element_id", &element.id())
                .field("start_pos", start_pos)
                .finish(),
            Self::Resizing { element, corner, original_rect, start_pos, handle_size } => f.debug_struct("Resizing")
                .field("element_id", &element.id())
                .field("corner", corner)
                .field("original_rect", original_rect)
                .field("start_pos", start_pos)
                .field("handle_size", handle_size)
                .finish(),
            Self::Dragging { element, offset } => f.debug_struct("Dragging")
                .field("element_id", &element.id())
                .field("offset", offset)
                .finish(),
        }
    }
}

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
        info!("start_selecting called with element ID: {} at position: {:?}", element.id(), pos);
        
        self.state = SelectionState::Selecting {
            element,
            start_pos: pos
        };
    }
    
    pub fn start_resizing(&mut self, element: ElementType, corner: Corner, original_rect: Rect, pos: Pos2) {
        info!("start_resizing called with element ID: {}, corner: {:?}, at position: {:?}", 
             element.id(), corner, pos);
        
        self.state = SelectionState::Resizing {
            element,
            corner,
            original_rect,
            start_pos: pos,
            handle_size: self.handle_size
        };
    }
    
    pub fn start_dragging(&mut self, element: ElementType, offset: egui::Vec2) {
        info!("start_dragging called with element ID: {}, offset: {:?}", element.id(), offset);
        
        self.state = SelectionState::Dragging {
            element,
            offset
        };
    }
    
    pub fn handle_pointer_move(&mut self, pos: Pos2, ui: &egui::Ui) -> Option<Command> {
        match &self.state {
            SelectionState::Idle => None,
            SelectionState::Selecting { .. } => {
                // Just update the preview, no command yet
                None
            },
            SelectionState::Resizing { element, corner, original_rect, start_pos, .. } => {
                // Calculate the new position for the resize handle
                let new_position = pos;
                
                // Create a resize command
                let command = Command::ResizeElement {
                    element_id: element.id(),
                    corner: *corner,
                    new_position,
                    original_element: Some(element.clone()),
                };
                
                // Update the preview
                self.current_preview = Some(*original_rect);
                
                // Return the command
                Some(command)
            },
            SelectionState::Dragging { element, offset } => {
                // Calculate the new position
                let element_rect = compute_element_rect(element);
                let new_pos = pos - *offset;
                let delta = new_pos - element_rect.min;
                
                // Create a move command
                let command = Command::MoveElement {
                    element_id: element.id(),
                    delta,
                    original_element: Some(element.clone()),
                };
                
                // Update the preview
                self.current_preview = Some(element_rect);
                
                // Return the command
                Some(command)
            }
        }
    }
    
    pub fn cancel_interaction(&mut self) {
        info!("cancel_interaction called");
        self.state = SelectionState::Idle;
        self.current_preview = None;
    }
    
    pub fn handle_pointer_up(&mut self, pos: Pos2, _editor_model: &EditorModel) -> Option<Command> {
        info!("handle_pointer_up called at position: {:?}", pos);
        
        let result = match &self.state {
            SelectionState::Idle => None,
            SelectionState::Selecting { element, .. } => {
                // When selecting, we don't generate a command, we just update the state
                // The app will handle the selection through the state
                info!("Finalizing selection of element ID: {}", element.id());
                None
            },
            SelectionState::Resizing { element, corner, original_rect, .. } => {
                // Create a resize command
                info!("Finalizing resize of element ID: {}", element.id());
                let command = Command::ResizeElement {
                    element_id: element.id(),
                    corner: *corner,
                    new_position: pos,
                    original_element: Some(element.clone()),
                };
                Some(command)
            },
            SelectionState::Dragging { element, offset } => {
                // Create a move command
                info!("Finalizing move of element ID: {}", element.id());
                let element_rect = compute_element_rect(element);
                let new_pos = pos - *offset;
                let delta = new_pos - element_rect.min;
                
                let command = Command::MoveElement {
                    element_id: element.id(),
                    delta,
                    original_element: Some(element.clone()),
                };
                Some(command)
            }
        };
        
        // Reset the state
        self.state = SelectionState::Idle;
        self.current_preview = None;
        
        result
    }
    
    pub fn update_preview(&mut self, renderer: &mut Renderer) {
        match &self.state {
            SelectionState::Idle => {
                // No preview in Idle state
                renderer.set_resize_preview(None);
                renderer.set_drag_preview(None);
            },
            SelectionState::Selecting { element, .. } => {
                // Show a preview rect around the element
                let rect = compute_element_rect(element);
                renderer.set_resize_preview(Some(rect));
                self.current_preview = Some(rect);
            },
            SelectionState::Resizing { .. } | SelectionState::Dragging { .. } => {
                // Preview is handled by the handle_pointer_move method
                if let Some(rect) = self.current_preview {
                    match &self.state {
                        SelectionState::Resizing { .. } => {
                            renderer.set_resize_preview(Some(rect));
                            renderer.set_drag_preview(None);
                        },
                        SelectionState::Dragging { .. } => {
                            renderer.set_drag_preview(Some(rect));
                            renderer.set_resize_preview(None);
                        },
                        _ => {} // Already handled above
                    }
                }
            }
        }
    }
    
    pub fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_resize_preview(None);
        renderer.set_drag_preview(None);
        self.current_preview = None;
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
    
    fn activate(&mut self, _editor_model: &EditorModel) {
        // Reset to idle state when activated
        self.state = SelectionState::Idle;
        self.current_preview = None;
        info!("Selection tool activated");
    }
    
    fn deactivate(&mut self, _editor_model: &EditorModel) {
        // Reset to idle state when deactivated
        self.state = SelectionState::Idle;
        self.current_preview = None;
        info!("Selection tool deactivated");
    }
    
    fn on_pointer_down(&mut self, pos: Pos2, editor_model: &EditorModel) -> Option<Command> {
        info!("Selection tool on_pointer_down at position: {:?}", pos);
        
        // First check if we're clicking on a resize handle of a selected element
        if let Some(element) = editor_model.selected_element() {
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
        let hit_element = editor_model.element_at_position(pos);
        
        if let Some(element) = hit_element {
            info!("Clicked on element: {:?}", element.id());
            
            // Check if this is already the selected element
            let is_already_selected = if let Some(selected) = editor_model.selected_element() {
                selected.get_stable_id() == element.get_stable_id()
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
    
    fn on_pointer_move(&mut self, pos: Pos2, _editor_model: &mut EditorModel, ui: &egui::Ui) -> Option<Command> {
        self.handle_pointer_move(pos, ui)
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, editor_model: &EditorModel) -> Option<Command> {
        self.handle_pointer_up(pos, editor_model)
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        match &self.state {
            SelectionState::Idle => {
                // No preview in Idle state
                renderer.set_resize_preview(None);
                renderer.set_drag_preview(None);
            },
            SelectionState::Selecting { element, .. } => {
                // Show a preview rect around the element
                let rect = compute_element_rect(element);
                renderer.set_resize_preview(Some(rect));
                self.current_preview = Some(rect);
            },
            SelectionState::Resizing { .. } | SelectionState::Dragging { .. } => {
                // Preview is handled by the handle_pointer_move method
                if let Some(rect) = self.current_preview {
                    match &self.state {
                        SelectionState::Resizing { .. } => {
                            renderer.set_resize_preview(Some(rect));
                            renderer.set_drag_preview(None);
                        },
                        SelectionState::Dragging { .. } => {
                            renderer.set_drag_preview(Some(rect));
                            renderer.set_resize_preview(None);
                        },
                        _ => {} // Already handled above
                    }
                }
            }
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_resize_preview(None);
        renderer.set_drag_preview(None);
        self.current_preview = None;
    }
    
    fn ui(&mut self, ui: &mut Ui, editor_model: &EditorModel) -> Option<Command> {
        ui.label("Selection Tool");
        
        // Show information about the current selection
        if let Some(element) = editor_model.selected_element() {
            ui.label("Selected Element:");
            
            match &element {
                ElementType::Image(img) => {
                    ui.label(format!("Type: Image"));
                    ui.label(format!("ID: {}", img.id()));
                    ui.label(format!("Size: {}x{}", img.size().x, img.size().y));
                    ui.label(format!("Position: {:.1},{:.1}", img.position().x, img.position().y));
                },
                ElementType::Stroke(stroke) => {
                    ui.label(format!("Type: Stroke"));
                    ui.label(format!("ID: {}", stroke.id()));
                    ui.label(format!("Points: {}", stroke.points().len()));
                    ui.label(format!("Color: {:?}", stroke.color()));
                    ui.label(format!("Thickness: {:.1}", stroke.thickness()));
                }
            }
            
            ui.separator();
            ui.label("Actions:");
            ui.label("• Drag to move");
            ui.label("• Drag corners to resize");
            ui.label("• Click empty space to deselect");
        } else {
            ui.label("No element selected");
            ui.label("Click on an element to select it");
        }
        
        // Show current tool state
        ui.separator();
        ui.label(format!("Tool State: {}", self.current_state_name()));
        
        None  // No immediate command from UI
    }
    
    fn get_config(&self) -> Box<dyn ToolConfig> {
        Box::new(SelectionToolConfig {
            handle_size: self.handle_size,
        })
    }
    
    fn apply_config(&mut self, config: &dyn ToolConfig) {
        if let Some(config) = config.as_any().downcast_ref::<SelectionToolConfig>() {
            self.handle_size = config.handle_size;
        }
    }
}

pub fn new_selection_tool() -> UnifiedSelectionTool {
    UnifiedSelectionTool::new()
}

fn is_near_handle_position(pos: Pos2, handle_pos: Pos2, radius: f32) -> bool {
    let distance = (pos - handle_pos).length();
    distance <= radius
}