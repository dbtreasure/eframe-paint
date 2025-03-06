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

        // Route the event to the active tool
        match event {
            InputEvent::PointerDown { location, button } if *button == egui::PointerButton::Primary => {
                let position = location.position;
                info!("Tool: pointer down at {:?}", position);
                
                // Create a temporary copy of the state for the on_pointer_down call
                let state_copy = state.clone();
                
                // Handle the tool's pointer down event
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
            
            InputEvent::PointerMove { location, held_buttons } => {
                // Handle pointer move regardless of whether buttons are held
                let position = location.position;
                info!("Tool: pointer move at {:?}", position);
                
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
            
            InputEvent::PointerUp { location, button } if *button == egui::PointerButton::Primary => {
                // Handle pointer up
                let position = location.position;
                info!("Tool: pointer up at {:?}", position);
                
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