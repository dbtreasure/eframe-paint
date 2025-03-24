use crate::command::Command;
use crate::command::CommandHistory;
use crate::state::EditorModel;
use crate::renderer::Renderer;
use crate::tools::{Tool, ToolType};
use egui;
use log::info;

/// A panel for the main editing area of the application
pub struct CentralPanel {
    last_pointer_pos: Option<egui::Pos2>,
    request_repaint: bool,
}

impl CentralPanel {
    pub fn new() -> Self {
        Self {
            last_pointer_pos: None,
            request_repaint: false,
        }
    }
    
    /// Handle pointer events (mouse down/move/up) and delegate to the active tool
    fn handle_pointer_events(
        &mut self,
        ctx: &egui::Context,
        pos: egui::Pos2,
        editor_model: &mut EditorModel,
        command_history: &mut CommandHistory,
        renderer: &mut Renderer,
        ui: &egui::Ui,
    ) {
        // Get input state from egui
        let modifiers = ctx.input(|i| i.modifiers);
        
        // Handle pointer down events
        for button in [egui::PointerButton::Primary, egui::PointerButton::Secondary] {
            if ctx.input(|i| i.pointer.button_pressed(button)) {
                info!("Tool: pointer down at {:?} with button {:?}", pos, button);
                
                // Get a clone of the active tool to avoid borrow issues
                let mut tool = editor_model.active_tool().clone();
                let cmd = tool.on_pointer_down(
                    pos, 
                    button, 
                    &modifiers,
                    editor_model,
                    renderer,
                );
                
                // Update the tool in the model
                editor_model.update_tool(|_| tool);
                
                if let Some(cmd) = cmd {
                    info!("Tool generated command from pointer down: {:?}", cmd);
                    self.execute_command(cmd, command_history, editor_model, renderer);
                    return; // Stop processing after executing a command
                }
            }
        }
        
        // Handle pointer move events
        if self.last_pointer_pos != Some(pos) || ctx.input(|i| i.pointer.any_down()) {
            // Get all held buttons
            let held_buttons: Vec<_> = [
                egui::PointerButton::Primary,
                egui::PointerButton::Secondary,
                egui::PointerButton::Middle,
            ]
            .iter()
            .filter(|&&button| ctx.input(|i| i.pointer.button_down(button)))
            .copied()
            .collect();
            
            if !held_buttons.is_empty() || self.last_pointer_pos != Some(pos) {
                // Update last known position
                self.last_pointer_pos = Some(pos);
                
                // Get a clone of the active tool to avoid borrow issues
                let mut tool = editor_model.active_tool().clone();
                let cmd = tool.on_pointer_move(
                    pos,
                    &held_buttons,
                    &modifiers,
                    editor_model,
                    ui,
                    renderer,
                );
                
                // Update the tool in the model
                editor_model.update_tool(|_| tool);
                
                if let Some(cmd) = cmd {
                    info!("Tool generated command from pointer move: {:?}", cmd);
                    self.execute_command(cmd, command_history, editor_model, renderer);
                    return; // Stop processing after executing a command
                }
            }
        }
        
        // Handle pointer up events
        for button in [egui::PointerButton::Primary, egui::PointerButton::Secondary] {
            if ctx.input(|i| i.pointer.button_released(button)) {
                info!("Tool: pointer up at {:?} with button {:?}", pos, button);
                
                // Get a clone of the active tool to avoid borrow issues
                let mut tool = editor_model.active_tool().clone();
                let cmd = tool.on_pointer_up(
                    pos, 
                    button, 
                    &modifiers,
                    editor_model,
                );
                
                // Update the tool in the model
                editor_model.update_tool(|_| tool);
                
                if let Some(cmd) = cmd {
                    info!("Tool generated command from pointer up: {:?}", cmd);
                    self.execute_command(cmd, command_history, editor_model, renderer);
                    return; // Stop processing after executing a command
                }
            }
        }
        
        // Always update preview after handling events
        let mut tool = editor_model.active_tool().clone();
        tool.update_preview(renderer);
        editor_model.update_tool(|_| tool);
    }
    
    /// Handle keyboard events and delegate to the active tool
    fn handle_keyboard_events(
        &mut self,
        ctx: &egui::Context,
        editor_model: &mut EditorModel,
        command_history: &mut CommandHistory,
        renderer: &mut Renderer,
    ) {
        // Get keyboard events and modifiers
        let modifiers = ctx.input(|i| i.modifiers);
        
        // Process key events
        let key_events: Vec<(egui::Key, bool)> = ctx.input(|i| {
            i.events.iter()
                .filter_map(|event| {
                    if let egui::Event::Key { key, pressed, .. } = event {
                        Some((*key, *pressed))
                    } else {
                        None
                    }
                })
                .collect()
        });
        
        // Send key events to the active tool
        for (key, pressed) in key_events {
            // Get a clone of the active tool to avoid borrow issues
            let mut tool = editor_model.active_tool().clone();
            let cmd = tool.on_key_event(
                key,
                pressed,
                &modifiers,
                editor_model,
            );
            
            // Update the tool in the model
            editor_model.update_tool(|_| tool);
            
            if let Some(cmd) = cmd {
                info!("Tool generated command from key event: {:?}", cmd);
                self.execute_command(cmd, command_history, editor_model, renderer);
                return; // Stop after executing a command
            }
        }
    }
    
    /// Execute a command and reset tool state
    fn execute_command(
        &mut self,
        cmd: Command,
        command_history: &mut CommandHistory,
        editor_model: &mut EditorModel,
        renderer: &mut Renderer,
    ) {
        // Execute the command
        let _ = command_history
            .execute(cmd.clone(), editor_model)
            .map_err(|err| log::warn!("Command execution failed: {}", err));
        
        // Reset the tool's interaction state
        let mut tool = editor_model.active_tool().clone();
        tool.reset_interaction_state();
        editor_model.update_tool(|_| tool);
        
        // Clear all previews in the renderer
        renderer.clear_all_previews();
        
        // Request a repaint
        self.request_repaint = true;
    }
}

/// Create and show the central editing panel
pub fn central_panel(
    editor_model: &mut EditorModel,
    command_history: &mut CommandHistory,
    renderer: &mut Renderer,
    ctx: &egui::Context,
) -> egui::Rect {
    let panel_response = egui::CentralPanel::default().show(ctx, |ui| {
        // Get the panel rect for hit testing
        let panel_rect = ui.max_rect();
        
        // Create or reuse a CentralPanel instance to handle input
        let mut central_panel = CentralPanel::new();
        
        // Render the document with the renderer
        renderer.render(ui, editor_model, panel_rect);
        
        // Get current pointer position if it's in the panel
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if panel_rect.contains(pos) {
                // Handle pointer events
                central_panel.handle_pointer_events(
                    ctx,
                    pos,
                    editor_model,
                    command_history,
                    renderer,
                    ui,
                );
            }
        }
        
        // Handle keyboard events regardless of pointer position
        central_panel.handle_keyboard_events(
            ctx,
            editor_model,
            command_history,
            renderer,
        );
        
        // Request repaint if needed
        if central_panel.request_repaint {
            ctx.request_repaint();
        }
        
        // Return the panel rect
        panel_rect
    });

    // Also request repaint if we're interacting with the panel
    if panel_response.response.hovered() || panel_response.response.dragged() {
        ctx.request_repaint();
    }

    panel_response.response.rect
}
