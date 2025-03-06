// src/renderer.rs
use eframe::egui;
use crate::stroke::{Stroke, StrokeRef};
use crate::document::Document;
use crate::image::Image;
use crate::state::ElementType;
use crate::widgets::{ResizeHandle, Corner};
use std::collections::HashMap;
use log::{info};

pub struct Renderer {
    _gl: Option<std::sync::Arc<eframe::glow::Context>>,
    preview_stroke: Option<StrokeRef>,
    // Track active resize handles
    active_handles: HashMap<usize, Corner>,
    // Track resize preview rectangle
    resize_preview: Option<egui::Rect>,
    // Track drag preview rectangle
    drag_preview: Option<egui::Rect>,
}

impl Renderer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone();
        
        Self {
            _gl: gl,
            preview_stroke: None,
            active_handles: HashMap::new(),
            resize_preview: None,
            drag_preview: None,
        }
    }

    pub fn set_preview_stroke(&mut self, stroke: Option<StrokeRef>) {
        self.preview_stroke = stroke;
    }

    pub fn set_resize_preview(&mut self, rect: Option<egui::Rect>) {
        self.resize_preview = rect;
    }
    
    pub fn set_drag_preview(&mut self, rect: Option<egui::Rect>) {
        self.drag_preview = rect;
    }
    
    pub fn is_handle_active(&self, element_id: usize) -> bool {
        self.active_handles.contains_key(&element_id)
    }
    
    pub fn get_active_handle(&self, element_id: usize) -> Option<&Corner> {
        self.active_handles.get(&element_id)
    }
    
    pub fn set_active_handle(&mut self, element_id: usize, corner: Corner) {
        self.active_handles.insert(element_id, corner);
    }
    
    pub fn clear_active_handle(&mut self, element_id: usize) {
        self.active_handles.remove(&element_id);
    }
    
    pub fn clear_all_active_handles(&mut self) {
        self.active_handles.clear();
    }

    pub fn any_handles_active(&self) -> bool {
        !self.active_handles.is_empty()
    }

    fn draw_stroke(&self, painter: &egui::Painter, stroke: &Stroke) {
        let points = stroke.points();
        if points.len() < 2 {
            return;
        }

        for points in points.windows(2) {
            painter.line_segment(
                [points[0], points[1]],
                egui::Stroke::new(stroke.thickness(), stroke.color()),
            );
        }
    }

    fn draw_image(&self, ctx: &egui::Context, painter: &egui::Painter, image: &Image) {
        // Create a new texture from the image data
        let width = image.size().x as usize;
        let height = image.size().y as usize;
        let data = image.data();
        
        // Create the color image from RGBA data
        let color_image = if data.len() == width * height * 4 {
            // Data is already in RGBA format and dimensions match
            egui::ColorImage::from_rgba_unmultiplied(
                [width, height],
                data,
            )
        } else if data.len() % 4 == 0 {
            // Data length is divisible by 4 (valid RGBA data), but dimensions don't match
            // Try to estimate dimensions from data length
            let total_pixels = data.len() / 4;
            
            // Option 1: Maintain aspect ratio but adjust dimensions to fit data
            let new_height = (total_pixels as f32 / width as f32).round() as usize;
            if new_height > 0 {
                // Use the original width but adjust height to fit data
                egui::ColorImage::from_rgba_unmultiplied(
                    [width, new_height],
                    data,
                )
            } else {
                // Option 2: Try a square approximation
                let side = (total_pixels as f32).sqrt().round() as usize;
                if side > 0 && side * side * 4 <= data.len() {
                    egui::ColorImage::from_rgba_unmultiplied(
                        [side, side],
                        &data[0..side * side * 4],
                    )
                } else {
                    // Fallback to a red placeholder if all else fails
                    println!("Image data mismatch: size={:?}, data length={}, expected={}", 
                             image.size(), data.len(), width * height * 4);
                    egui::ColorImage::new([width, height], egui::Color32::RED)
                }
            }
        } else {
            // Data isn't even valid RGBA (not divisible by 4)
            println!("Invalid image data: length {} is not divisible by 4", data.len());
            egui::ColorImage::new([width, height], egui::Color32::RED)
        };
        
        // Load the texture (this will be automatically freed at the end of the frame)
        let texture = ctx.load_texture(
            format!("image_{}", image.id()),
            color_image,
            egui::TextureOptions::default(),
        );
        
        // Draw the image at its position with its size
        let rect = image.rect();
        
        // Use the full texture (UV coordinates from 0,0 to 1,1)
        let uv = egui::Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 1.0)
        );
        
        painter.image(texture.id(), rect, uv, egui::Color32::WHITE);
    }

    fn draw_selection_box(&self, ui: &mut egui::Ui, element: &ElementType) -> Vec<egui::Response> {
        // Get the element's bounding rectangle using compute_element_rect
        let rect = crate::geometry::hit_testing::compute_element_rect(element);
        
        // Draw the selection box with a more visible stroke
        ui.painter().rect_stroke(
            rect,
            0.0, // no rounding
            egui::Stroke::new(2.0, egui::Color32::from_rgb(30, 120, 255)), // Thicker, brighter blue
        );
        
        // Draw the resize handles at each corner
        let handle_size = crate::geometry::hit_testing::RESIZE_HANDLE_RADIUS / 2.0;
        
        let corners = [
            (rect.left_top(), Corner::TopLeft),
            (rect.right_top(), Corner::TopRight),
            (rect.left_bottom(), Corner::BottomLeft),
            (rect.right_bottom(), Corner::BottomRight),
        ];
        
        for (pos, corner) in corners {
            // Create a temporary handle for drawing
            let _handle = ResizeHandle::new(0, corner, pos, handle_size);
            ui.painter().circle_filled(
                pos,
                handle_size,
                egui::Color32::from_rgb(200, 200, 200)
            );
            
            ui.painter().circle_stroke(
                pos,
                handle_size,
                egui::Stroke::new(1.0, egui::Color32::BLACK)
            );
        }
        
        // We don't need to return responses here anymore since we handle them in process_resize_interactions
        Vec::new()
    }

    // Update the render method to handle both resize and drag previews
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        document: &Document,
        selected_elements: &[ElementType],
    ) -> Option<(usize, Corner, egui::Pos2)> {
        // Process interactions first before drawing
        let resize_info = self.process_resize_interactions(ui, selected_elements, document);
        
        // Add logging to see when resize_info is returned
        if let Some((element_id, corner, pos)) = resize_info {
            info!("Returning resize info: element={}, corner={:?}, pos={:?}", 
                 element_id, corner, pos);
        }
        
        // Draw background
        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
        
        // Draw non-selected elements normally
        for stroke in document.strokes() {
            let stroke_id = std::sync::Arc::as_ptr(stroke) as usize;
            if !selected_elements.iter().any(|e| match e {
                ElementType::Stroke(s) => std::sync::Arc::as_ptr(s) as usize == stroke_id,
                _ => false,
            }) {
                self.draw_stroke(ui.painter(), stroke);
            }
        }
        
        for image in document.images() {
            let image_id = image.id();
            if !selected_elements.iter().any(|e| match e {
                ElementType::Image(i) => i.id() == image_id,
                _ => false,
            }) {
                self.draw_image(ctx, ui.painter(), image);
            }
        }
        
        // Draw selected elements (either in their original position or preview position)
        for element in selected_elements {
            match element {
                ElementType::Stroke(stroke) => {
                    // For now, strokes are drawn normally during resize
                    self.draw_stroke(ui.painter(), stroke);
                }
                ElementType::Image(image) => {
                    // If we're resizing and this is the selected image, draw it in the preview rect
                    if let Some(preview_rect) = self.resize_preview {
                        // Create a new texture from the image data
                        let width = image.size().x as usize;
                        let height = image.size().y as usize;
                        
                        let color_image = if image.data().len() == width * height * 4 {
                            egui::ColorImage::from_rgba_unmultiplied(
                                [width, height],
                                image.data(),
                            )
                        } else {
                            // Instead of showing a red rectangle, we should try to render the image
                            // with its original dimensions, even if we're resizing it
                            println!("Image data size mismatch: expected {}x{}x4={}, got {}", 
                                width, height, width * height * 4, image.data().len());
                                
                            // Try to create a properly sized color image
                            let data_len = image.data().len();
                            if data_len % 4 == 0 {
                                // Estimate dimensions based on data length
                                let pixel_count = data_len / 4;
                                let estimated_width = (pixel_count as f32).sqrt() as usize;
                                let estimated_height = (pixel_count + estimated_width - 1) / estimated_width;
                                
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [estimated_width, estimated_height],
                                    image.data(),
                                )
                            } else {
                                // Fallback to red if we can't make sense of the data
                                egui::ColorImage::new([width, height], egui::Color32::RED)
                            }
                        };
                        
                        let texture = ctx.load_texture(
                            format!("image_{}", image.id()),
                            color_image,
                            egui::TextureOptions::default(),
                        );
                        
                        // Use the preview rect instead of the image's original rect
                        let uv = egui::Rect::from_min_max(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(1.0, 1.0)
                        );
                        
                        // Draw the image at the preview position
                        ui.painter().image(texture.id(), preview_rect, uv, egui::Color32::WHITE);
                        
                        // Draw a light border around the preview
                        ui.painter().rect_stroke(
                            preview_rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(100, 100, 255, 100)),
                        );
                    } else {
                        // Draw normally if not being resized
                        self.draw_image(ctx, ui.painter(), image);
                    }
                }
            }
        }
        
        // Draw preview stroke if any
        if let Some(preview) = &self.preview_stroke {
            self.draw_stroke(ui.painter(), preview);
        }
        
        // Draw selection boxes for selected elements (only if not resizing)
        if self.resize_preview.is_none() {
            for element in selected_elements {
                self.draw_selection_box(ui, element);
            }
        } else if let Some(preview_rect) = self.resize_preview {
            // Draw selection box around the preview rect during resize
            ui.painter().rect_stroke(
                preview_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(30, 120, 255)),
            );
            
            // Draw resize handles at preview rect corners
            let handle_size = crate::geometry::hit_testing::RESIZE_HANDLE_RADIUS / 2.0;
            let corners = [
                (preview_rect.left_top(), Corner::TopLeft),
                (preview_rect.right_top(), Corner::TopRight),
                (preview_rect.left_bottom(), Corner::BottomLeft),
                (preview_rect.right_bottom(), Corner::BottomRight),
            ];
            
            for (pos, _corner) in corners {
                // Draw simple visual representation of the handle
                ui.painter().circle_filled(
                    pos,
                    handle_size,
                    egui::Color32::from_rgb(200, 200, 200)
                );
                
                ui.painter().circle_stroke(
                    pos,
                    handle_size,
                    egui::Stroke::new(1.0, egui::Color32::BLACK)
                );
            }
        }
        
        // Draw drag preview if any
        if let Some(preview_rect) = self.drag_preview {
            ui.painter().rect_stroke(
                preview_rect,
                0.0, // no rounding
                egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(30, 255, 120, 180)), // Semi-transparent green
            );
        }
        
        resize_info
    }

    pub fn process_resize_interactions(
        &mut self,
        ui: &mut egui::Ui,
        selected_elements: &[ElementType],
        _document: &Document,
    ) -> Option<(usize, Corner, egui::Pos2)> {
        let mut resize_info = None;
        
        if selected_elements.is_empty() {
            return None;
        }
        
        // Handle size in screen pixels
        let handle_size = 8.0;
        
        // Process each selected element
        for element in selected_elements {
            let element_id = match element {
                ElementType::Image(img) => img.id(),
                ElementType::Stroke(s) => std::sync::Arc::as_ptr(s) as usize,
            };
            
            // Get the element's rectangle
            let rect = crate::geometry::hit_testing::compute_element_rect(element);
            
            // Log the element and its rectangle
            log::info!("Processing element {} with rect {:?}", element_id, rect);
            
            // Skip processing if element has zero size
            if rect.width() < 1.0 || rect.height() < 1.0 {
                log::warn!("Skipping resize processing for element {} with zero size", element_id);
                continue;
            }
            
            // Process each corner
            for corner in &[Corner::TopLeft, Corner::TopRight, Corner::BottomLeft, Corner::BottomRight] {
                log::info!("Processing corner {:?} for element {}", corner, element_id);
                
                // Calculate the position of this corner
                let corner_pos = match corner {
                    Corner::TopLeft => rect.left_top(),
                    Corner::TopRight => rect.right_top(),
                    Corner::BottomLeft => rect.left_bottom(),
                    Corner::BottomRight => rect.right_bottom(),
                };
                
                // Create a resize handle for this corner
                let handle = ResizeHandle::new(element_id, *corner, corner_pos, handle_size);
                
                // Show the handle and get interaction response
                let response = handle.show(ui);
                
                // Check for active drag more explicitly
                if response.dragged() {
                    log::info!("Handle being dragged for element {}, corner {:?}, delta: {:?}", 
                              element_id, corner, response.drag_delta());
                    
                    // If this is a new drag (no active handles yet), set this as the active handle
                    if !self.is_handle_active(element_id) {
                        log::info!("Setting active handle for element {}, corner {:?}", element_id, corner);
                        self.set_active_handle(element_id, *corner);
                    }
                    
                    // Always update active handle to the current corner being dragged
                    if self.is_handle_active(element_id) {
                        self.set_active_handle(element_id, *corner);
                        
                        // Get the current mouse position for the resize
                        let mouse_pos = response.hover_pos()
                            .or_else(|| ui.ctx().pointer_hover_pos())
                            .unwrap_or(corner_pos);
                        
                        // Compute the new rectangle based on this drag position
                        let new_rect = Self::compute_resized_rect(rect, *corner, mouse_pos);
                        
                        // Update the resize preview
                        self.set_resize_preview(Some(new_rect));
                        
                        // Return the resize information (element ID, corner, new position)
                        resize_info = Some((element_id, *corner, mouse_pos));
                        log::info!("Setting resize_info: element={}, corner={:?}, pos={:?}", 
                                  element_id, corner, mouse_pos);
                    }
                }
                
                // Handle drag release - clear active handle for this element
                if response.drag_stopped() {
                    log::info!("Drag released for element {}, corner {:?}", element_id, corner);
                    
                    // Get the final resize preview rect
                    if let Some(final_rect) = self.resize_preview {
                        log::info!("Applying final resize: {:?}", final_rect);
                        
                        // Return the resize info so the selection tool can update the element
                        resize_info = Some((element_id, *corner, response.hover_pos().unwrap_or(response.interact_pointer_pos().unwrap())));
                    }
                    
                    self.clear_active_handle(element_id);
                }
            }
        }
        
        // If no resize is in progress, clear all previews
        if resize_info.is_none() {
            log::info!("No resize_info to return");
            // If we don't have active handles, clear the preview
            if !self.any_handles_active() {
                self.set_resize_preview(None);
            }
        }
        
        resize_info
    }

    pub fn get_resize_preview(&self) -> Option<egui::Rect> {
        self.resize_preview
    }
    
    pub fn compute_resized_rect(original: egui::Rect, corner: Corner, new_pos: egui::Pos2) -> egui::Rect {
        let mut rect = original;
        
        match corner {
            Corner::TopLeft => {
                rect.min.x = new_pos.x.min(rect.max.x - 10.0);
                rect.min.y = new_pos.y.min(rect.max.y - 10.0);
            }
            Corner::TopRight => {
                rect.max.x = new_pos.x.max(rect.min.x + 10.0);
                rect.min.y = new_pos.y.min(rect.max.y - 10.0);
            }
            Corner::BottomLeft => {
                rect.min.x = new_pos.x.min(rect.max.x - 10.0);
                rect.max.y = new_pos.y.max(rect.min.y + 10.0);
            }
            Corner::BottomRight => {
                rect.max.x = new_pos.x.max(rect.min.x + 10.0);
                rect.max.y = new_pos.y.max(rect.min.y + 10.0);
            }
        }
        
        rect
    }

    pub fn get_active_corner(&self, element_id: usize) -> Option<&Corner> {
        self.get_active_handle(element_id)
    }
}