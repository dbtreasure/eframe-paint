use crate::renderer::{Renderer, Tool};
use crate::document::Document;
use crate::Stroke;
use eframe::egui;
use eframe::egui::Color32;
use crate::command::{Command, commands::ToolPropertyValue};
use crate::gizmo::TransformGizmo;
use crate::layer::{LayerContent, Transform};
use std::mem;
use egui::DroppedFile;
use uuid;
use futures;
use crate::layer::LayerId;
use crate::input::{InputState, InputRouter};
use crate::state::{EditorState, EditorContext, SelectionInProgress};
use crate::tool::ToolType;
use crate::selection::{SelectionMode, SelectionShape, Selection};
use crate::state::context::FeedbackLevel;
use crate::tool::types::DrawingTool;
use crate::log;

/// Logger that deduplicates messages and tracks canvas state
#[derive(Default, Debug)]
struct CanvasLogger {
    last_message: Option<String>,
    last_state: Option<CanvasState>,
}

#[derive(Debug, Eq, PartialEq)]
struct CanvasState {
    clicked: bool,
    released: bool,
    dragged: bool,
    has_hover: bool,
    has_interact: bool,
}

impl CanvasLogger {
    fn log_input(&mut self, response: &egui::Response) {
        let current_state = CanvasState {
            clicked: response.clicked(),
            released: response.drag_released(),
            dragged: response.dragged(),
            has_hover: response.hover_pos().is_some(),
            has_interact: response.interact_pointer_pos().is_some(),
        };

        // Only log if state changed
        if self.last_state.as_ref() != Some(&current_state) {
            let msg = format!(
                "[Canvas] clicked={}, released={}, dragged={}, has_hover={}, has_interact={}", 
                current_state.clicked,
                current_state.released,
                current_state.dragged,
                current_state.has_hover,
                current_state.has_interact
            );
            log!("{}", msg);
            self.last_state = Some(current_state);
        }
    }

    fn log_brush(&mut self, msg: &str) {
        if self.last_message.as_deref() != Some(msg) {
            log!("[Brush] {}", msg);
            self.last_message = Some(msg.to_string());
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PaintApp {
    #[serde(skip)]
    logger: CanvasLogger,
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
    // Selection state
    #[serde(skip)]
    current_selection_start: Option<egui::Pos2>,
    #[serde(skip)]
    input_router: InputRouter,
}

impl Default for PaintApp {
    fn default() -> Self {
        Self {
            logger: CanvasLogger::default(),
            renderer: None,
            document: Document::default(),
            current_stroke: Stroke::default(),
            transform_gizmo: None,
            last_canvas_rect: None,
            editing_layer_name: None,
            dragged_layer: None,
            current_selection_start: None,
            input_router: InputRouter::new(),
        }
    }
}

impl PaintApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let renderer = Renderer::new(cc);
        let document = Document::default();
        
        Self {
            logger: CanvasLogger::default(),
            renderer: Some(renderer),
            document,
            current_stroke: Stroke::default(),
            transform_gizmo: None,
            last_canvas_rect: None,
            editing_layer_name: None,
            dragged_layer: None,
            current_selection_start: None,
            input_router: InputRouter::new(),
        }
    }

    fn commit_current_stroke(&mut self) {
        // Auto-create a default layer if none exist
        if self.document.layers.is_empty() {
            log!("[DEBUG] commit_current_stroke: No layers found, creating default layer.");
            let command = Command::AddLayer { name: "Default Layer".into(), texture: None };
            if let Err(e) = self.document.execute_command(command) {
                eprintln!("[DEBUG] Failed to create default layer: {:?}", e);
                return;
            }
        }

        // Check if an active layer is set; if not, log and return
        if self.document.active_layer.is_none() {
            log!("[DEBUG] commit_current_stroke: No active layer present even after creating default layer. Stroke not committed.");
            return;
        }
        
        // If the stroke has only one point, duplicate it with a slight offset
        if self.current_stroke.points.len() < 2 {
            if let Some(first_point) = self.current_stroke.points.first().cloned() {
                let offset_point = egui::pos2(first_point.x + 1.0, first_point.y + 1.0);
                self.current_stroke.add_point(offset_point);
                log!("[DEBUG] Stroke had <2 points; duplicated point: now {} points", self.current_stroke.points.len());
            }
        } else {
            log!("[DEBUG] Stroke has {} points", self.current_stroke.points.len());
        }
        
        // Filter out nearly duplicate points (only keep points that move at least 1.0 units)
        {
           let original_count = self.current_stroke.points.len();
           let mut filtered = Vec::new();
           for p in &self.current_stroke.points {
                if let Some(last) = filtered.last() {
                    let delta: egui::Vec2 = *p - *last;
                    if delta.length() >= 1.0 {
                        filtered.push(*p);
                    }
                } else {
                    filtered.push(*p);
                }
           }
           log!("[DEBUG] Filtered stroke points: {} -> {}", original_count, filtered.len());
           self.current_stroke.points = filtered;
        }

        let stroke = std::mem::take(&mut self.current_stroke);
        log!("[DEBUG] Committing stroke with points: {:?}", stroke.points);
        if let Some(active_layer) = self.document.active_layer {
            self.logger.log_brush(&format!("Committing stroke to layer {}", active_layer));
            let command = Command::AddStroke {
                layer_id: LayerId(active_layer),
                stroke,
            };
            println!("[DEBUG] Executing AddStroke command for layer id: {:?}", active_layer);
            if let Err(e) = self.document.execute_command(command) {
                eprintln!("[DEBUG] Failed to execute AddStroke command: {:?}", e);
            } else {
                println!("[DEBUG] Successfully executed AddStroke command for layer id: {:?}", active_layer);
            }
            if self.should_show_gizmo() {
                if let Some(layer) = self.document.layers.get(active_layer) {
                    if let Some(transformed_bounds) = self.calculate_transformed_bounds(&layer.content, &layer.transform) {
                        self.transform_gizmo = Some(TransformGizmo::new(transformed_bounds, Transform::default()));
                    }
                }
            }
        }
    }

    fn handle_dropped_file(&mut self, file: DroppedFile) {
        let process_image = async move |bytes: &[u8]| {
            image::load_from_memory(bytes)
        };

        let img_result = if let Some(bytes) = file.bytes {
            futures::executor::block_on(process_image(&bytes))
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
                    log!("[DEBUG] Created texture with id: {:?}", texture.id());
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
                        let _initial_transform = Transform {
                            position: egui::Vec2::new(center_x, center_y),
                            scale: egui::Vec2::splat(1.0),
                            rotation: 0.0,
                        };

                        // Create and add the centered image layer
                        let command = Command::AddLayer {
                            name: layer_name,
                            texture: Some((texture, size)),
                        };
                        if let Err(e) = self.document.execute_command(command) {
                            eprintln!("Failed to execute command: {:?}", e);
                        }

                        // Always show transform gizmo for new images
                        if let Some(active_idx) = self.document.active_layer {
                            if let Some(layer) = self.document.layers.get(active_idx) {
                                if let Some(transformed_bounds) = self.calculate_transformed_bounds(&layer.content, &layer.transform) {
                                    self.transform_gizmo = Some(TransformGizmo::new(transformed_bounds, Transform::default()));
                                }
                            }
                        }
                    } else {
                        // Fallback to regular add_image_layer if we don't have canvas dimensions yet
                        let command = Command::AddLayer {
                            name: layer_name,
                            texture: Some((texture, size)),
                        };
                        if let Err(e) = self.document.execute_command(command) {
                            eprintln!("Failed to execute command: {:?}", e);
                        }
                        // Clear any existing transform gizmo
                        self.clear_transform_gizmo();
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
                    for pos in &stroke.points {
                        min_x = min_x.min(pos.x);
                        min_y = min_y.min(pos.y);
                        max_x = max_x.max(pos.x);
                        max_y = max_y.max(pos.y);
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
                let original_bounds = self.calculate_layer_bounds(&layer.content);
                let pivot = original_bounds.map(|b| b.center()).unwrap_or(egui::pos2(0.0, 0.0));
                for stroke in strokes {
                    let matrix = layer.transform.to_matrix_with_pivot(pivot.to_vec2());
                    let transformed_points: Vec<egui::Pos2> = stroke.points.iter().map(|&pos| {
                        let x_transformed = matrix[0][0] * pos.x + matrix[0][1] * pos.y + matrix[0][2];
                        let y_transformed = matrix[1][0] * pos.x + matrix[1][1] * pos.y + matrix[1][2];
                        egui::pos2(x_transformed, y_transformed)
                    }).collect();
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

    fn handle_transform_change(&mut self, layer_idx: usize, _old_transform: crate::layer::Transform, new_transform: crate::layer::Transform) {
        let command = Command::TransformLayer {
            layer_id: LayerId(layer_idx),
            transform: new_transform,
        };
        if let Err(e) = self.document.execute_command(command) {
            eprintln!("Failed to execute command: {:?}", e);
        }
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
            layer_id: LayerId(from_index),
            new_index: to_index,
        };
        if let Err(e) = self.document.execute_command(command) {
            eprintln!("Failed to execute command: {:?}", e);
        }
    }

    fn handle_layer_rename(&mut self, layer_index: usize, old_name: String, new_name: String) {
        let command = Command::RenameLayer {
            layer_id: LayerId(layer_index),
            old_name,
            new_name,
        };
        if let Err(e) = self.document.execute_command(command) {
            eprintln!("Failed to execute command: {:?}", e);
        }
    }

    // Make generate_dashed_line static
    fn generate_dashed_line(start: egui::Pos2, end: egui::Pos2, dash_length: f32, gap_length: f32) -> Vec<[egui::Pos2; 2]> {
        let vec = end - start;
        let total_length = vec.length();
        let dir = vec.normalized();
        let segment_length = dash_length + gap_length;
        let num_segments = (total_length / segment_length).floor() as usize;
        
        let mut segments = Vec::new();
        for i in 0..num_segments {
            let t_start = i as f32 * segment_length;
            let t_end = t_start + dash_length;
            if t_end <= total_length {
                segments.push([
                    start + dir * t_start,
                    start + dir * t_end,
                ]);
            }
        }
        
        // Add final segment if there's room
        let remaining = total_length - (num_segments as f32 * segment_length);
        if remaining > 0.0 {
            let t_start = num_segments as f32 * segment_length;
            let t_end = t_start + remaining.min(dash_length);
            segments.push([
                start + dir * t_start,
                start + dir * t_end,
            ]);
        }
        
        segments
    }

    // Make generate_dashed_rect static too since it only uses generate_dashed_line
    fn generate_dashed_rect(rect: egui::Rect, dash_length: f32, gap_length: f32) -> Vec<[egui::Pos2; 2]> {
        let mut segments = Vec::new();
        
        // Top edge
        segments.extend(Self::generate_dashed_line(
            rect.left_top(),
            rect.right_top(),
            dash_length,
            gap_length,
        ));
        
        // Right edge
        segments.extend(Self::generate_dashed_line(
            rect.right_top(),
            rect.right_bottom(),
            dash_length,
            gap_length,
        ));
        
        // Bottom edge
        segments.extend(Self::generate_dashed_line(
            rect.right_bottom(),
            rect.left_bottom(),
            dash_length,
            gap_length,
        ));
        
        // Left edge
        segments.extend(Self::generate_dashed_line(
            rect.left_bottom(),
            rect.left_top(),
            dash_length,
            gap_length,
        ));
        
        segments
    }

    fn render_selection(&self, painter: &egui::Painter, canvas_rect: egui::Rect, selection: &crate::selection::Selection, is_dragging: bool) {
        let dash_length = 8.0;
        let gap_length = 4.0;

        match &selection.shape {
            crate::selection::SelectionShape::Rectangle(rect) => {
                // Transform the rect to screen space
                let screen_rect = rect.translate(canvas_rect.min.to_vec2());
                
                // Generate and draw dashed segments
                let segments = Self::generate_dashed_rect(screen_rect, dash_length, gap_length);
                for segment in segments {
                    painter.line_segment(
                        segment,
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );
                }
            }
            crate::selection::SelectionShape::Freeform(points) => {
                if points.len() >= 2 {
                    // Convert points to screen space
                    let screen_points: Vec<egui::Pos2> = points.iter()
                        .map(|p| *p + canvas_rect.min.to_vec2())
                        .collect();
                    
                    // Generate dashed segments between consecutive points
                    for points_pair in screen_points.windows(2) {
                        let segments = Self::generate_dashed_line(
                            points_pair[0],
                            points_pair[1],
                            dash_length,
                            gap_length,
                        );
                        
                        for segment in segments {
                            painter.line_segment(
                                segment,
                                egui::Stroke::new(1.0, egui::Color32::WHITE),
                            );
                        }
                    }
                    
                    // Close the path if it's a complete selection
                    if !is_dragging && screen_points.len() > 2 {
                        let segments = Self::generate_dashed_line(
                            *screen_points.last().unwrap(),
                            screen_points[0],
                            dash_length,
                            gap_length,
                        );
                        
                        for segment in segments {
                            painter.line_segment(
                                segment,
                                egui::Stroke::new(1.0, egui::Color32::WHITE),
                            );
                        }
                    }
                }
            }
        }
    }

    // Add a helper method to clear the transform gizmo
    fn clear_transform_gizmo(&mut self) {
        self.transform_gizmo = None;
    }

    fn should_show_gizmo(&self) -> bool {
        // Never show gizmo during active operations
        if self.current_stroke.points.len() > 0 || 
            self.current_selection_start.is_some() {
            return false;
        }

        // Only show gizmo when we have an active layer
        if let Some(active_layer) = self.document.active_layer {
            // And that layer has content
            if let Some(_layer) = self.document.layers.get(active_layer) {
                return true;
            }
        }
        false
    }

    fn handle_tool_change(&mut self, new_tool: Tool) {
        // Clear gizmo when switching to drawing/selection tools
        match new_tool {
            Tool::Brush | Tool::Eraser | Tool::Selection => self.clear_transform_gizmo(),
        }
    }

    fn handle_selection_input(&mut self, canvas_response: &egui::Response, canvas_rect: egui::Rect, _painter: &egui::Painter) {
        if let Some(renderer) = &mut self.renderer {
            let mut editor_ctx = EditorContext::new(self.document.clone(), renderer.clone());
            
            if canvas_response.drag_started() {
                if let Some(pos) = canvas_response.interact_pointer_pos() {
                    let doc_pos = pos - canvas_rect.min.to_vec2();
                    if renderer.current_tool() == Tool::Selection {
                        let selection_mode = renderer.selection_mode();
                        let start_pos = egui::pos2(doc_pos.x, doc_pos.y);
                        let command = Command::BeginOperation(EditorState::Selecting { 
                            mode: selection_mode,
                            in_progress: Some(SelectionInProgress {
                                start: start_pos,
                                current: start_pos,
                                mode: selection_mode,
                                points: vec![start_pos],
                            }),
                        });
                        if let Err(_e) = editor_ctx.execute_command(Box::new(command)) {
                            editor_ctx.set_feedback("Failed to start selection", FeedbackLevel::Error);
                        }
                    }
                }
            }

            if canvas_response.dragged() {
                if let Some(pos) = canvas_response.hover_pos() {
                    let doc_pos = pos - canvas_rect.min.to_vec2();
                    if renderer.current_tool() == Tool::Selection {
                        let current_pos = egui::pos2(doc_pos.x, doc_pos.y);
                        if let EditorState::Selecting { mode, in_progress } = editor_ctx.current_state() {
                            if let Some(mut selection) = in_progress.clone() {
                                selection.current = current_pos;
                                if mode == &SelectionMode::Freeform {
                                    selection.points.push(current_pos);
                                }
                                let command = Command::BeginOperation(EditorState::Selecting { 
                                    mode: *mode,
                                    in_progress: Some(selection),
                                });
                                if let Err(_e) = editor_ctx.execute_command(Box::new(command)) {
                                    editor_ctx.set_feedback("Failed to update selection", FeedbackLevel::Error);
                                }
                            }
                        }
                    }
                }
            }

            if canvas_response.drag_stopped() {
                if let Some(pos) = canvas_response.hover_pos() {
                    let doc_pos = pos - canvas_rect.min.to_vec2();
                    if renderer.current_tool() == Tool::Selection {
                        let current_pos = egui::pos2(doc_pos.x, doc_pos.y);
                        if let EditorState::Selecting { mode, in_progress } = editor_ctx.current_state() {
                            if let Some(selection) = in_progress {
                                let shape = match mode {
                                    SelectionMode::Rectangle => {
                                        SelectionShape::Rectangle(egui::Rect::from_two_pos(selection.start, current_pos))
                                    },
                                    SelectionMode::Freeform => {
                                        let mut points = selection.points.clone();
                                        points.push(current_pos);
                                        SelectionShape::Freeform(points)
                                    }
                                };
                                let command = Command::SetSelection {
                                    selection: Selection { shape },
                                };
                                if let Err(_e) = editor_ctx.execute_command(Box::new(command)) {
                                    editor_ctx.set_feedback("Failed to complete selection", FeedbackLevel::Error);
                                }
                            }
                            let command = Command::EndOperation;
                            if let Err(_e) = editor_ctx.execute_command(Box::new(command)) {
                                editor_ctx.set_feedback("Failed to end selection operation", FeedbackLevel::Error);
                            }
                        }
                    }
                }
            }

            // Update document and renderer if changed
            let new_document = editor_ctx.document.clone();
            let new_renderer = editor_ctx.renderer.clone();
            
            if new_document != self.document {
                self.document = new_document;
            }
            if let Some(renderer) = &mut self.renderer {
                if new_renderer != *renderer {
                    *renderer = new_renderer;
                }
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
        // Create input state from egui context
        let mut input_state = InputState::from_egui(ctx);
        log!("[DEBUG] Raw egui input: pointer.button_down: {:?}, pointer.hover_pos: {:?}",
            ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary)),
            ctx.input(|i| i.pointer.hover_pos()));
        
        // First handle tool state
        let current_tool = self.renderer.as_ref().map(|r| r.current_tool()).unwrap_or(Tool::Brush);
        
        // Check for tool changes
        if let Some(renderer) = &mut self.renderer {
            if renderer.set_tool(current_tool) {
                self.handle_tool_change(current_tool);
            }
        }

        // Check for dropped files
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            for file in dropped_files {
                self.handle_dropped_file(file);
            }
        }

        // Create editor context for input handling
        let mut editor_ctx = EditorContext::new(
            self.document.clone(),
            self.renderer.clone().unwrap_or_default(),
        );

        // Set the current tool in the editor context
        if let Some(renderer) = &self.renderer {
            match renderer.current_tool() {
                Tool::Brush => {
                    editor_ctx.current_tool = ToolType::Brush(crate::tool::BrushTool::default());
                }
                Tool::Eraser => {
                    editor_ctx.current_tool = ToolType::Eraser(crate::tool::EraserTool::default());
                }
                Tool::Selection => {
                    editor_ctx.current_tool = ToolType::Selection(crate::tool::SelectionTool::default());
                }
            }
        }

        // Process input through the router
        self.input_router.handle_input(&mut editor_ctx, &mut input_state);

        // Update document and renderer if changed
        let new_document = editor_ctx.document.clone();
        let new_renderer = editor_ctx.renderer.clone();
        
        if new_document != self.document {
            self.document = new_document;
        }
        if let Some(renderer) = &mut self.renderer {
            if new_renderer != *renderer {
                *renderer = new_renderer;
            }
        }

        // Add top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        // Create a new document through the command system
                        let command = Command::BeginOperation(EditorState::Idle);
                        if let Err(e) = self.document.execute_command(command) {
                            eprintln!("Failed to create new document: {}", e);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Open...").clicked() {
                        // TODO: Implement file open dialog
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        // TODO: Implement save command
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        // TODO: Implement save as command
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        // TODO: Check for unsaved changes through command system
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let can_undo = self.document.history.can_undo();
                    if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
                        let command = Command::Undo;
                        if let Err(e) = self.document.execute_command(command) {
                            eprintln!("Failed to undo: {}", e);
                        }
                        ui.close_menu();
                    }
                    
                    let can_redo = self.document.history.can_redo();
                    if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
                        let command = Command::Redo;
                        if let Err(e) = self.document.execute_command(command) {
                            eprintln!("Failed to redo: {}", e);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    
                    if ui.button("Cut").clicked() {
                        // TODO: Implement cut command
                        ui.close_menu();
                    }
                    if ui.button("Copy").clicked() {
                        // TODO: Implement copy command
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        // TODO: Implement paste command
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Preferences...").clicked() {
                        // TODO: Show preferences dialog
                        ui.close_menu();
                    }
                });

                ui.menu_button("Layer", |ui| {
                    if ui.button("New Layer").clicked() {
                        let layer_name = format!("Layer {}", self.document.layers.len());
                        let command = Command::AddLayer { 
                            name: layer_name,
                            texture: None,
                        };
                        if let Err(e) = editor_ctx.execute_command(Box::new(command)) {
                            editor_ctx.set_feedback("Failed to add layer", FeedbackLevel::Error);
                        }
                    }
                    if ui.button("Delete Layer").clicked() {
                        if let Some(active_layer) = self.document.active_layer {
                            // TODO: Implement delete layer command
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Merge Down").clicked() {
                        // TODO: Implement merge down command
                        ui.close_menu();
                    }
                });
            });
        });

        // Add left panel for tools
        egui::SidePanel::left("tools_panel")
            .exact_width(48.0)
            .resizable(false)
            .frame(egui::Frame::none()
                .outer_margin(egui::Margin::symmetric(0.0, 0.0))
                .inner_margin(egui::Vec2::ZERO))
            .show(ctx, |ui| {
                if let Some(renderer) = &mut self.renderer {
                    renderer.render_tools_panel(ui, &mut self.document);
                }
            });

        // After the left tools panel and before the right layers panel
        egui::SidePanel::right("tool_properties_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Tool Properties");
                ui.separator();

                if let Some(renderer) = &mut self.renderer {
                    // Create editor context for tool property updates
                    let mut editor_ctx = EditorContext::new(
                        self.document.clone(),
                        renderer.clone(),
                    );

                    match renderer.current_tool() {
                        Tool::Brush => {
                            ui.label("Brush Properties");
                            ui.add_space(8.0);
                            
                            // Color picker
                            let mut color = renderer.brush_color();
                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    let command = Command::SetToolProperty {
                                        tool: ToolType::Brush(crate::tool::BrushTool::default()),
                                        property: "color".into(),
                                        value: ToolPropertyValue::Color(color),
                                    };
                                    if let Err(e) = editor_ctx.execute_command(Box::new(command)) {
                                        editor_ctx.set_feedback("Failed to set brush color", FeedbackLevel::Error);
                                    }
                                }
                            });

                            // Thickness slider
                            let mut thickness = renderer.brush_thickness();
                            ui.horizontal(|ui| {
                                ui.label("Size:");
                                if ui.add(egui::Slider::new(&mut thickness, 1.0..=100.0)).changed() {
                                    let command = Command::SetToolProperty {
                                        tool: ToolType::Brush(crate::tool::BrushTool::default()),
                                        property: "thickness".into(),
                                        value: ToolPropertyValue::Thickness(thickness),
                                    };
                                    if let Err(e) = editor_ctx.execute_command(Box::new(command)) {
                                        editor_ctx.set_feedback("Failed to set brush size", FeedbackLevel::Error);
                                    }
                                }
                            });

                            // Additional brush properties can be added here
                            ui.checkbox(&mut false, "Pressure Sensitivity"); // TODO: Implement
                            ui.checkbox(&mut false, "Stabilization"); // TODO: Implement
                        }
                        Tool::Eraser => {
                            ui.label("Eraser Properties");
                            ui.add_space(8.0);
                            
                            // Thickness slider
                            let mut thickness = renderer.brush_thickness();
                            ui.horizontal(|ui| {
                                ui.label("Size:");
                                if ui.add(egui::Slider::new(&mut thickness, 1.0..=100.0)).changed() {
                                    let command = Command::SetToolProperty {
                                        tool: ToolType::Eraser(crate::tool::EraserTool::default()),
                                        property: "thickness".into(),
                                        value: ToolPropertyValue::Thickness(thickness),
                                    };
                                    if let Err(e) = editor_ctx.execute_command(Box::new(command)) {
                                        editor_ctx.set_feedback("Failed to set eraser size", FeedbackLevel::Error);
                                    }
                                }
                            });

                            // Additional eraser properties
                            ui.checkbox(&mut false, "Pressure Sensitivity"); // TODO: Implement
                            ui.checkbox(&mut false, "Preserve Alpha"); // TODO: Implement
                        }
                        Tool::Selection => {
                            ui.label("Selection Properties");
                            ui.add_space(8.0);
                            
                            // Selection mode
                            ui.horizontal(|ui| {
                                ui.label("Mode:");
                                let mut current_mode = renderer.selection_mode();
                                egui::ComboBox::new("selection_mode", "")
                                    .selected_text(format!("{:?}", current_mode))
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_value(&mut current_mode, crate::selection::SelectionMode::Rectangle, "Rectangle").clicked() {
                                            let command = Command::SetToolProperty {
                                                tool: ToolType::Selection(crate::tool::SelectionTool::default()),
                                                property: "mode".into(),
                                                value: ToolPropertyValue::SelectionMode(crate::selection::SelectionMode::Rectangle),
                                            };
                                            if let Err(e) = editor_ctx.execute_command(Box::new(command)) {
                                                editor_ctx.set_feedback("Failed to set selection mode", FeedbackLevel::Error);
                                            }
                                        }
                                        if ui.selectable_value(&mut current_mode, crate::selection::SelectionMode::Freeform, "Freeform").clicked() {
                                            let command = Command::SetToolProperty {
                                                tool: ToolType::Selection(crate::tool::SelectionTool::default()),
                                                property: "mode".into(),
                                                value: ToolPropertyValue::SelectionMode(crate::selection::SelectionMode::Freeform),
                                            };
                                            if let Err(e) = editor_ctx.execute_command(Box::new(command)) {
                                                editor_ctx.set_feedback("Failed to set selection mode", FeedbackLevel::Error);
                                            }
                                        }
                                    });
                            });

                            // Additional selection properties
                            ui.checkbox(&mut false, "Anti-aliasing"); // TODO: Implement
                            ui.checkbox(&mut false, "Show Measurements"); // TODO: Implement
                        }
                    }

                    // Update document and renderer if changed
                    if editor_ctx.document != self.document {
                        self.document = editor_ctx.document;
                    }
                    if editor_ctx.renderer != *renderer {
                        *renderer = editor_ctx.renderer;
                    }
                }
            });

        // Right layers panel
        egui::SidePanel::right("layers_panel").show(ctx, |ui| {
            ui.heading("Layers");
            
            // Create editor context for layer operations
            let mut editor_ctx = EditorContext::new(
                self.document.clone(),
                self.renderer.clone().unwrap_or_default(),
            );
            
            if ui.button("+ Add Layer").clicked() {
                let layer_name = format!("Layer {}", self.document.layers.len());
                let command = Command::AddLayer { 
                    name: layer_name,
                    texture: None,
                };
                if let Err(e) = editor_ctx.execute_command(Box::new(command)) {
                    editor_ctx.set_feedback("Failed to add layer", FeedbackLevel::Error);
                }
            }
            
            ui.separator();

            let text_height = ui.text_style_height(&egui::TextStyle::Body);
            let layer_height = text_height * 1.5;

            // Get the response area for the layer list
            let layer_list_height = layer_height * self.document.layers.len() as f32;
            let (layer_list_rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), layer_list_height),
                egui::Sense::hover(),
            );

            // Collect layer operations to avoid borrow checker issues
            let mut layer_operations: Vec<Box<dyn FnOnce(&mut Document)>> = Vec::new();

            // Handle layer operations
            for (index, layer) in self.document.layers.iter().enumerate() {
                let layer_rect = egui::Rect::from_min_size(
                    egui::pos2(layer_list_rect.min.x, layer_list_rect.min.y + layer_height * index as f32),
                    egui::vec2(layer_list_rect.width(), layer_height),
                );

                // Layer visibility toggle
                let visibility_rect = egui::Rect::from_min_size(
                    layer_rect.min,
                    egui::vec2(layer_height, layer_height),
                );
                if ui.put(visibility_rect, egui::SelectableLabel::new(layer.visible, "👁")).clicked() {
                    let idx = index;
                    layer_operations.push(Box::new(move |doc: &mut Document| {
                        doc.toggle_layer_visibility(idx);
                    }));
                }

                // Layer name/selection
                let name_rect = egui::Rect::from_min_max(
                    visibility_rect.max,
                    layer_rect.max,
                );
                let is_active = Some(index) == self.document.active_layer;
                if ui.put(name_rect, egui::SelectableLabel::new(is_active, &layer.name)).clicked() {
                    let idx = index;
                    layer_operations.push(Box::new(move |doc: &mut Document| {
                        doc.active_layer = Some(idx);
                    }));
                }
            }

            // Apply collected operations
            for op in layer_operations {
                op(&mut self.document);
            }

            // Update document if changed
            if editor_ctx.document.layers.len() != self.document.layers.len() ||
               editor_ctx.document.active_layer != self.document.active_layer {
                self.document = editor_ctx.document;
            }
        });

        // Central panel with canvas
        egui::CentralPanel::default()
            .frame(egui::Frame::none())  // Remove any frame styling
            .show(ctx, |ui| {
                let canvas_rect = ui.max_rect();
                let painter = ui.painter();

                // Clear the background
                painter.rect_filled(
                    canvas_rect,
                    0.0,
                    egui::Color32::WHITE,
                );

                // Make the entire area interactive
                let canvas_response = ui.interact(
                    canvas_rect,
                    ui.id().with("canvas_area"),
                    egui::Sense::click_and_drag()
                );

                self.logger.log_input(&canvas_response);
                self.last_canvas_rect = Some(canvas_rect);

                // Update input state with canvas-relative coordinates
                if let Some(pos) = canvas_response.interact_pointer_pos() {
                    let doc_pos = pos - canvas_rect.min.to_vec2();
                    input_state.pointer_pos = Some(egui::pos2(doc_pos.x, doc_pos.y));
                }

                // Update input state based on canvas response
                input_state.pointer_pressed = canvas_response.clicked() || canvas_response.drag_started();
                input_state.pointer_released = canvas_response.clicked() || canvas_response.drag_released();

                // Draw all layers
                for layer in &self.document.layers {
                    self.render_layer(&painter, canvas_rect, layer);
                }

                // Draw in-progress stroke preview (if any)
                if !self.current_stroke.points.is_empty() {
                    let preview_points: Vec<egui::Pos2> = self.current_stroke.points.iter()
                        .map(|p| *p + canvas_rect.min.to_vec2())
                        .collect();
                    painter.add(egui::Shape::line(
                        preview_points,
                        egui::Stroke::new(self.current_stroke.thickness, self.current_stroke.color),
                    ));
                }

                // Create editor context for tool updates
                let mut editor_ctx = EditorContext::new(
                    self.document.clone(),
                    self.renderer.clone().unwrap_or_default(),
                );

                // Set the current tool in the editor context
                if let Some(renderer) = &self.renderer {
                    match renderer.current_tool() {
                        Tool::Brush => {
                            editor_ctx.current_tool = ToolType::Brush(crate::tool::BrushTool::default());

                            // Use raw input instead of canvas_response for brush events
                            let raw_hover = ctx.input(|i| i.pointer.hover_pos());
                            let is_down = ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
                            log!("[DEBUG] Raw input: button_down={}, hover_pos={:?}", is_down, raw_hover);

                            // Start a new stroke if the primary button is down and no stroke is active
                            if is_down && self.current_stroke.points.is_empty() {
                                self.logger.log_brush("Brush: Starting new stroke");
                                if let Some(pos) = raw_hover {
                                    let clamped = egui::pos2(pos.x.clamp(canvas_rect.left(), canvas_rect.right()), pos.y.clamp(canvas_rect.top(), canvas_rect.bottom()));
                                    let doc_pos = clamped - canvas_rect.min.to_vec2();
                                    self.current_stroke = Stroke::new(renderer.brush_color(), renderer.brush_thickness());
                                    self.current_stroke.add_point(doc_pos);
                                }
                                if !matches!(editor_ctx.current_state(), EditorState::Drawing { .. }) {
                                    log!("[DEBUG] Brush: Forcing transition to Drawing state");
                                    editor_ctx.transition_to(EditorState::Drawing {
                                        tool: DrawingTool::Brush(crate::tool::BrushTool::default()),
                                        stroke: Some(self.current_stroke.clone()),
                                    }).unwrap_or_else(|e| eprintln!("[ERROR] Force transition failed: {:?}", e));
                                }
                            }

                            // Continue the stroke if the button remains down and a stroke is active
                            if is_down && !self.current_stroke.points.is_empty() {
                                if let Some(pos) = raw_hover {
                                    let clamped = egui::pos2(pos.x.clamp(canvas_rect.left(), canvas_rect.right()), pos.y.clamp(canvas_rect.top(), canvas_rect.bottom()));
                                    let doc_pos = clamped - canvas_rect.min.to_vec2();
                                    log!("[DEBUG] Brush: Adding point at {:?}", doc_pos);
                                    self.current_stroke.add_point(doc_pos);
                                }
                            }

                            // End the stroke if the button is released and a stroke is active
                            if !is_down && !self.current_stroke.points.is_empty() {
                                self.logger.log_brush("Brush: Ending stroke and committing");
                                self.commit_current_stroke();
                                editor_ctx.document = self.document.clone();
                                editor_ctx.return_to_idle().unwrap_or_else(|e| eprintln!("[ERROR] Failed to return to idle state: {:?}", e));
                            }
                        }
                        Tool::Eraser => {
                            editor_ctx.current_tool = ToolType::Eraser(crate::tool::EraserTool::default());
                        }
                        Tool::Selection => {
                            editor_ctx.current_tool = ToolType::Selection(crate::tool::SelectionTool::default());
                        }
                    }
                }

                // Process input through the router
                self.input_router.handle_input(&mut editor_ctx, &mut input_state);

                // Handle selection input
                if let Some(renderer) = &self.renderer {
                    if renderer.current_tool() == Tool::Selection {
                        self.handle_selection_input(&canvas_response, canvas_rect, &painter);
                    }
                }

                // Draw transform gizmo if active
                if let Some(gizmo) = &mut self.transform_gizmo {
                    gizmo.render(&painter);
                }
            });

        // Add status bar with feedback at the bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Show current feedback message if any
                if let Some((message, level)) = self.document.current_feedback() {
                    let color = match level {
                        FeedbackLevel::Info => egui::Color32::WHITE,
                        FeedbackLevel::Success => egui::Color32::GREEN,
                        FeedbackLevel::Warning => egui::Color32::YELLOW,
                        FeedbackLevel::Error => egui::Color32::RED,
                    };
                    ui.colored_label(color, message);
                }
            });
        });
    }
}