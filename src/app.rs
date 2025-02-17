use crate::renderer::Renderer;
use crate::document::{Document, Stroke};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PaintApp {
    // Skip serializing the renderer since it contains GPU resources
    #[serde(skip)]
    renderer: Option<Renderer>,
    document: Document,
    #[serde(skip)]
    current_stroke: Stroke,
}

impl Default for PaintApp {
    fn default() -> Self {
        Self {
            renderer: None,
            document: Document::default(),
            current_stroke: Stroke::default(),
        }
    }
}

impl PaintApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize renderer - no need for pollster now
        let renderer = Renderer::new(cc);
        
        Self {
            renderer: Some(renderer),
            document: Document::default(),
            current_stroke: Stroke::default(),
        }
    }
}

impl eframe::App for PaintApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Add debug window to show document state
        egui::Window::new("Document Debug")
            .show(ctx, |ui| {
                ui.label(format!("Number of layers: {}", self.document.layers.len()));
                if let Some(active_layer) = self.document.active_layer() {
                    ui.label(format!("Active layer: {}", active_layer.name));
                }
                if ui.button("Add Layer").clicked() {
                    self.document.add_layer(&format!("Layer {}", self.document.layers.len()));
                }
            });

        // In your update method in src/app.rs:
        egui::CentralPanel::default().show(ctx, |ui| {
            // Define the drawing canvas area.
            let available_size = ui.available_size();
            let (_id, canvas_rect) = ui.allocate_space(available_size);
            let response = ui.interact(canvas_rect, ui.make_persistent_id("drawing_canvas"), egui::Sense::click_and_drag());

            // Get the painter for the canvas.
            let painter = ui.painter_at(canvas_rect);

            // Render all committed strokes from the active layer.
            if let Some(active_idx) = self.document.active_layer {
                let layer = &self.document.layers[active_idx];
                for stroke in &layer.strokes {
                    painter.add(egui::Shape::line(
                        stroke.points.iter().map(|&(x, y)| egui::pos2(x, y)).collect(),
                        egui::Stroke::new(stroke.thickness, stroke.color),
                    ));
                }
            }

            // Handle pointer events for drawing a new stroke.
            if response.drag_started() {
                self.current_stroke.points.clear();
                let pos = response.interact_pointer_pos().unwrap_or_default();
                self.current_stroke.points.push((pos.x, pos.y));
            }

            if response.dragged() {
                // Append current pointer position to the current stroke.
                if let Some(pos) = response.hover_pos() {
                    let (x, y) = (pos.x, pos.y);
                    self.current_stroke.points.push((x, y));
                }
            }

            if response.drag_stopped() {
                // On release, commit the stroke to the document.
                if let Some(active_idx) = self.document.active_layer {
                    self.document.layers[active_idx].strokes.push(self.current_stroke.clone());
                }
                self.current_stroke.points.clear();
            }

            // Optionally, render the current stroke (in-progress) on top of everything.
            if !self.current_stroke.points.is_empty() {
                painter.add(egui::Shape::line(
                    self.current_stroke.points.iter().map(|&(x, y)| egui::pos2(x, y)).collect(),
                    egui::Stroke::new(self.current_stroke.thickness, self.current_stroke.color),
                ));
            }
        });

        
    }
}