use crate::PaintApp;
use egui;
use crate::command::Command;

pub fn tools_window(app: &mut PaintApp, ctx: &egui::Context) {
    egui::Window::new("Tools")
        .fixed_pos(egui::pos2(20.0, 20.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let can_undo = app.command_history().can_undo();
                let can_redo = app.command_history().can_redo();
                
                if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
                    
                }
                if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
                    
                }
            });

            ui.separator();
            
            // Add stack sizes debug info
            let history = app.command_history();
            ui.horizontal(|ui| {
                ui.label(format!("Undo stack size: {}", history.undo_stack().len()));
                ui.label(format!("Redo stack size: {}", history.redo_stack().len()));
            });
            
            // Add command history table
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
                    
                    // Get the max length of both stacks
                    let max_len = undo_stack.len().max(redo_stack.len());
                    
                    for i in 0..max_len {
                        // Show undo stack entry
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
        });
} 