use crate::PaintApp;
use crate::command::CommandHistory;
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::input::InputEvent;
use egui;

pub struct CentralPanel {
}

impl CentralPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_input_event(
        &self,
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
        
        match event {
            InputEvent::PointerDown { location, button } 
                if *button == egui::PointerButton::Primary => {
                // Check if we have an active tool
                let mut state_builder = state.builder();
                
                if let Some(mut active_tool) = state_builder.take_active_tool() {
                    let position = location.position;
                    
                    // Handle selection if the selection tool is active
                    if active_tool.is_selection_tool() {
                        // Use the new element_at_position method to get the element at the cursor position
                        let selected_element = document.element_at_position(position);
                        
                        // Update the state builder with the selected element (or none)
                        state_builder = match selected_element {
                            Some(element) => state_builder.with_selected_elements(vec![element]),
                            None => state_builder.with_selected_elements(vec![]),
                        };
                        
                        // Let the tool handle its own state transitions based on selection
                        // This will be handled in the on_pointer_down method of the tool
                    }
                    
                    // Process the tool's pointer down event
                    if let Some(cmd) = active_tool.on_pointer_down(location.position, document) {
                        command_history.execute(cmd, document);
                    }
                    
                    // Update preview using the tool's trait method
                    active_tool.update_preview(renderer);
                    
                    // Update the state with the modified tool
                    *state = state_builder
                        .with_active_tool(Some(active_tool))
                        .build();
                }
            }
            
            InputEvent::PointerMove { location, held_buttons } => {
                // Handle pointer move regardless of whether buttons are held
                let mut state_builder = state.builder();
                
                if let Some(mut active_tool) = state_builder.take_active_tool() {                    
                    // Process the tool's pointer move event
                    if let Some(cmd) = active_tool.on_pointer_move(location.position, document) {
                        command_history.execute(cmd, document);
                    }
                    
                    // Update preview if a button is held
                    if held_buttons.contains(&egui::PointerButton::Primary) {
                        active_tool.update_preview(renderer);
                    }
                    
                    // Update the state with the modified tool
                    *state = state_builder
                        .with_active_tool(Some(active_tool))
                        .build();
                }
            }
            
            InputEvent::PointerUp { location, button } 
                if *button == egui::PointerButton::Primary => {
                // Use the active tool to handle the pointer up event
                let mut state_builder = state.builder();
                
                if let Some(mut active_tool) = state_builder.take_active_tool() {
                    if let Some(cmd) = active_tool.on_pointer_up(location.position, document) {
                        command_history.execute(cmd, document);
                    }
                    
                    // Clear preview using the tool's trait method
                    active_tool.clear_preview(renderer);
                    
                    // Update the state with the modified tool
                    *state = state_builder
                        .with_active_tool(Some(active_tool))
                        .build();
                }
            }
            
            InputEvent::PointerEnter { location } => {
                // Check if we have an active tool
                let mut state_builder = state.builder();
                
                if let Some(mut active_tool) = state_builder.take_active_tool() {
                    // Only handle for selection tool in TextureSelected state
                    if active_tool.is_selection_tool() {
                        // Process the tool's pointer move event (which handles hover detection)
                        if let Some(cmd) = active_tool.on_pointer_move(location.position, document) {
                            command_history.execute(cmd, document);
                        }
                        
                        // Update the state with the modified tool
                        *state = state_builder
                            .with_active_tool(Some(active_tool))
                            .build();
                    } else {
                        // Put the tool back if we didn't use it
                        *state = state_builder
                            .with_active_tool(Some(active_tool))
                            .build();
                    }
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
                    } else {
                        // For other tools, use the default cursor
                        ctx.set_cursor_icon(egui::CursorIcon::Default);
                    }
                }
            }
        }
    });
}