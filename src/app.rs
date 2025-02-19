use crate::renderer::Renderer;
use crate::document::Document;
use crate::Stroke;
use eframe::egui;
use crate::renderer::Tool;
use eframe::egui::Color32;
use crate::command::Command;
use crate::gizmo::TransformGizmo;
use crate::layer::{Layer, LayerContent, Transform};
use std::mem;
use egui::DroppedFile;
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
    #[serde(skip)]
    transform_gizmo: Option<TransformGizmo>,
    #[serde(skip)]
    last_canvas_rect: Option<egui::Rect>,
    #[serde(skip)]
    editing_layer_name: Option<usize>,
    #[serde(skip)]
    dragged_layer: Option<usize>,
}

impl Default for PaintApp {
    fn default() -> Self {
        Self {
            renderer: None,
            document: Document::default(),
            current_stroke: Stroke::default(),
            transform_gizmo: None,
            last_canvas_rect: None,
            editing_layer_name: None,
            dragged_layer: None,
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
            transform_gizmo: None,
            last_canvas_rect: None,
            editing_layer_name: None,
            dragged_layer: None,
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

                    // Get the current canvas size
                    if let Some(screen_rect) = self.last_canvas_rect {
                        // Calculate the center position for the image
                        let image_width = size[0] as f32;
                        let image_height = size[1] as f32;
                        
                        // Calculate center position relative to canvas
                        let center_x = (screen_rect.width() - image_width) / 2.0;
                        let center_y = (screen_rect.height() - image_height) / 2.0;

                        // Create initial transform for centered position
                        let initial_transform = Transform {
                            position: egui::Vec2::new(center_x, center_y),
                            scale: egui::Vec2::splat(1.0),
                            rotation: 0.0,
                        };

                        // Create and add the centered image layer
                        let command = Command::AddImageLayer {
                            name: layer_name,
                            texture: Some(texture),
                            size,
                            initial_transform,
                        };
                        self.document.execute_command(command);

                        // Update transform gizmo with the correct initial bounds
                        if let Some(active_idx) = self.document.active_layer {
                            if let Some(layer) = self.document.layers.get(active_idx) {
                                if let Some(transformed_bounds) = self.calculate_transformed_bounds(&layer.content, &layer.transform) {
                                    self.transform_gizmo = Some(TransformGizmo::new(transformed_bounds));
                                }
                            }
                        }
                    } else {
                        // Fallback to regular add_image_layer if we don't have canvas dimensions yet
                        self.document.add_image_layer(&layer_name, texture);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to load image: {:?}", e);
            }
        }
    }

    fn calculate_layer_bounds(&self, content: &LayerContent) -> Option<egui::Rect> {
        match content {
            LayerContent::Strokes(strokes) => {
                if strokes.is_empty() {
                    return None;
                }
                
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                
                for stroke in strokes {
                    for &(x, y) in &stroke.points {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
                
                Some(egui::Rect::from_min_max(
                    egui::pos2(min_x, min_y),
                    egui::pos2(max_x, max_y),
                ))
            }
            LayerContent::Image { size, .. } => {
                // Use a fixed starting position for images (0,0)
                let width = size[0] as f32;
                let height = size[1] as f32;
                Some(egui::Rect::from_min_max(
                    egui::pos2(0.0, 0.0),
                    egui::pos2(width, height),
                ))
            }
        }
    }

    fn render_layer(&self, painter: &egui::Painter, canvas_rect: egui::Rect, layer: &crate::Layer) {
        match &layer.content {
            LayerContent::Strokes(strokes) => {
                // Calculate the center of the original bounds for pivot
                let original_bounds = self.calculate_layer_bounds(&layer.content);
                let pivot = original_bounds.map(|b| b.center()).unwrap_or(egui::pos2(0.0, 0.0));
                
                // Draw debug visualization for the transform
                if let Some(bounds) = original_bounds {
                    self.draw_transform_debug(painter, bounds, layer.transform, canvas_rect);
                }
                
                for stroke in strokes {
                    let matrix = layer.transform.to_matrix_with_pivot(pivot.to_vec2());
                    let transformed_points: Vec<egui::Pos2> = stroke.points.iter().map(|&(x, y)| {
                        let x_transformed = matrix[0][0] * x + matrix[0][1] * y + matrix[0][2];
                        let y_transformed = matrix[1][0] * x + matrix[1][1] * y + matrix[1][2];
                        egui::pos2(x_transformed, y_transformed)
                    }).collect();
                    
                    // Offset the transformed points by the canvas position
                    let final_points: Vec<egui::Pos2> = transformed_points.iter()
                        .map(|p| *p + canvas_rect.min.to_vec2())
                        .collect();
                    
                    painter.add(egui::Shape::line(
                        final_points,
                        egui::Stroke::new(stroke.thickness, stroke.color),
                    ));
                }
            }
            LayerContent::Image { texture: Some(texture), size } => {
                // Create the original rect starting from (0,0)
                let original_rect = egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(size[0] as f32, size[1] as f32),
                );
                
                let pivot = original_rect.center();
                let matrix = layer.transform.to_matrix_with_pivot(pivot.to_vec2());
                
                // Create UV rect for texture coordinates
                let uv_rect = egui::Rect::from_min_max(
                    egui::pos2(0.0, 0.0),
                    egui::pos2(1.0, 1.0),
                );
                
                // Create the transformed mesh
                let mut mesh = self.create_transformed_image_mesh(
                    original_rect,
                    uv_rect,
                    matrix,
                    Color32::WHITE,
                );
                
                // Set the texture ID for the mesh
                if let egui::Shape::Mesh(mesh) = &mut mesh {
                    mesh.texture_id = texture.id();
                }
                
                // Translate the mesh to canvas position
                if let egui::Shape::Mesh(mesh) = &mut mesh {
                    for vertex in &mut mesh.vertices {
                        vertex.pos += canvas_rect.min.to_vec2();
                    }
                }
                
                // Add the mesh to the painter
                painter.add(mesh);
            }
            LayerContent::Image { texture: None, .. } => {}
        }
    }

    fn draw_transform_debug(&self, painter: &egui::Painter, bounds: egui::Rect, transform: Transform, canvas_rect: egui::Rect) {
        let pivot = bounds.center();
        let matrix = transform.to_matrix_with_pivot(pivot.to_vec2());
        
        // Draw coordinate axes
        let axis_length = 50.0;
        let origin = pivot + canvas_rect.min.to_vec2();
        
        // Draw X axis (red)
        let x_end = egui::pos2(
            matrix[0][0] * axis_length + origin.x,
            matrix[1][0] * axis_length + origin.y,
        );
        painter.line_segment(
            [origin, x_end],
            egui::Stroke::new(2.0, egui::Color32::RED),
        );
        
        // Draw Y axis (green)
        let y_end = egui::pos2(
            matrix[0][1] * axis_length + origin.x,
            matrix[1][1] * axis_length + origin.y,
        );
        painter.line_segment(
            [origin, y_end],
            egui::Stroke::new(2.0, egui::Color32::GREEN),
        );
        
        // Draw pivot point
        painter.circle_filled(
            origin,
            4.0,
            egui::Color32::YELLOW,
        );
        
        // Draw rotation angle indicator
        let angle_radius = 30.0;
        let angle_points: Vec<egui::Pos2> = (0..=20).map(|i| {
            let t = i as f32 / 20.0;
            let angle = t * transform.rotation;
            egui::pos2(
                origin.x + angle_radius * angle.cos(),
                origin.y - angle_radius * angle.sin()
            )
        }).collect();
        
        painter.add(egui::Shape::line(
            angle_points,
            egui::Stroke::new(1.0, egui::Color32::YELLOW),
        ));
    }

    fn handle_transform_change(&mut self, layer_idx: usize, old_transform: crate::layer::Transform, new_transform: crate::layer::Transform) {
        let command = Command::TransformLayer {
            layer_index: layer_idx,
            old_transform,
            new_transform,
        };
        self.document.execute_command(command);
    }

    fn calculate_transformed_bounds(&self, content: &LayerContent, transform: &crate::layer::Transform) -> Option<egui::Rect> {
        let original_bounds = self.calculate_layer_bounds(content)?;
        let pivot = original_bounds.center();
        let matrix = transform.to_matrix_with_pivot(pivot.to_vec2());
        
        // Transform all corners of the original bounds
        let corners = [
            original_bounds.left_top(),
            original_bounds.right_top(),
            original_bounds.right_bottom(),
            original_bounds.left_bottom(),
        ];
        
        let transformed_corners: Vec<egui::Pos2> = corners.iter().map(|&pos| {
            let x_transformed = matrix[0][0] * pos.x + matrix[0][1] * pos.y + matrix[0][2];
            let y_transformed = matrix[1][0] * pos.x + matrix[1][1] * pos.y + matrix[1][2];
            egui::pos2(x_transformed, y_transformed)
        }).collect();
        
        // Include the canvas offset in the transformed bounds
        if let Some(canvas_rect) = self.last_canvas_rect {
            Some(egui::Rect::from_points(&transformed_corners).translate(canvas_rect.min.to_vec2()))
        } else {
            Some(egui::Rect::from_points(&transformed_corners))
        }
    }

    fn create_transformed_image_mesh(
        &self,
        rect: egui::Rect,
        uv_rect: egui::Rect,
        matrix: [[f32; 3]; 3],
        color: Color32,
    ) -> egui::Shape {
        // Create two triangles for the quad
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
        
        let uvs = [
            uv_rect.left_top(),
            uv_rect.right_top(),
            uv_rect.right_bottom(),
            uv_rect.left_bottom(),
        ];
        
        // Transform the corners using our matrix
        let transformed_corners: Vec<egui::Pos2> = corners.iter().map(|&pos| {
            let x_transformed = matrix[0][0] * pos.x + matrix[0][1] * pos.y + matrix[0][2];
            let y_transformed = matrix[1][0] * pos.x + matrix[1][1] * pos.y + matrix[1][2];
            egui::pos2(x_transformed, y_transformed)
        }).collect();
        
        // Create the mesh with two triangles
        let indices = vec![0, 1, 2, 0, 2, 3];
        let vertices: Vec<egui::epaint::Vertex> = transformed_corners
            .iter()
            .zip(uvs.iter())
            .map(|(&pos, &uv)| egui::epaint::Vertex {
                pos,
                uv,
                color,
            })
            .collect();
        
        egui::Shape::mesh(egui::epaint::Mesh {
            indices,
            vertices,
            texture_id: egui::TextureId::default(), // Will be set later
        })
    }

    fn handle_layer_reorder(&mut self, from_index: usize, to_index: usize) {
        let command = Command::ReorderLayer {
            from_index,
            to_index,
        };
        self.document.execute_command(command);
    }

    fn handle_layer_rename(&mut self, layer_index: usize, old_name: String, new_name: String) {
        let command = Command::RenameLayer {
            layer_index,
            old_name,
            new_name,
        };
        self.document.execute_command(command);
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
            
            if ui.button("+ Add Layer").clicked() {
                self.document.add_layer(&format!("Layer {}", self.document.layers.len()));
            }
            
            ui.separator();

            let mut visibility_change = None;
            let mut active_change = None;
            let mut layer_rename = None;
            let text_height = ui.text_style_height(&egui::TextStyle::Body);
            let layer_height = text_height * 1.5;

            // Get the response area for the layer list
            let layer_list_height = layer_height * self.document.layers.len() as f32;
            let (layer_list_rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), layer_list_height),
                egui::Sense::hover(),
            );

            // List layers in order (top layer first)
            for (idx, layer) in self.document.layers.iter().enumerate() {
                let layer_rect = egui::Rect::from_min_size(
                    egui::pos2(layer_list_rect.min.x, layer_list_rect.min.y + idx as f32 * layer_height),
                    egui::vec2(layer_list_rect.width(), layer_height),
                );
                
                let layer_response = ui.allocate_rect(layer_rect, egui::Sense::click_and_drag());
                let is_being_dragged = layer_response.dragged();
                let is_active = Some(idx) == self.document.active_layer;
                
                // Handle dragging
                if is_being_dragged && self.dragged_layer.is_none() {
                    self.dragged_layer = Some(idx);
                }
                
                // Draw drag indicator if this layer is being dragged
                if Some(idx) == self.dragged_layer {
                    ui.painter().rect_filled(
                        layer_rect,
                        0.0,
                        if is_active {
                            egui::Color32::from_rgba_premultiplied(100, 100, 255, 100)
                        } else {
                            egui::Color32::from_rgba_premultiplied(100, 100, 100, 100)
                        },
                    );
                }

                // Layer content
                ui.allocate_ui_at_rect(layer_rect, |ui| {
                    ui.horizontal(|ui| {
                        let layer_icon = match &layer.content {
                            LayerContent::Strokes(_) => "✏️",
                            LayerContent::Image { .. } => "🖼️",
                        };

                        // Visibility toggle
                        if ui.button(if layer.visible { "👁" } else { "👁‍🗨" }).clicked() {
                            visibility_change = Some(idx);
                        }

                        // Layer name (editable or static)
                        if Some(idx) == self.editing_layer_name {
                            let mut name = layer.name.clone();
                            let response = ui.text_edit_singleline(&mut name);
                            if response.lost_focus() {
                                if !name.is_empty() && name != layer.name {
                                    layer_rename = Some((idx, layer.name.clone(), name));
                                }
                                self.editing_layer_name = None;
                            }
                        } else {
                            let label = format!("{} {}", layer_icon, layer.name);
                            let response = ui.selectable_label(is_active, label);
                            if response.clicked() {
                                active_change = Some(idx);
                            }
                            if response.double_clicked() {
                                self.editing_layer_name = Some(idx);
                            }
                        }
                    });
                });
            }

            // Handle drag and drop
            if let Some(dragged_idx) = self.dragged_layer {
                if !ui.input(|i| i.pointer.any_down()) {
                    // Find the target position based on the cursor position
                    if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        let target_idx = ((pointer_pos.y - layer_list_rect.min.y) / layer_height)
                            .floor()
                            .clamp(0.0, (self.document.layers.len() - 1) as f32) as usize;
                        
                        if target_idx != dragged_idx {
                            self.handle_layer_reorder(dragged_idx, target_idx);
                        }
                    }
                    self.dragged_layer = None;
                }
            }

            // Apply changes after the iteration
            if let Some(idx) = visibility_change {
                self.document.toggle_layer_visibility(idx);
            }
            if let Some(idx) = active_change {
                self.document.active_layer = Some(idx);
            }
            if let Some((idx, old_name, new_name)) = layer_rename {
                self.handle_layer_rename(idx, old_name, new_name);
            }
        });

        // In your update method in src/app.rs:
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let (_id, canvas_rect) = ui.allocate_space(available_size);
            
            // Store the canvas rect for use when adding new images
            self.last_canvas_rect = Some(canvas_rect);

            let painter = ui.painter_at(canvas_rect);

            // Render layers from bottom to top
            for layer in self.document.layers.iter().rev() {
                if layer.visible {
                    self.render_layer(&painter, canvas_rect, layer);
                }
            }

            // Create a separate response for the canvas drawing
            let canvas_response = ui.interact(
                canvas_rect,
                ui.make_persistent_id("drawing_canvas"),
                egui::Sense::drag(),
            );

            // Handle transform gizmo for active layer with higher priority
            let mut gizmo_interacted = false;
            if let Some(active_idx) = self.document.active_layer {
                if let Some(layer) = self.document.layers.get(active_idx) {
                    // Calculate transformed bounds for the gizmo
                    if let Some(transformed_bounds) = self.calculate_transformed_bounds(&layer.content, &layer.transform) {
                        let gizmo = self.transform_gizmo.get_or_insert_with(|| TransformGizmo::new(transformed_bounds));
                        
                        // Always update the gizmo bounds to match the current transform
                        gizmo.update_bounds(transformed_bounds);
                        
                        let mut transform = layer.transform;
                        let old_transform = transform;

                        if gizmo.update(ui, &mut transform) {
                            // Apply the transform change
                            if let Some(layer) = self.document.layers.get_mut(active_idx) {
                                layer.transform = transform;
                                self.handle_transform_change(active_idx, old_transform, transform);
                            }
                            gizmo_interacted = true;
                        }
                    }
                }
            }

            // Only handle drawing if the gizmo wasn't interacted with
            if !gizmo_interacted {
                // Handle drawing input
                if canvas_response.drag_started() {
                    self.current_stroke.points.clear();
                    if let Some(pos) = canvas_response.interact_pointer_pos() {
                        if let Some(renderer) = &self.renderer {
                            match renderer.current_tool() {
                                Tool::Brush => {
                                    self.current_stroke.color = renderer.brush_color();
                                    self.current_stroke.thickness = renderer.brush_thickness();
                                }
                                Tool::Eraser => {
                                    self.current_stroke.color = Color32::WHITE;
                                    self.current_stroke.thickness = renderer.brush_thickness();
                                }
                                Tool::Selection => {
                                    self.current_stroke.color = Color32::TRANSPARENT;
                                    self.current_stroke.thickness = 1.0;
                                }
                            }
                        }
                        self.current_stroke.points.push((pos.x, pos.y));
                    }
                }

                if canvas_response.dragged() {
                    if let Some(pos) = canvas_response.hover_pos() {
                        self.current_stroke.points.push((pos.x, pos.y));
                    }
                }

                if canvas_response.drag_released() {
                    self.commit_current_stroke();
                }

                // Draw current stroke
                if !self.current_stroke.points.is_empty() {
                    painter.add(egui::Shape::line(
                        self.current_stroke.points.iter()
                            .map(|&(x, y)| egui::pos2(x, y))
                            .collect(),
                        egui::Stroke::new(
                            self.current_stroke.thickness,
                            self.current_stroke.color,
                        ),
                    ));
                }
            }
        });
    }
}