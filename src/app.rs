use crate::renderer::Renderer;
use crate::document::Document;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PaintApp {
    // Skip serializing the renderer since it contains GPU resources
    #[serde(skip)]
    renderer: Option<Renderer>,
    document: Document,
    // Add state for tracking if modal is open
    #[serde(skip)]
    show_modal: bool,
    
}

impl Default for PaintApp {
    fn default() -> Self {
        Self {
            renderer: None,
            show_modal: false,
            document: Document::default(),
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
            show_modal: false,
            document: Document::default(),
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

        // Draw UI elements first (background)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Paint App");
            
            // Create a painting area that fills the remaining space
            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(
                available_size,
                egui::Sense::drag()
            );
            
            // Get the rect where we'll render
            let rect = response.rect;
            
            // Execute render pass on top of the panel
            if let Some(renderer) = &mut self.renderer {
                renderer.render(ctx, &painter, rect);
            }

            // Add a button that floats on top
            ui.put(
                egui::Rect::from_center_size(rect.center(), egui::vec2(100.0, 30.0)),
                egui::Button::new("Open Modal")
            ).clicked().then(|| self.show_modal = true);
        });

        // Show modal window if show_modal is true
        if self.show_modal {
            egui::Window::new("Example Modal")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("This is a modal window!");
                    if ui.button("Close").clicked() {
                        self.show_modal = false;
                    }
                });
        }
    }
}