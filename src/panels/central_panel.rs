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
                if let Some(active_tool) = state.active_tool().cloned() {
                    let position = location.position;
                    
                    // Handle selection if the selection tool is active
                    if active_tool.is_selection_tool() {
                        let mut element_clicked = false;
                        let mut selected_element = None;
                        
                        // Check if we clicked on a stroke
                        for stroke in document.strokes() {
                            let points = stroke.points();
                            if points.len() < 2 {
                                continue;
                            }
                            
                            for window in points.windows(2) {
                                let line_start = window[0];
                                let line_end = window[1];
                                
                                // Calculate distance from point to line segment
                                let distance = distance_to_line_segment(position, line_start, line_end);
                                
                                // If the distance is less than the stroke thickness plus a small margin, consider it a hit
                                if distance <= stroke.thickness() + 2.0 {
                                    // We clicked on this stroke, select it
                                    selected_element = Some(crate::state::ElementType::Stroke(stroke.clone()));
                                    element_clicked = true;
                                    break;
                                }
                            }
                            
                            if element_clicked {
                                break;
                            }
                        }
                        
                        // If we didn't click on a stroke, check if we clicked on an image
                        if !element_clicked {
                            for image in document.images() {
                                let rect = image.rect();
                                if rect.contains(position) {
                                    // We clicked on this image, select it
                                    selected_element = Some(crate::state::ElementType::Image(image.clone()));
                                    break;
                                }
                            }
                        }
                        
                        // Update the state with the selected element (or none)
                        let current_state = state.clone();
                        *state = current_state.with_selected_element(selected_element);
                    }
                    
                    // Process the tool's pointer down event
                    let mut tool_clone = active_tool.clone();
                    
                    if let Some(cmd) = tool_clone.on_pointer_down(location.position, document) {
                        command_history.execute(cmd, document);
                    }
                    
                    // Update preview using the tool's trait method
                    tool_clone.update_preview(renderer);
                    
                    // Update the state with the modified tool
                    let current_state = state.clone();
                    *state = current_state.with_active_tool(Some(tool_clone));
                }
            }
            
            InputEvent::PointerMove { location, held_buttons } => {
                if held_buttons.contains(&egui::PointerButton::Primary) {
                    // Use the active tool to handle the pointer move event
                    if let Some(active_tool) = state.active_tool() {
                        let mut tool_clone = active_tool.clone();
                        
                        if let Some(cmd) = tool_clone.on_pointer_move(location.position, document) {
                            command_history.execute(cmd, document);
                        }
                        
                        // Update preview using the tool's trait method
                        tool_clone.update_preview(renderer);
                        
                        // Update the state with the modified tool
                        *state = state.clone().with_active_tool(Some(tool_clone));
                    }
                }
            }
            
            InputEvent::PointerUp { location, button } 
                if *button == egui::PointerButton::Primary => {
                // Use the active tool to handle the pointer up event
                if let Some(active_tool) = state.active_tool() {
                    let mut tool_clone = active_tool.clone();
                    
                    if let Some(cmd) = tool_clone.on_pointer_up(location.position, document) {
                        command_history.execute(cmd, document);
                    }
                    
                    // Clear preview using the tool's trait method
                    tool_clone.clear_preview(renderer);
                    
                    // Update the state with the modified tool
                    *state = state.clone().with_active_tool(Some(tool_clone));
                }
            }
            
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
                        
                        if document.is_point_over_stroke(pointer_pos) {
                            // Set cursor to a "move" cursor when over a stroke
                            ctx.set_cursor_icon(egui::CursorIcon::Move);
                        } else if document.is_point_over_image(pointer_pos) {
                            // Set cursor to a "grab" cursor when over an image
                            ctx.set_cursor_icon(egui::CursorIcon::Grab);
                        } else {
                            // Reset to default cursor
                            ctx.set_cursor_icon(egui::CursorIcon::Default);
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

// Helper function to calculate distance from a point to a line segment
fn distance_to_line_segment(point: egui::Pos2, line_start: egui::Pos2, line_end: egui::Pos2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    
    let line_len_sq = line_vec.x * line_vec.x + line_vec.y * line_vec.y;
    
    // If the line segment is actually a point
    if line_len_sq == 0.0 {
        return point_vec.length();
    }
    
    // Calculate projection of point_vec onto line_vec
    let t = (point_vec.x * line_vec.x + point_vec.y * line_vec.y) / line_len_sq;
    
    if t < 0.0 {
        // Closest point is line_start
        return point_vec.length();
    } else if t > 1.0 {
        // Closest point is line_end
        return (point - line_end).length();
    } else {
        // Closest point is on the line segment
        let closest = line_start + line_vec * t;
        return (point - closest).length();
    }
}