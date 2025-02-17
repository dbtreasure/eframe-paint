use crate::renderer::Renderer;
use crate::document::Document;
use crate::Stroke;
use eframe::egui;
use crate::renderer::Tool;
use eframe::egui::Color32;
use crate::command::Command;
use std::mem;
use egui::DroppedFile;
use crate::layer::LayerContent;
use uuid;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PaintApp {
    // Skip serializing the renderer since it contains GPU resources
    #[serde(skip)]
    renderer: Option<Renderer>,
    document: Document,
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
        let renderer = Renderer::new(cc);
        
        Self {
            renderer: Some(renderer),
            document: Document::default(),
            current_stroke: Stroke::default(),
        }
    }

    fn commit_current_stroke(&mut self) {
        let stroke = mem::take(&mut self.current_stroke);
        if let Some(active_layer) = self.document.active_layer {
            let command = Command::AddStroke {
                layer_index: active_layer,
                stroke,
            };
            self.document.execute_command(command);
        }
    }

    fn handle_dropped_file(&mut self, file: DroppedFile) {
        let img_result = if let Some(bytes) = file.bytes {
            image::load_from_memory(&bytes)
        } else if let Some(path) = file.path {
            image::open(&path)
        } else {
            return;
        };

        match img_result {
            Ok(img) => {
                // Resize image if it's too large
                let img = if img.width() > 2048 || img.height() > 2048 {
                    let scale = 2048.0 / img.width().max(img.height()) as f32;
                    let new_width = (img.width() as f32 * scale) as u32;
                    let new_height = (img.height() as f32 * scale) as u32;
                    img.resize(new_width, new_height, image::imageops::FilterType::Triangle)
                } else {
                    img
                };

                // Convert to RGBA
                let img = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.into_raw();
                
                // Convert to egui color format
                let color_pixels: Vec<egui::Color32> = pixels
                    .chunks_exact(4)
                    .map(|chunk| egui::Color32::from_rgba_unmultiplied(
                        chunk[0], chunk[1], chunk[2], chunk[3]
                    ))
                    .collect();
                
                let color_image = egui::ColorImage { size, pixels: color_pixels };
                
                // Create texture from the image
                if let Some(renderer) = &mut self.renderer {
                    let texture_name = format!("image_texture_{}", uuid::Uuid::new_v4());
                    let texture = renderer.create_texture(color_image, &texture_name);
                    let layer_name = file.name;
                    self.document.add_image_layer(&layer_name, texture);
                }
            }
            Err(e) => {
                eprintln!("Failed to load image: {:?}", e);
            }
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
        // Check for dropped files
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            // Get the dropped files
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            
            for file in dropped_files {
                // Handle each dropped file
                self.handle_dropped_file(file);
            }
        }

        // Add left panel for tools
        egui::SidePanel::left("tools_panel").show(ctx, |ui| {
            if let Some(renderer) = &mut self.renderer {
                renderer.render_tools_panel(ui);
            }
            
            ui.separator();
            
            // Add undo/redo buttons
            ui.horizontal(|ui| {
                if ui.button("⟲ Undo").clicked() {
                    self.document.undo();
                }
                if ui.button("⟳ Redo").clicked() {
                    self.document.redo();
                }
            });
        });

        // After the left tools panel and before the central panel
        egui::SidePanel::right("layers_panel").show(ctx, |ui| {
            ui.heading("Layers");
            ui.separator();

            let mut visibility_change = None;
            let mut active_change = None;

            // List layers in reverse order (top layer first)
            for (idx, layer) in self.document.layers.iter().enumerate().rev() {
                ui.horizontal(|ui| {
                    let is_active = Some(idx) == self.document.active_layer;
                    
                    let layer_icon = match &layer.content {
                        LayerContent::Strokes(_) => "✏️",
                        LayerContent::Image { .. } => "🖼️",
                    };
                    
                    if ui.selectable_label(is_active, format!("{} {}", layer_icon, layer.name)).clicked() {
                        active_change = Some(idx);
                    }
                    
                    if ui.button(if layer.visible { "👁" } else { "👁‍🗨" }).clicked() {
                        visibility_change = Some(idx);
                    }
                });
            }

            // Apply changes after the iteration
            if let Some(idx) = visibility_change {
                self.document.toggle_layer_visibility(idx);
            }
            if let Some(idx) = active_change {
                self.document.active_layer = Some(idx);
            }

            ui.separator();
            if ui.button("+ Add Layer").clicked() {
                self.document.add_layer(&format!("Layer {}", self.document.layers.len()));
            }
        });

        // In your update method in src/app.rs:
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let (_id, canvas_rect) = ui.allocate_space(available_size);

            let painter = ui.painter_at(canvas_rect);

            // Render all layers
            for layer in &self.document.layers {
                if !layer.visible {
                    continue;
                }

                match &layer.content {
                    LayerContent::Strokes(strokes) => {
                        for stroke in strokes {
                            painter.add(egui::Shape::line(
                                stroke.points.iter().map(|&(x, y)| egui::pos2(x, y)).collect(),
                                egui::Stroke::new(stroke.thickness, stroke.color),
                            ));
                        }
                    }
                    LayerContent::Image { texture: Some(texture), size } => {
                        let image_rect = egui::Rect::from_min_size(
                            canvas_rect.min,
                            egui::vec2(size[0] as f32, size[1] as f32),
                        );
                        painter.image(texture.id(), image_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                    }
                    LayerContent::Image { texture: None, .. } => {}
                }
            }

            // Handle pointer events for drawing a new stroke.
            let response = ui.interact(canvas_rect, ui.make_persistent_id("drawing_canvas"), egui::Sense::click_and_drag());

            if response.drag_started() {
                self.current_stroke.points.clear();
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(renderer) = &self.renderer {
                        // Set stroke properties based on current tool
                        match renderer.current_tool() {
                            Tool::Brush => {
                                self.current_stroke.color = renderer.brush_color();
                                self.current_stroke.thickness = renderer.brush_thickness();
                            }
                            Tool::Eraser => {
                                self.current_stroke.color = Color32::WHITE; // Or your background color
                                self.current_stroke.thickness = renderer.brush_thickness();
                            }
                            Tool::Selection => {
                                // For selection tool, maybe just store the points but don't draw
                                self.current_stroke.color = Color32::TRANSPARENT;
                                self.current_stroke.thickness = 1.0;
                            }
                        }
                    }
                    self.current_stroke.points.push((pos.x, pos.y));
                }
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
                self.commit_current_stroke();
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