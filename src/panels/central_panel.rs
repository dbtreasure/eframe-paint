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
                
                // Variables to track selection updates
                let mut should_update_selection = false;
                let mut new_element = None;
                
                // Handle the tool's pointer down event
                state.with_tool_mut(|active_tool| {
                    if let Some(tool) = active_tool {
                        // Check if this is the selection tool
                        let is_selection_tool = tool.name() == "Selection";
                        
                        // Directly modify the tool in place
                        let tool_ref = Arc::make_mut(tool);
                        
                        // Process the tool's pointer down event
                        cmd_result = tool_ref.on_pointer_down(position, document, &state_copy);
                        
                        // If this is the selection tool and no command was returned,
                        // check if we need to update the selection
                        if is_selection_tool && cmd_result.is_none() {
                            // Check if we clicked on an element
                            if let Some(element) = document.element_at_position(position) {
                                // Check if this element is already selected
                                let is_already_selected = if let Some(selected) = state_copy.selected_element() {
                                    match (selected, &element) {
                                        (crate::state::ElementType::Image(sel_img), crate::state::ElementType::Image(hit_img)) => 
                                            sel_img.id() == hit_img.id(),
                                        (crate::state::ElementType::Stroke(sel_stroke), crate::state::ElementType::Stroke(hit_stroke)) => {
                                            // Use stable IDs for comparison
                                            let sel_element = crate::state::ElementType::Stroke(sel_stroke.clone());
                                            let hit_element = crate::state::ElementType::Stroke(hit_stroke.clone());
                                            sel_element.get_stable_id() == hit_element.get_stable_id()
                                        },
                                        _ => false,
                                    }
                                } else {
                                    false
                                };
                                
                                // If not already selected, update the selection
                                if !is_already_selected {
                                    info!("Updating selection with element");
                                    should_update_selection = true;
                                    new_element = Some(element.clone());
                                }
                            }
                        }
                        
                        // Update preview using the tool's trait method
                        tool_ref.update_preview(renderer);
                    }
                });
                
                // Update selection if needed (outside the closure to avoid borrow issues)
                if should_update_selection && new_element.is_some() {
                    *state = state.update_selection(|_| vec![new_element.unwrap()]);
                }
            }
            
            InputEvent::PointerMove { location, held_buttons: _ } => {
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
        
        // If the tool returned a command, execute it and mark for redraw
        if let Some(cmd_result) = cmd_result {
            // Handle different command types before executing
            match &cmd_result {
                crate::command::Command::AddStroke(stroke) => {
                    let element = crate::state::ElementType::Stroke(stroke.clone());
                    renderer.handle_element_update(&element);
                    // Add explicit logging for stroke rendering
                    info!("🔄 Adding stroke with ID: {}, requesting redraw", stroke.id());
                    
                    // Force document version to increment by a larger value to ensure redraw
                    for _ in 0..5 {
                        document.mark_modified();
                    }
                },
                crate::command::Command::AddImage(image) => {
                    let element = crate::state::ElementType::Image(image.clone());
                    renderer.handle_element_update(&element);
                    info!("🔄 Adding image with ID: {}, requesting redraw", image.id());
                    
                    // Force document version to increment by a larger value to ensure redraw
                    for _ in 0..5 {
                        document.mark_modified();
                    }
                },
                _ => {}
            }
            
            // Execute the command
            command_history.execute(cmd_result, document);
            
            // Force document to be marked as modified multiple times to ensure redraw
            for _ in 0..5 {
                document.mark_modified();
            }
            
            // Reset renderer state to ensure full redraw
            renderer.reset_state();
            
            // Force immediate repaint
            renderer.get_ctx().request_repaint();
            
            // Additional logging
            info!("✅ Command executed, document version now: {}", document.version());
            
            // Log what's in the document after the command
            info!("📝 Document now contains {} strokes and {} images", 
                 document.strokes().len(), document.images().len());
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
            
            // Add debugging overlay that can be toggled with alt/option key
            if cfg!(debug_assertions) && ctx.input(|i| i.modifiers.alt) {
                ui.label(egui::RichText::new("DEBUG: Holding ALT to show debug info")
                    .background_color(egui::Color32::from_rgb(40, 40, 80))
                    .color(egui::Color32::WHITE));
                
                // Display element rectangles for debugging
                if let Some(selected) = app.state().selected_element() {
                    // Use compute_element_rect instead of the private rect() method
                    let rect = crate::geometry::hit_testing::compute_element_rect(&selected);
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::RED),
                    );
                    ui.painter().text(
                        rect.min,
                        egui::Align2::LEFT_TOP,
                        format!("ID: {}", selected.get_stable_id()),
                        egui::FontId::monospace(14.0),
                        egui::Color32::RED,
                    );
                }
            }
            
            // Handle input events
            app.handle_input(ctx);
        });
    
    // Request continuous rendering if we're interacting with the panel
    if panel_response.response.hovered() || panel_response.response.dragged() {
        ctx.request_repaint();
    }
}