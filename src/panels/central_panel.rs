use crate::command::Command;
use crate::command::CommandHistory;
use crate::element::{Element, ElementType};
use crate::input::InputEvent;
use crate::renderer::Renderer;
use crate::state::EditorModel;
use crate::tools::Tool;
use egui;
use log::info;

pub struct CentralPanel {}

impl CentralPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_input_event(
        &mut self,
        event: &InputEvent,
        command_history: &mut CommandHistory,
        renderer: &mut Renderer,
        panel_rect: egui::Rect,
        ui: &egui::Ui,
        editor_model: &mut EditorModel,
    ) {
        if !self.is_event_in_panel(event, panel_rect) {
            return;
        }

        let mut cmd_result = None;

        // Route the event to the active tool
        match event {
            InputEvent::PointerDown { location, button }
                if *button == egui::PointerButton::Primary =>
            {
                let position = location.position;
                info!("Tool: pointer down at {:?}", position);

                // Get the active tool from the editor model
                let mut tool = editor_model.active_tool().clone();

                // Process the tool's pointer down event
                cmd_result = tool.on_pointer_down(position, editor_model);

                // Add this line to update the preview after handling the down event
                tool.update_preview(renderer);

                // Update the tool in the editor model
                editor_model.update_tool(|_| tool);

                // If no command was returned, handle selection
                if cmd_result.is_none() {
                    // Check if we clicked on an element for selection handling
                    if let Some(element) = editor_model.element_at_position(position) {
                        // Log the element we found
                        info!(
                            "Found element at position {:?}: ID={}",
                            position,
                            element.id()
                        );
                        match &element {
                            ElementType::Image(img) => {
                                info!(
                                    "Found image: ID={}, size={:?}, pos={:?}",
                                    img.id(),
                                    img.size(),
                                    img.position()
                                );
                            }
                            ElementType::Stroke(stroke) => {
                                info!(
                                    "Found stroke: ID={}, points={}",
                                    stroke.id(),
                                    stroke.points().len()
                                );
                            }
                        }

                        // Check if this element is already selected
                        let is_already_selected =
                            editor_model.is_element_selected(element.id());

                        // If not already selected, update the selection
                        if !is_already_selected {
                            info!(
                                "Updating selection to element ID: {}",
                                element.id()
                            );

                            // Create a selection command
                            let select_cmd = Command::SelectElement(element.id());

                            // Execute the command and handle any errors
                            let _ = command_history
                                .execute(select_cmd, editor_model)
                                .map_err(|err| log::warn!("Selection command failed: {}", err));
                        }
                    } else {
                        // Clicked on empty space, clear selection
                        info!("Clearing selection (clicked on empty space)");

                        // Create a clear selection command that properly stores the previous selection
                        let clear_cmd = Command::new_clear_selection(editor_model);

                        // Execute the command and handle any errors
                        let _ = command_history
                            .execute(clear_cmd, editor_model)
                            .map_err(|err| log::warn!("Clear selection command failed: {}", err));
                    }
                }
            }
            InputEvent::PointerMove {
                location,
                held_buttons,
            } if held_buttons.contains(&egui::PointerButton::Primary) => {
                let position = location.position;

                // Get the active tool from the editor model
                let mut tool = editor_model.active_tool().clone();

                // Process the tool's pointer move event
                cmd_result = tool.on_pointer_move(position, editor_model, ui, renderer);

                // Update the tool in the editor model
                editor_model.update_tool(|_| tool);
            }
            InputEvent::PointerUp { location, button }
                if *button == egui::PointerButton::Primary =>
            {
                let position = location.position;
                info!("Tool: pointer up at {:?}", position);

                // Get the active tool from the editor model
                let mut tool = editor_model.active_tool().clone();

                // Process the tool's pointer up event
                cmd_result = tool.on_pointer_up(position, editor_model);

                // Update the tool in the editor model
                editor_model.update_tool(|_| tool);
            }
            _ => {}
        }

        // If a command was returned, execute it
        if let Some(cmd) = cmd_result {
            info!("Executing command from input event");

            // Execute the command on editor_model and handle any errors
            let _ = command_history
                .execute(cmd.clone(), editor_model)
                .map_err(|err| log::warn!("Tool command execution failed: {}", err));

            // Invalidate textures to ensure proper rendering
            cmd.invalidate_textures(renderer);
        }
    }

    fn is_event_in_panel(&self, event: &InputEvent, panel_rect: egui::Rect) -> bool {
        match event {
            InputEvent::PointerDown { location, .. }
            | InputEvent::PointerMove { location, .. }
            | InputEvent::PointerUp { location, .. } => panel_rect.contains(location.position),
            _ => true,
        }
    }
}

pub fn central_panel(
    editor_model: &mut EditorModel,
    command_history: &mut CommandHistory,
    renderer: &mut Renderer,
    ctx: &egui::Context,
) -> egui::Rect {
    let panel_response = egui::CentralPanel::default().show(ctx, |ui| {
        // Get the panel rect for hit testing
        let panel_rect = ui.max_rect();

        // Render the document with the UI
        renderer.render(ui, editor_model, panel_rect);

        // Create a temporary central panel for handling input
        let mut central_panel = CentralPanel::new();

        // Process input events directly
        let events = crate::input::InputHandler::process_input_static(ctx, panel_rect);
        for event in events {
            central_panel.handle_input_event(
                &event,
                command_history,
                renderer,
                panel_rect,
                ui,
                editor_model,
            );
        }

        // Return the panel rect for the caller to use
        panel_rect
    });

    // Request continuous rendering if we're interacting with the panel
    if panel_response.response.hovered() || panel_response.response.dragged() {
        ctx.request_repaint();
    }

    // Return the panel rect
    panel_response.response.rect
}
