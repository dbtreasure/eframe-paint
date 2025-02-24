use crate::PaintApp;
use crate::command::Command;

pub fn central_panel(app: &mut PaintApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let canvas_rect = ui.available_rect_before_wrap();
        
        // Handle undo/redo panel
        egui::Window::new("Tools")
            .fixed_pos(egui::pos2(20.0, 20.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let can_undo = app.command_history().can_undo();
                    let can_redo = app.command_history().can_redo();
                    
                    if ui.button("Undo").clicked() && can_undo {
                        if let Some(Command::AddStroke(_)) = app.command_history_mut().undo() {
                            app.document_mut().remove_last_stroke();
                        }
                    }
                    
                    if ui.button("Redo").clicked() && can_redo {
                        if let Some(Command::AddStroke(stroke)) = app.command_history_mut().redo() {
                            app.document_mut().add_stroke(stroke);
                        }
                    }
                });
            });

        // Handle input
        app.handle_input(ctx, canvas_rect);

        // Render the canvas
        let painter = ui.painter();
        let renderer = app.renderer();
        renderer.render(ctx, painter, canvas_rect, app.document());
    });
}