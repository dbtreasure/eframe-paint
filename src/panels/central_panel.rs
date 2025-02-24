use crate::PaintApp;
use super::tools_panel;

pub fn central_panel(app: &mut PaintApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let canvas_rect = ui.available_rect_before_wrap();
        
        // Handle input before UI elements
        app.handle_input(ctx, canvas_rect);
        
        // Show the tools window
        tools_panel::tools_window(app, ctx);
        
        let painter = ui.painter();
        let renderer = app.renderer();
        renderer.render(ctx, painter, canvas_rect, app.document());
    });
}