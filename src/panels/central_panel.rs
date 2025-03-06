use crate::PaintApp;
use crate::command::CommandHistory;
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::input::InputEvent;
use egui;
use crate::geometry::hit_testing::HitTestCache;
use std::sync::Arc;
use log::info;
use crate::tools::Tool;

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
        
        let mut cmd_result = None;

        // Route the event to the active tool if available
        if let Some(active_tool) = state.active_tool() {
            match event {
                InputEvent::PointerDown { location, button } if *button == egui::PointerButton::Primary => {
                    let pos = location.position;
                    info!("Tool: pointer down at {:?}", pos);
                    
                    // We need to update the state with a new tool instance that has handled the event
                    // This is a temporary solution until we refactor the architecture
                    // In a real implementation, we would use a command pattern or similar
                }
                _ => {}
            }
        } else {
            // Existing old code path
            match event {
                InputEvent::PointerDown { location, button } 
                    if *button == egui::PointerButton::Primary => {
                    // Check if we have an active tool
                    let position = location.position;
                    
                    // First, handle selection if the selection tool is active
                    if let Some(active_tool) = state.active_tool() {
                        if active_tool.is_selection_tool() {
                            
                            // Check if the click is on a resize handle
                            let is_on_handle = state.selected_elements().iter().any(|element| {
                                crate::geometry::hit_testing::is_point_near_handle(position, element)
                            });
                            
                            // Only update selection if no handle is being dragged AND not clicking on a handle
                            if !renderer.any_handles_active() && !is_on_handle { 
                                info!("Updating selection");
                                let selected_element = document.element_at_position(position);
                                
                                // Update the state with the selected element (or none)
                                *state = state.update_selection(|_selected_elements| {
                                    match selected_element {
                                        Some(element) => vec![element],
                                        None => vec![],
                                    }
                                });
                            } else if is_on_handle {
                                info!("Click on resize handle, keeping selection");
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
                }
                
                InputEvent::PointerMove { location, held_buttons: _ } => {
                    // Handle pointer move regardless of whether buttons are held
                    let position = location.position;
                    
                    // Create a temporary copy of the state for the on_pointer_move call
                    let state_copy = state.clone();
                    
                    // Handle the tool's pointer move event
                    state.with_tool_mut(|active_tool| {
                        if let Some(tool) = active_tool {
                            // Directly modify the tool in place
                            let tool_ref = Arc::make_mut(tool);
                            
                            // Process the tool's pointer move event
                            cmd_result = tool_ref.on_pointer_move(position, document, &state_copy);
                            
                            // Update preview using the tool's trait method
                            tool_ref.update_preview(renderer);
                        }
                    });
                }
                
                InputEvent::PointerUp { location, button } 
                    if *button == egui::PointerButton::Primary => {
                    // Handle pointer up
                    let position = location.position;
                    
                    // Create a temporary copy of the state for the on_pointer_up call
                    let state_copy = state.clone();
                    
                    // Handle the tool's pointer up event
                    state.with_tool_mut(|active_tool| {
                        if let Some(tool) = active_tool {
                            // Directly modify the tool in place
                            let tool_ref = Arc::make_mut(tool);
                            
                            // Process the tool's pointer up event
                            cmd_result = tool_ref.on_pointer_up(position, document, &state_copy);
                            
                            // Update preview using the tool's trait method
                            tool_ref.update_preview(renderer);
                        }
                    });
                    
                    // Clear any active resize handles
                    renderer.clear_all_active_handles();
                }
                
                _ => {}
            }
        }
        
        // If the tool returned a command, execute it
        if let Some(cmd) = cmd_result {
            command_history.execute(cmd, document);
        }
    }

    fn is_event_in_panel(&self, event: &InputEvent, panel_rect: egui::Rect) -> bool {
        match event {
            InputEvent::PointerDown { location, .. } |
            InputEvent::PointerMove { location, .. } |
            InputEvent::PointerUp { location, .. } => {
                panel_rect.contains(location.position)
            }
            _ => true,
        }
    }
}

pub fn central_panel(app: &mut PaintApp, ctx: &egui::Context) {
    let panel_response = egui::CentralPanel::default()
        .show(ctx, |ui| {
            // Get the panel rect for hit testing
            let panel_rect = ui.max_rect();
            
            // Store the panel rect for future use
            app.set_central_panel_rect(panel_rect);
            
            // Render the document with the UI
            app.render(ctx, ui, panel_rect);
            
            // Handle input events
            app.handle_input(ctx);
        });
    
    // Request continuous rendering if we're interacting with the panel
    if panel_response.response.hovered() || panel_response.response.dragged() {
        ctx.request_repaint();
    }
}