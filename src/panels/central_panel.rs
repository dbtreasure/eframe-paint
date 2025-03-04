use crate::PaintApp;
use crate::command::CommandHistory;
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::input::InputEvent;
use egui;
use crate::geometry::hit_testing::HitTestCache;
use std::sync::Arc;

pub struct CentralPanel {
    hit_test_cache: HitTestCache,
}

impl CentralPanel {
    pub fn new() -> Self {
        Self {
            hit_test_cache: HitTestCache::new(),
        }
    }

    pub fn handle_input_event(
        &mut self,
        event: &InputEvent,
        state: &mut EditorState,
        document: &mut Document,
        command_history: &mut CommandHistory,
        renderer: &mut Renderer,
        panel_rect: egui::Rect,
    ) {
        if !self.is_event_in_panel(event, panel_rect) {
            return;
        }
        
        // Update hit test cache with current state
        self.hit_test_cache.update(state);
        
        match event {
            InputEvent::PointerDown { location, button } 
                if *button == egui::PointerButton::Primary => {
                // Check if we have an active tool
                let position = location.position;
                let mut cmd_result = None;
                
                // First, handle selection if the selection tool is active
                if let Some(active_tool) = state.active_tool() {
                    if active_tool.is_selection_tool() {
                        // Check if we're clicking on a resize handle using the cache
                        let is_on_resize_handle = self.hit_test_cache.is_point_near_any_handle(position);
                        
                        // Only check for element at position and update selection if NOT on a resize handle
                        if !is_on_resize_handle {
                            // Use the new element_at_position method to get the element at the cursor position
                            let selected_element = document.element_at_position(position);
                            
                            // Update the state with the selected element (or none)
                            *state = state.update_selection(|_selected_elements| {
                                match selected_element {
                                    Some(element) => vec![element],
                                    None => vec![],
                                }
                            });
                        }
                    }
                }
                
                // Create a temporary copy of the state for the on_pointer_down call
                let state_copy = state.clone();
                
                // Now handle the tool's pointer down event without cloning
                state.with_tool_mut(|active_tool| {
                    if let Some(tool) = active_tool {
                        // Directly modify the tool in place
                        let tool_ref = Arc::make_mut(tool);
                        
                        // Process the tool's pointer down event
                        cmd_result = tool_ref.on_pointer_down(position, document, &state_copy);
                        
                        // Update preview using the tool's trait method
                        tool_ref.update_preview(renderer);
                    }
                });
                
                // If the tool returned a command, execute it
                if let Some(cmd) = cmd_result {
                    command_history.execute(cmd, document);
                }
            }
            
            InputEvent::PointerMove { location, held_buttons } => {
                // Handle pointer move regardless of whether buttons are held
                let position = location.position;
                let mut cmd_result = None;
                
                // Create a temporary copy of the state for the on_pointer_move call
                let state_copy = state.clone();
                
                // Update the tool state without cloning
                state.with_tool_mut(|active_tool| {
                    if let Some(tool) = active_tool {
                        // Directly modify the tool in place
                        let tool_ref = Arc::make_mut(tool);
                        
                        // Process the tool's pointer move event using the state copy
                        cmd_result = tool_ref.on_pointer_move(position, document, &state_copy);
                        
                        // Update preview if a button is held
                        if held_buttons.contains(&egui::PointerButton::Primary) {
                            tool_ref.update_preview(renderer);
                        }
                    }
                });
                
                // If the tool returned a command, execute it
                if let Some(cmd) = cmd_result {
                    command_history.execute(cmd, document);
                }
            }
            
            InputEvent::PointerUp { location, button } 
                if *button == egui::PointerButton::Primary => {
                // Use the active tool to handle the pointer up event
                let position = location.position;
                let mut cmd_result = None;
                
                // Create a temporary copy of the state for the on_pointer_up call
                let state_copy = state.clone();
                
                // Update the tool state without cloning
                state.with_tool_mut(|active_tool| {
                    if let Some(tool) = active_tool {
                        // Directly modify the tool in place
                        let tool_ref = Arc::make_mut(tool);
                        
                        // Process the tool's pointer up event
                        cmd_result = tool_ref.on_pointer_up(position, document, &state_copy);
                        
                        // Clear preview using the tool's trait method
                        tool_ref.clear_preview(renderer);
                    }
                });
                
                // If the tool returned a command, execute it
                if let Some(cmd) = cmd_result {
                    command_history.execute(cmd, document);
                }
            }
            
            InputEvent::PointerEnter { location } => {
                // Check if we have an active tool
                let position = location.position;
                let mut cmd_result = None;
                
                // Create a temporary copy of the state for the on_pointer_move call
                let state_copy = state.clone();
                
                // Only handle for selection tool without cloning
                state.with_tool_mut(|active_tool| {
                    if let Some(tool) = active_tool {
                        // Only handle for selection tool
                        if Arc::make_mut(tool).is_selection_tool() {
                            // Directly modify the tool in place
                            let tool_ref = Arc::make_mut(tool);
                            
                            // Process the tool's pointer move event (which handles hover detection)
                            cmd_result = tool_ref.on_pointer_move(position, document, &state_copy);
                        }
                    }
                });
                
                // If the tool returned a command, execute it
                if let Some(cmd) = cmd_result {
                    command_history.execute(cmd, document);
                }
            },
            
            _ => {}
        }
    }
    
    fn is_event_in_panel(&self, event: &InputEvent, panel_rect: egui::Rect) -> bool {
        match event {
            InputEvent::PointerDown { location, .. } |
            InputEvent::PointerUp { location, .. } |
            InputEvent::PointerMove { location, .. } |
            InputEvent::PointerEnter { location } => {
                panel_rect.contains(location.position)
            },
            InputEvent::PointerLeave { last_known_location } => {
                panel_rect.contains(last_known_location.position)
            },
        }
    }
}

pub fn central_panel(app: &mut PaintApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let canvas_rect = ui.available_rect_before_wrap();
        app.set_central_panel_rect(canvas_rect);
        
        let painter = ui.painter();
        
        // Render directly from the app, passing all needed components
        // This avoids borrowing conflicts by letting the app manage access to its components
        app.render(ctx, painter, canvas_rect);
        
        // Update cursor based on what's under the pointer, but only if selection tool is active
        if let Some(pointer_pos) = ctx.pointer_hover_pos() {
            if canvas_rect.contains(pointer_pos) {
                // Only change cursor if the selection tool is active
                if let Some(active_tool) = app.active_tool() {
                    if active_tool.is_selection_tool() {
                        // Get document reference from app to check for strokes/images
                        let document = app.document();
                        
                        // Optimize cursor handling based on selection state
                        let state = app.state();
                        if !state.selected_elements().is_empty() {
                            // If elements are selected, use Move cursor
                            ctx.set_cursor_icon(egui::CursorIcon::Move);
                        } else {
                            // Use the new element_at_position method to determine what's under the cursor
                            match document.element_at_position(pointer_pos) {
                                Some(crate::state::ElementType::Stroke(_)) => {
                                    // Set cursor to a "move" cursor when over a stroke
                                    ctx.set_cursor_icon(egui::CursorIcon::Move);
                                },
                                Some(crate::state::ElementType::Image(_)) => {
                                    // Set cursor to a "grab" cursor when over an image
                                    ctx.set_cursor_icon(egui::CursorIcon::Grab);
                                },
                                None => {
                                    // Reset to default cursor
                                    ctx.set_cursor_icon(egui::CursorIcon::Default);
                                }
                            }
                        }
                    } else {
                        // For other tools, use the default cursor
                        ctx.set_cursor_icon(egui::CursorIcon::Default);
                    }
                }
            }
        }
    });
}