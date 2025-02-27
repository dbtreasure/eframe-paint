use crate::PaintApp;
use egui;
use crate::command::Command;

pub fn tools_panel(app: &mut PaintApp, ctx: &egui::Context) {
    egui::SidePanel::left("tools_panel")
        .resizable(true)
        .default_width(200.0)
        .show(ctx, |ui| {
            app.set_tools_panel_rect(ui.max_rect());
            
            ui.heading("Tools");
            
            // Tool selection section
            ui.group(|ui| {
                ui.label("Drawing Tools");
                
                // Collect tool names first to avoid borrowing issues
                let tool_names: Vec<&str> = app.available_tools()
                    .iter()
                    .map(|tool| tool.name())
                    .collect();
                
                // Create buttons for each tool
                for &tool_name in &tool_names {
                    if ui.button(tool_name).clicked() {
                        app.set_active_tool_by_name(tool_name);
                    }
                }
            });
            
            ui.separator();
            
            // Undo/Redo section
            ui.horizontal(|ui| {
                let can_undo = app.command_history().can_undo();
                let can_redo = app.command_history().can_redo();
                
                if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
                    app.undo();
                }
                if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
                    app.redo();
                }
            });

            ui.separator();
            
            let history = app.command_history();
            ui.horizontal(|ui| {
                ui.label(format!("Undo stack size: {}", history.undo_stack().len()));
                ui.label(format!("Redo stack size: {}", history.redo_stack().len()));
            });
            
            egui::Grid::new("command_history_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Undo Stack");
                    ui.strong("Redo Stack");
                    ui.end_row();

                    let undo_stack = history.undo_stack();
                    let redo_stack = history.redo_stack();
                    
                    let max_len = undo_stack.len().max(redo_stack.len());
                    
                    for i in 0..max_len {
                        if i < undo_stack.len() {
                            match &undo_stack[i] {
                                Command::AddStroke(_) => { 
                                    ui.label(format!("Add Stroke {}", i)); 
                                }
                            }
                        } else {
                            ui.label("");
                        }
                        
                        // Show redo stack entry
                        if i < redo_stack.len() {
                            match &redo_stack[i] {
                                Command::AddStroke(_) => { 
                                    ui.label(format!("Add Stroke {}", i)); 
                                }
                            }
                        } else {
                            ui.label("");
                        }
                        
                        ui.end_row();
                    }
                });
                
            // If there's an active tool, show its UI
            if app.active_tool().is_some() {
                ui.separator();
                ui.heading("Tool Options");
                
                // Use the new method that handles both the tool and document access
                if let Some(cmd) = app.handle_tool_ui(ui) {
                    app.execute_command(cmd);
                }
            }
        });
} 