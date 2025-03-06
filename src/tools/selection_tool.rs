use egui::{Pos2, Ui, Rect};
use crate::command::Command;
use crate::document::Document;
use crate::tools::{Tool, ToolConfig};
use crate::renderer::Renderer;
use crate::state::ElementType;
use crate::geometry::hit_testing::{compute_element_rect, is_point_near_handle, RESIZE_HANDLE_RADIUS};
use crate::state::EditorState;
use crate::widgets::Corner;
use std::any::Any;
use log::{debug, info};

// Config for SelectionTool
#[derive(Clone, Debug)]
pub struct SelectionToolConfig {
    // Add configurable properties
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
            handle_size: 8.0,
            current_preview: None
        }
    }
    
    pub fn start_selecting(&mut self, element: ElementType, pos: Pos2) {
        self.state = SelectionState::Selecting {
            element,
            start_pos: pos
        };
    }
    
    pub fn start_resizing(&mut self, element: ElementType, corner: Corner, original_rect: Rect, pos: Pos2) {
        self.state = SelectionState::Resizing {
            element,
            corner,
            original_rect,
            start_pos: pos,
            handle_size: self.handle_size
        };
    }
    
    pub fn start_dragging(&mut self, element: ElementType, offset: egui::Vec2) {
        self.state = SelectionState::Dragging {
            element,
            offset
        };
    }
    
    pub fn cancel_interaction(&mut self) {
        self.state = SelectionState::Idle;
        self.current_preview = None;
    }
    
    pub fn on_pointer_down(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        // Check if we're clicking on a resize handle
        let mut is_over_handle = false;
        if let Some(element) = state.selected_element() {
            is_over_handle = is_point_near_handle(pos, element);
        }
        
        if is_over_handle {
            // Handle resize logic
            return None;
        }
        
        // Check if we're clicking on an element
        let hit_element = doc.element_at_position(pos);
        
        if let Some(element) = hit_element {
            // Start dragging the element
            let element_rect = element.rect();
            let offset = pos - element_rect.min;
            self.start_dragging(element.clone(), offset);
        } else {
            // Start a selection rectangle
            if let Some(element) = state.selected_element() {
                self.start_selecting(element.clone(), pos);
            }
        }
        
        None
    }
    
    pub fn handle_pointer_move(&mut self, pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        match &self.state {
            SelectionState::Dragging { element, offset } => {
                let new_pos = pos - *offset;
                let element_rect = element.rect();
                let delta = new_pos - element_rect.min;
                
                // Update preview
                self.current_preview = Some(element_rect.translate(delta));
            }
            SelectionState::Resizing { element, corner, original_rect, start_pos, .. } => {
                // Calculate new rect based on resize operation
                let delta = pos - *start_pos;
                let new_rect = match corner {
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
                
                // Update preview
                self.current_preview = Some(new_rect);
            }
            _ => {}
        }
        
        None
    }
    
    pub fn handle_pointer_up(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        let command = match &self.state {
            SelectionState::Dragging { element, .. } => {
                if let Some(preview) = self.current_preview {
                    let delta = preview.min - element.rect().min;
                    // TODO: Implement MoveElement command
                    None
                } else {
                    None
                }
            }
            SelectionState::Resizing { element, corner, .. } => {
                if let Some(preview) = self.current_preview {
                    Some(Command::ResizeElement {
                        element_id: element.get_id(),
                        corner: *corner,
                        new_position: match corner {
                            Corner::TopLeft => preview.min,
                            Corner::TopRight => preview.right_top(),
                            Corner::BottomLeft => preview.left_bottom(),
                            Corner::BottomRight => preview.max,
                        },
                    })
                } else {
                    None
                }
            }
            _ => None,
        };
        
        // Reset state
        self.cancel_interaction();
        
        command
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        if let Some(rect) = self.current_preview {
            renderer.set_resize_preview(Some(rect));
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_resize_preview(None);
    }
    
    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool");
        
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            ui.label("Handle size:");
            if ui.add(egui::Slider::new(&mut self.handle_size, 4.0..=16.0)).changed() {
                // Handle size changed
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
        log::info!("Selection tool activated");
    }
    
    fn deactivate(&mut self, _doc: &Document) {
        // Reset to idle state when deactivated
        self.state = SelectionState::Idle;
        self.current_preview = None;
        log::info!("Selection tool deactivated");
    }
    
    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        // Check if we're clicking on a resize handle
        let mut is_over_handle = false;
        if let Some(element) = state.selected_element() {
            is_over_handle = is_point_near_handle(pos, element);
        }
        
        if is_over_handle {
            // Handle resize logic
            return None;
        }
        
        // Check if we're clicking on an element
        let hit_element = doc.element_at_position(pos);
        
        if let Some(element) = hit_element {
            // Start dragging the element
            let element_rect = element.rect();
            let offset = pos - element_rect.min;
            self.start_dragging(element.clone(), offset);
        } else {
            // Start a selection rectangle
            if let Some(element) = state.selected_element() {
                self.start_selecting(element.clone(), pos);
            }
        }
        
        None
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        self.handle_pointer_move(pos, doc, state)
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        self.handle_pointer_up(pos, doc, state)
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        if let Some(rect) = self.current_preview {
            renderer.set_resize_preview(Some(rect));
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_resize_preview(None);
    }
    
    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool");
        
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            ui.label("Handle size:");
            if ui.add(egui::Slider::new(&mut self.handle_size, 4.0..=16.0)).changed() {
                // Handle size changed
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
            ElementType::Stroke(stroke_ref) => {
                // Use the pointer address as a unique ID
                std::sync::Arc::as_ptr(stroke_ref) as usize
            },
            ElementType::Image(image_ref) => image_ref.id(),
        }
    }
    
    fn rect(&self) -> Rect {
        match self {
            ElementType::Stroke(stroke_ref) => compute_element_rect(self),
            ElementType::Image(image_ref) => compute_element_rect(self),
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

/// Helper function to check if a point is over a resize handle
fn is_over_resize_handle(pos: Pos2, doc: &Document, state: &crate::state::EditorState) -> bool {
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