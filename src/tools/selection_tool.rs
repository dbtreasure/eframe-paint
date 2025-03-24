use crate::command::Command;
use crate::element::Element;
use crate::element::ElementType;
use crate::element::{RESIZE_HANDLE_RADIUS, compute_element_rect};
use crate::renderer::Renderer;
use crate::state::EditorModel;
use crate::tools::{Tool, ToolConfig};
use crate::widgets::Corner;
use egui::{Pos2, Ui};
use log::info;
use std::any::Any;

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
        start_pos: egui::Pos2,
        current_pos: egui::Pos2,
        adding_to_selection: bool, // Tracks if Shift is held
    },
    Dragging {
        start_pos: egui::Pos2,
        current_pos: egui::Pos2,
        initial_element_positions: std::collections::HashMap<usize, egui::Pos2>,
        original_rect: egui::Rect,  // Store the exact original rect
        grid_snap_enabled: bool, // Tracks if Ctrl is held
    },
    Resizing {
        element_id: usize,
        corner: Corner,
        start_pos: egui::Pos2,
        current_pos: egui::Pos2,
        original_rect: egui::Rect,
        preserve_aspect_ratio: bool, // Tracks if Shift is held
    },
}

// Manual Debug implementation for SelectionState
impl std::fmt::Debug for SelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Selecting { start_pos, current_pos, adding_to_selection } => f
                .debug_struct("Selecting")
                .field("start_pos", start_pos)
                .field("current_pos", current_pos)
                .field("adding_to_selection", adding_to_selection)
                .finish(),
            Self::Resizing {
                element_id,
                corner,
                original_rect,
                start_pos,
                current_pos,
                preserve_aspect_ratio,
            } => f
                .debug_struct("Resizing")
                .field("element_id", element_id)
                .field("corner", corner)
                .field("original_rect", original_rect)
                .field("start_pos", start_pos)
                .field("current_pos", current_pos)
                .field("preserve_aspect_ratio", preserve_aspect_ratio)
                .finish(),
            Self::Dragging { 
                start_pos, 
                current_pos, 
                initial_element_positions, 
                original_rect,
                grid_snap_enabled 
            } => f
                .debug_struct("Dragging")
                .field("start_pos", start_pos)
                .field("current_pos", current_pos)
                .field("element_count", &initial_element_positions.len())
                .field("original_rect", original_rect)
                .field("grid_snap_enabled", grid_snap_enabled)
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnifiedSelectionTool {
    pub state: SelectionState,
    pub handle_size: f32,
}

impl UnifiedSelectionTool {
    pub fn new() -> Self {
        Self {
            state: SelectionState::Idle,
            handle_size: DEFAULT_HANDLE_SIZE,
        }
    }

    // Helper to reset state to idle
    pub fn reset_interaction_state(&mut self) {
        self.state = SelectionState::Idle;
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
        info!("SelectionTool activated");
    }

    fn deactivate(&mut self, _editor_model: &EditorModel) {
        info!("SelectionTool deactivated");
        self.reset_interaction_state();
    }

    fn requires_selection(&self) -> bool {
        false // Selection tool works without a selection
    }

    fn on_pointer_down(
        &mut self, 
        pos: Pos2,
        button: egui::PointerButton,
        modifiers: &egui::Modifiers,
        editor_model: &EditorModel,
        renderer: &mut Renderer,
    ) -> Option<Command> {
        // Only respond to primary button
        if button != egui::PointerButton::Primary {
            return None;
        }
        
        // First, check if we're clicking on a resize handle of a selected element
        for &element_id in editor_model.selected_ids() {
            if let Some(element) = editor_model.find_element_by_id(element_id) {
                let rect = compute_element_rect(element);
                
                // Check all corners for potential resize handles
                let handle_radius = RESIZE_HANDLE_RADIUS;
                let corners = [
                    (rect.left_top(), Corner::TopLeft),
                    (rect.right_top(), Corner::TopRight),
                    (rect.left_bottom(), Corner::BottomLeft),
                    (rect.right_bottom(), Corner::BottomRight),
                ];
                
                for (corner_pos, corner) in corners {
                    if is_near_handle_position(pos, corner_pos, handle_radius) {
                        // Start resizing this element from this corner
                        renderer.set_resize_preview(Some(rect));
                        
                        self.state = SelectionState::Resizing {
                            element_id,
                            corner,
                            start_pos: pos,
                            current_pos: pos,
                            original_rect: rect,
                            preserve_aspect_ratio: modifiers.shift,
                        };
                        return None;
                    }
                }
            }
        }
        
        // Check if we're clicking on any element
        if let Some(element) = editor_model.element_at_position(pos) {
            let element_id = element.id();
            
            if modifiers.shift {
                // Shift+click: Toggle selection
                return Some(Command::ToggleSelection(element_id));
            }
            
            let is_already_selected = editor_model.is_element_selected(element_id);
            
            // Not already selected, and not using shift? Select it first
            if !is_already_selected {
                // We'll capture this element and deselect others
                // But we don't want to use the command system for this when dragging
                // We'll embed this in the MoveElement command later
                let selection_command = Command::SelectElement(element_id);
                
                // If we're just clicking, return the selection command
                // But initialize the drag state first
                let rect = compute_element_rect(element);
                let mut initial_positions = std::collections::HashMap::new();
                initial_positions.insert(element_id, rect.min);
                
                self.state = SelectionState::Dragging {
                    start_pos: pos,
                    current_pos: pos,
                    initial_element_positions: initial_positions,
                    original_rect: rect,
                    grid_snap_enabled: modifiers.ctrl,
                };
                
                // Set up the preview
                renderer.set_drag_preview(Some(rect));
                
                // Return the selection command
                return Some(selection_command);
            } else {
                // Already selected - start dragging all selected elements
                let mut initial_positions = std::collections::HashMap::new();
                let mut original_rect = None;
                
                // Include all selected elements in the drag operation
                for &id in editor_model.selected_ids() {
                    if let Some(el) = editor_model.find_element_by_id(id) {
                        let rect = compute_element_rect(el);
                        initial_positions.insert(id, rect.min);
                        
                        // Use the clicked element's rect as our reference
                        if id == element_id {
                            original_rect = Some(rect);
                            // Set the initial drag preview to be exactly the same as the selection box
                            renderer.set_drag_preview(Some(rect));
                        }
                    }
                }
                
                // If we somehow don't have the clicked element's rect, use it as the preview
                if original_rect.is_none() {
                    let rect = compute_element_rect(element);
                    original_rect = Some(rect);
                    renderer.set_drag_preview(Some(rect));
                    initial_positions.insert(element_id, rect.min);
                }
                
                // Set up dragging state
                self.state = SelectionState::Dragging {
                    start_pos: pos,
                    current_pos: pos,
                    initial_element_positions: initial_positions,
                    original_rect: original_rect.unwrap(),
                    grid_snap_enabled: modifiers.ctrl,
                };
            }
        } else {
            // Clicked in empty space
            if !modifiers.shift && !editor_model.selected_ids().is_empty() {
                // Clear selection when clicking in empty space (if not using shift)
                return Some(Command::ClearSelection {
                    previous_selection: editor_model.selected_ids().clone(),
                });
            }
            
            // Start selection rectangle
            self.state = SelectionState::Selecting {
                start_pos: pos,
                current_pos: pos,
                adding_to_selection: modifiers.shift,
            };
        }
        
        None
    }

    fn on_pointer_move(
        &mut self, 
        pos: Pos2,
        held_buttons: &[egui::PointerButton],
        modifiers: &egui::Modifiers,
        editor_model: &mut EditorModel,
        _ui: &egui::Ui,
        renderer: &mut Renderer
    ) -> Option<Command> {
        // Check if primary button is held for drag operations
        let primary_held = held_buttons.contains(&egui::PointerButton::Primary);
        
        // Update current position in state based on the interaction mode
        match &mut self.state {
            SelectionState::Selecting { current_pos, adding_to_selection, .. } => {
                if primary_held {
                    *current_pos = pos;
                    *adding_to_selection = modifiers.shift; // Update for shift toggle
                }
            }
            SelectionState::Dragging { 
                current_pos, 
                grid_snap_enabled, 
                .. 
            } => {
                if primary_held {
                    *current_pos = pos;
                    *grid_snap_enabled = modifiers.ctrl; // Update for grid snap toggle
                }
            }
            SelectionState::Resizing { 
                current_pos,
                preserve_aspect_ratio,
                .. 
            } => {
                if primary_held {
                    *current_pos = pos;
                    *preserve_aspect_ratio = modifiers.shift; // Update for aspect ratio toggle
                }
            }
            SelectionState::Idle => {
                // In idle state, highlight resize handles when hovering
                let mut found_handle = false;
                
                for &element_id in editor_model.selected_ids() {
                    if let Some(element) = editor_model.find_element_by_id(element_id) {
                        let rect = compute_element_rect(element);
                        let handle_radius = RESIZE_HANDLE_RADIUS;
                        let corners = [
                            (rect.left_top(), Corner::TopLeft),
                            (rect.right_top(), Corner::TopRight),
                            (rect.left_bottom(), Corner::BottomLeft),
                            (rect.right_bottom(), Corner::BottomRight),
                        ];
                        
                        for (corner_pos, corner) in corners {
                            if is_near_handle_position(pos, corner_pos, handle_radius) {
                                renderer.set_active_handle(element_id, Some(corner));
                                found_handle = true;
                                break;
                            }
                        }
                        
                        if found_handle {
                            break;
                        }
                    }
                }
                
                if !found_handle {
                    renderer.clear_active_handles();
                }
            }
        }
        
        // Update the preview based on the current state
        self.update_preview(renderer);
        
        None // No command during pointer move
    }

    fn on_pointer_up(
        &mut self,
        _pos: Pos2,
        button: egui::PointerButton,
        _modifiers: &egui::Modifiers,
        editor_model: &EditorModel
    ) -> Option<Command> {
        // Only respond to primary button
        if button != egui::PointerButton::Primary {
            return None;
        }

        let result = match &self.state {
            SelectionState::Selecting { 
                start_pos, 
                current_pos, 
                adding_to_selection 
            } => {
                let selection_rect = egui::Rect::from_two_pos(*start_pos, *current_pos);
                
                // Only act if the selection has some size
                if selection_rect.width() > 2.0 || selection_rect.height() > 2.0 {
                    let mut ids = Vec::new();
                    
                    // Find elements that intersect with the selection rectangle
                    for &id in editor_model.selected_ids() {
                        if let Some(element) = editor_model.find_element_by_id(id) {
                            let element_rect = compute_element_rect(element);
                            if selection_rect.intersects(element_rect) {
                                ids.push(id);
                            }
                        }
                    }
                    
                    if !ids.is_empty() {
                        if *adding_to_selection {
                            // Add to current selection - toggle each element
                            let current_selection = editor_model.selected_ids();
                            
                            // Create a command to update selection
                            let mut previous_selection = current_selection.clone();
                            for id in ids {
                                if current_selection.contains(&id) {
                                    previous_selection.remove(&id);
                                } else {
                                    previous_selection.insert(id);
                                }
                            }
                            
                            Some(Command::ClearSelection {
                                previous_selection
                            })
                        } else {
                            // Replace current selection with new selection
                            Some(Command::ClearSelection {
                                previous_selection: ids.into_iter().collect()
                            })
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            SelectionState::Dragging { 
                start_pos, 
                current_pos, 
                initial_element_positions,
                original_rect: _original_rect,
                grid_snap_enabled,
            } => {
                // Only create a command if we actually moved
                if start_pos.distance(*current_pos) > 1.0 {
                    let delta = *current_pos - *start_pos;
                    let mut new_positions = std::collections::HashMap::new();
                    
                    for (&id, &initial_pos) in initial_element_positions {
                        let mut new_pos = initial_pos + delta;
                        
                        // Apply grid snapping if enabled
                        if *grid_snap_enabled {
                            const GRID_SIZE: f32 = 10.0;
                            new_pos.x = (new_pos.x / GRID_SIZE).round() * GRID_SIZE;
                            new_pos.y = (new_pos.y / GRID_SIZE).round() * GRID_SIZE;
                        }
                        
                        new_positions.insert(id, new_pos);
                    }
                    
                    // Create move commands for all elements
                    let mut commands = Vec::new();
                    for (id, new_pos) in new_positions {
                        if let Some(element) = editor_model.find_element_by_id(id) {
                            let old_pos = compute_element_rect(element).min;
                            let delta = new_pos - old_pos;
                            
                            // Only add a move command if we actually moved this element
                            if delta.x.abs() > 0.1 || delta.y.abs() > 0.1 {
                                commands.push(Command::MoveElement {
                                    element_id: id,
                                    _element_type: element.element_type().to_string(),
                                    _old_position: old_pos,
                                    new_position: new_pos,
                                });
                            }
                        }
                    }
                    
                    // Return the first move command if any
                    commands.into_iter().next()
                } else {
                    None
                }
            }
            SelectionState::Resizing { 
                element_id, 
                corner, 
                original_rect, 
                current_pos,
                preserve_aspect_ratio,
                .. 
            } => {
                // Calculate the new rectangle
                let new_rect = compute_resized_rect_with_constraints(
                    *original_rect, 
                    *corner, 
                    *current_pos,
                    *preserve_aspect_ratio
                );
                
                // Only create a command if the size actually changed
                if (new_rect.width() - original_rect.width()).abs() > 1.0 ||
                   (new_rect.height() - original_rect.height()).abs() > 1.0 {
                    Some(Command::ResizeElement {
                        element_id: *element_id,
                        _element_type: "unknown".to_string(),
                        _old_rect: *original_rect,
                        new_rect: new_rect,
                        _scaling_corner: *corner,
                        _original_image: egui::Image::new((egui::TextureId::default(), egui::Vec2::new(10.0, 10.0))),
                    })
                } else {
                    None
                }
            }
            SelectionState::Idle => None,
        };
        
        // Reset state regardless of whether a command was generated
        self.reset_interaction_state();
        
        result
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
                    ui.label(format!(
                        "Position: {:.1},{:.1}",
                        img.position().x,
                        img.position().y
                    ));
                }
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

        None // No immediate command from UI
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

    fn on_key(
        &mut self,
        key: egui::Key,
        pressed: bool,
        _modifiers: &egui::Modifiers,
        editor_model: &EditorModel
    ) {
        if pressed {
            match key {
                egui::Key::Delete | egui::Key::Backspace => {
                    // Delete selected elements
                    let selected_ids = editor_model.selected_ids();
                    if !selected_ids.is_empty() {
                        // Since we don't have a DeleteElements command, we need to delete them one by one
                        // For now, just delete the first selected element as an example
                        if let Some(&id) = selected_ids.iter().next() {
                            if let Some(_element) = editor_model.find_element_by_id(id) {
                                // We can no longer return commands, so we need to handle deletion differently
                                log::info!("Delete key pressed on element {}", id);
                            }
                        }
                    }
                }
                // TODO: Implement Copy/Paste when that functionality is available
                egui::Key::A if _modifiers.ctrl => {
                    // Select all elements - for now, just use the already selected elements
                    // This is a simplified version until we have proper access to all elements
                    log::info!("Ctrl+A pressed (select all)");
                }
                // Arrow keys for nudging selected elements
                egui::Key::ArrowLeft | egui::Key::ArrowRight | 
                egui::Key::ArrowUp | egui::Key::ArrowDown => {
                    let selected_id = editor_model.selected_ids().iter().next().copied();
                    if let Some(id) = selected_id {
                        let mut delta = egui::Vec2::ZERO;
                        let step = if _modifiers.shift { 10.0 } else { 1.0 };
                        
                        match key {
                            egui::Key::ArrowLeft => delta.x = -step,
                            egui::Key::ArrowRight => delta.x = step,
                            egui::Key::ArrowUp => delta.y = -step,
                            egui::Key::ArrowDown => delta.y = step,
                            _ => {}
                        }
                        
                        if let Some(_element) = editor_model.find_element_by_id(id) {
                            log::info!("Arrow key pressed on element {}, delta: {:?}", id, delta);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn update_preview(&mut self, renderer: &mut Renderer) {
        match &self.state {
            SelectionState::Selecting { start_pos, current_pos, .. } => {
                let selection_rect = egui::Rect::from_two_pos(*start_pos, *current_pos);
                renderer.set_resize_preview(Some(selection_rect));
            }
            SelectionState::Dragging { start_pos, current_pos, original_rect, .. } => {
                // Calculate the offset from start to current position
                let drag_offset = *current_pos - *start_pos;
                
                // Move the original rect by the drag offset
                let preview_rect = egui::Rect::from_min_size(
                    original_rect.min + drag_offset,
                    original_rect.size()
                );
                
                renderer.set_drag_preview(Some(preview_rect));
            }
            SelectionState::Resizing { element_id, corner, current_pos, original_rect, preserve_aspect_ratio, .. } => {
                // Calculate the new rectangle based on the resize operation
                let new_rect = if *preserve_aspect_ratio {
                    compute_resized_rect_with_constraints(*original_rect, *corner, *current_pos, true)
                } else {
                    Renderer::compute_resized_rect(*original_rect, *corner, *current_pos)
                };
                
                // Set the preview in the renderer
                renderer.set_resize_preview(Some(new_rect));
                renderer.set_active_handle(*element_id, Some(*corner));
            }
            SelectionState::Idle => {
                // Clear any previews
                renderer.set_resize_preview(None);
                renderer.set_drag_preview(None);
            }
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.clear_all_previews();
    }

    fn reset_interaction_state(&mut self) {
        self.state = SelectionState::Idle;
    }
}

pub fn new_selection_tool() -> UnifiedSelectionTool {
    UnifiedSelectionTool::new()
}

fn is_near_handle_position(pos: Pos2, handle_pos: Pos2, radius: f32) -> bool {
    let distance = (pos - handle_pos).length();
    distance <= radius
}

// Helper function to compute a resized rectangle with aspect ratio preservation
fn compute_resized_rect_with_constraints(
    original: egui::Rect,
    corner: Corner,
    new_pos: egui::Pos2,
    preserve_aspect_ratio: bool
) -> egui::Rect {
    if preserve_aspect_ratio {
        // Calculate original aspect ratio
        let original_width = original.width();
        let original_height = original.height();
        let aspect_ratio = original_width / original_height;
        
        // Determine the opposing corner based on which corner is being dragged
        let opposing_corner = match corner {
            Corner::TopLeft => original.right_bottom(),
            Corner::TopRight => original.left_bottom(),
            Corner::BottomLeft => original.right_top(),
            Corner::BottomRight => original.left_top(),
        };
        
        // Calculate the proposed width and height
        let proposed_rect = Renderer::compute_resized_rect(original, corner, new_pos);
        let proposed_width = proposed_rect.width();
        let proposed_height = proposed_rect.height();
        
        // Adjust to maintain aspect ratio, allowing the larger dimension to dominate
        if proposed_width / proposed_height > aspect_ratio {
            // Width is proportionally larger, so adjust height to match
            let adjusted_height = proposed_width / aspect_ratio;
            
            match corner {
                Corner::TopLeft => {
                    egui::Rect::from_min_max(
                        egui::pos2(opposing_corner.x - proposed_width, opposing_corner.y - adjusted_height),
                        opposing_corner,
                    )
                }
                Corner::TopRight => {
                    egui::Rect::from_min_max(
                        egui::pos2(opposing_corner.x, opposing_corner.y - adjusted_height),
                        egui::pos2(opposing_corner.x + proposed_width, opposing_corner.y),
                    )
                }
                Corner::BottomLeft => {
                    egui::Rect::from_min_max(
                        egui::pos2(opposing_corner.x - proposed_width, opposing_corner.y),
                        egui::pos2(opposing_corner.x, opposing_corner.y + adjusted_height),
                    )
                }
                Corner::BottomRight => {
                    egui::Rect::from_min_max(
                        opposing_corner,
                        egui::pos2(opposing_corner.x + proposed_width, opposing_corner.y + adjusted_height),
                    )
                }
            }
        } else {
            // Height is proportionally larger, so adjust width to match
            let adjusted_width = proposed_height * aspect_ratio;
            
            match corner {
                Corner::TopLeft => {
                    egui::Rect::from_min_max(
                        egui::pos2(opposing_corner.x - adjusted_width, opposing_corner.y - proposed_height),
                        opposing_corner,
                    )
                }
                Corner::TopRight => {
                    egui::Rect::from_min_max(
                        egui::pos2(opposing_corner.x, opposing_corner.y - proposed_height),
                        egui::pos2(opposing_corner.x + adjusted_width, opposing_corner.y),
                    )
                }
                Corner::BottomLeft => {
                    egui::Rect::from_min_max(
                        egui::pos2(opposing_corner.x - adjusted_width, opposing_corner.y),
                        egui::pos2(opposing_corner.x, opposing_corner.y + proposed_height),
                    )
                }
                Corner::BottomRight => {
                    egui::Rect::from_min_max(
                        opposing_corner,
                        egui::pos2(opposing_corner.x + adjusted_width, opposing_corner.y + proposed_height),
                    )
                }
            }
        }
    } else {
        // Just use the standard resizing logic
        Renderer::compute_resized_rect(original, corner, new_pos)
    }
}
