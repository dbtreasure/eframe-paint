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
    // Texture cache with version tracking
    texture_cache: HashMap<usize, u64>,
    // Generate unique keys for texture names to prevent reuse
    texture_keys: HashMap<usize, u64>,
    // Counter for generating unique texture keys
    texture_id_counter: u64,
    // Track elements that need texture refresh
    elements_to_refresh: Vec<usize>,
    // Track elements rendered this frame
    elements_rendered_this_frame: std::collections::HashSet<usize>,
    // Track elements rendered in the previous frame
    elements_rendered_last_frame: std::collections::HashSet<usize>,
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
            texture_cache: HashMap::new(),
            texture_keys: HashMap::new(),
            texture_id_counter: 1,
            elements_to_refresh: Vec::new(),
            elements_rendered_this_frame: std::collections::HashSet::new(),
            elements_rendered_last_frame: std::collections::HashSet::new(),
        }
    }
    
    // Add methods to track frame rendering
    pub fn begin_frame(&mut self) {
        // Clear the set of elements rendered this frame
        self.elements_rendered_this_frame.clear();
    }
    
    pub fn end_frame(&mut self, ctx: &egui::Context) {
        // Find elements that were rendered previously but not this frame
        let outdated: Vec<_> = self.elements_rendered_last_frame
            .difference(&self.elements_rendered_this_frame)
            .copied()
            .collect();
        
        // Clean up outdated textures
        let has_outdated = !outdated.is_empty();
        for element_id in outdated {
            self.texture_cache.remove(&element_id);
            info!("Removed outdated texture for element {}", element_id);
        }
        
        // Swap collections for next frame
        std::mem::swap(&mut self.elements_rendered_last_frame, 
                     &mut self.elements_rendered_this_frame);
                     
        // Force repaint if we had outdated elements
        if has_outdated {
            ctx.request_repaint();
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

    // Completely rewritten draw_image method using an ephemeral texture approach
fn draw_image(&mut self, ctx: &egui::Context, painter: &egui::Painter, image: &Image) {
        // Get dimensions and data
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
        
        // Critical change: Create completely EPHEMERAL textures on every frame
        // This prevents caching issues by forcing egui to create fresh textures
        // Generate a unique name with the frame counter to ensure uniqueness
        let image_id = image.id();
        
        // Always increment the counter to ensure unique texture names
        self.texture_id_counter += 1;
        
        // Each texture now gets a unique name every time it's drawn - no caching!
        let unique_texture_name = format!("ephemeral_img_{}_{}", 
                                         image_id, 
                                         self.texture_id_counter);
        
        info!("Drawing image {} with ephemeral texture name {}", image_id, unique_texture_name);
        
        // Load the texture with the unique name - egui will automatically free at the end of the frame
        let texture = ctx.load_texture(
            unique_texture_name,
            color_image,
            egui::TextureOptions::default(),
        );
        
        // Track that this element was rendered this frame
        self.elements_rendered_this_frame.insert(image_id);
        
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
        
        // Collection to track which elements we've already drawn
        // This prevents double rendering of the same element
        let mut drawn_element_ids = std::collections::HashSet::new();
        
        // Draw selected elements with drag/resize preview if applicable
        // We draw selected elements first to prevent original versions from showing
        for element in selected_elements {
            let element_id = element.get_stable_id();
            drawn_element_ids.insert(element_id);
            
            // If we have active previews, don't draw the original elements
            let drag_preview_active = self.drag_preview.is_some();
            let resize_preview_active = self.resize_preview.is_some();
            
            match element {
                ElementType::Stroke(stroke) => {
                    // For strokes, we show the preview if available, otherwise the original
                    // Preview is handled directly in the stroke rendering for now
                    if !resize_preview_active && !drag_preview_active {
                        self.draw_stroke(ui.painter(), stroke);
                    }
                }
                ElementType::Image(image) => {
                    // For images, handle resize preview
                    if let Some(preview_rect) = self.resize_preview {
                        // Draw resized preview instead of original
                        let width = image.size().x as usize;
                        let height = image.size().y as usize;
                        
                        let color_image = if image.data().len() == width * height * 4 {
                            egui::ColorImage::from_rgba_unmultiplied(
                                [width, height],
                                image.data(),
                            )
                        } else {
                            // Fallback for mismatched data
                            let data_len = image.data().len();
                            if data_len % 4 == 0 {
                                // Estimate dimensions
                                let pixel_count = data_len / 4;
                                let estimated_width = (pixel_count as f32).sqrt() as usize;
                                let estimated_height = (pixel_count + estimated_width - 1) / estimated_width;
                                
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [estimated_width, estimated_height],
                                    image.data(),
                                )
                            } else {
                                egui::ColorImage::new([width, height], egui::Color32::RED)
                            }
                        };
                        
                        // Similar to draw_image, create a completely ephemeral texture
                        // Always increment the counter to ensure absolute uniqueness
                        self.texture_id_counter += 1;
                        
                        // Each preview texture now gets a unique name every time it's drawn - no caching!
                        let unique_preview_name = format!("ephemeral_preview_{}_{}", 
                                                         image.id(), 
                                                         self.texture_id_counter);
                        
                        info!("Drawing preview for image {} with ephemeral texture name {}", 
                              image.id(), unique_preview_name);
                        
                        // Load the texture with the unique name - egui will automatically free it
                        let texture = ctx.load_texture(
                            unique_preview_name,
                            color_image,
                            egui::TextureOptions::default(),
                        );
                        
                        // Use full texture coordinates
                        let uv = egui::Rect::from_min_max(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(1.0, 1.0)
                        );
                        
                        // Draw preview at new position
                        ui.painter().image(texture.id(), preview_rect, uv, egui::Color32::WHITE);
                        
                        // Draw preview border
                        ui.painter().rect_stroke(
                            preview_rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(100, 100, 255, 100)),
                        );
                    } else if let Some(preview_rect) = self.drag_preview {
                        // Draw dragged preview instead of original
                        // Create texture for the dragged image
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [image.size().x as usize, image.size().y as usize],
                            image.data(),
                        );
                        
                        // Similar approach for drag preview - using ephemeral textures
                        // Always increment the counter to ensure absolute uniqueness
                        self.texture_id_counter += 1;
                        
                        // Each drag texture now gets a unique name every time it's drawn - no caching!
                        let unique_drag_name = format!("ephemeral_drag_{}_{}", 
                                                      image.id(), 
                                                      self.texture_id_counter);
                        
                        info!("Drawing drag preview for image {} with ephemeral texture name {}", 
                              image.id(), unique_drag_name);
                        
                        // Load the texture with the unique name - egui will automatically free it
                        let texture = ctx.load_texture(
                            unique_drag_name,
                            color_image,
                            egui::TextureOptions::default(),
                        );
                        
                        // Use full texture coordinates
                        let uv = egui::Rect::from_min_max(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(1.0, 1.0)
                        );
                        
                        // Draw preview at dragged position
                        ui.painter().image(texture.id(), preview_rect, uv, egui::Color32::WHITE);
                        
                        // Draw preview border
                        ui.painter().rect_stroke(
                            preview_rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(100, 100, 255, 100)),
                        );
                    } else {
                        // No preview active, draw normally
                        self.draw_image(ctx, ui.painter(), image);
                    }
                }
            }
        }
        
        // Now draw non-selected elements
        for stroke in document.strokes() {
            let stroke_element = ElementType::Stroke(stroke.clone());
            let stroke_id = stroke_element.get_stable_id();
            
            // Only draw if we haven't already drawn this element
            if !drawn_element_ids.contains(&stroke_id) {
                self.draw_stroke(ui.painter(), stroke);
                drawn_element_ids.insert(stroke_id);
            }
        }
        
        for image in document.images() {
            let image_id = image.id();
            
            // Only draw if we haven't already drawn this element
            if !drawn_element_ids.contains(&image_id) {
                self.draw_image(ctx, ui.painter(), image);
                drawn_element_ids.insert(image_id);
            }
        }
        
        // Draw preview stroke if any
        if let Some(preview) = &self.preview_stroke {
            self.draw_stroke(ui.painter(), preview);
        }
        
        // Draw selection boxes - show them on the correct (preview or actual) rect
        if let Some(preview_rect) = self.resize_preview {
            // During resize, draw selection box around the preview rect
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
                // Draw resize handles
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
        } else if let Some(preview_rect) = self.drag_preview {
            // During drag, draw selection box around the drag preview rect
            ui.painter().rect_stroke(
                preview_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(30, 120, 255)),
            );
            
            // Draw lighter outline to show it's being dragged
            ui.painter().rect_stroke(
                preview_rect.expand(2.0),
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(30, 255, 120, 180)),
            );
        } else {
            // If no preview is active, draw normal selection boxes
            for element in selected_elements {
                self.draw_selection_box(ui, element);
            }
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
            let element_id = element.get_stable_id();
            
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

    // Enhanced method to clear the renderer's state for a specific element
    pub fn clear_element_state(&mut self, element_id: usize) {
        // Remove any active handles for this element
        self.active_handles.remove(&element_id);
        
        // Clear resize preview if it's for this element - logic fixed to check active handles first
        if self.active_handles.contains_key(&element_id) {
            self.resize_preview = None;
        }
        
        // Clear drag preview if it's for this element - now always clear to be safe
        self.drag_preview = None;
        
        // Force refresh texture for this element to ensure clean state
        self.force_texture_refresh_for_element(element_id);
    }
    
    // A method to clear all element-related state (not preview strokes)
    pub fn clear_all_element_state(&mut self) {
        // Clear all state except preview strokes and texture caches
        self.active_handles.clear();
        self.resize_preview = None;
        self.drag_preview = None;
        info!("Cleared all element state in renderer");
    }
    
    // Enhanced method to reset all renderer state
    pub fn reset_state(&mut self) {
        self.active_handles.clear();
        self.resize_preview = None;
        self.drag_preview = None;
        self.preview_stroke = None;
        
        // Don't reset texture cache or keys - that's handled by invalidate_texture
        // and the frame tracking
    }
    
    // Add method to reset all texture-related state
    pub fn reset_texture_state(&mut self) {
        // Clear all texture caches
        self.texture_cache.clear();
        
        // Increment the counter by a large amount to ensure new keys
        self.texture_id_counter += 1000; 
        
        // Clear the tracked elements
        self.elements_to_refresh.clear();
        self.elements_rendered_this_frame.clear();
        self.elements_rendered_last_frame.clear();
        
        info!("Complete texture state reset performed");
    }
    
    // Enhanced method to reset only element-related state, preserving preview strokes
    pub fn reset_element_state(&mut self) {
        self.active_handles.clear();
        self.resize_preview = None;
        self.drag_preview = None;
        // Intentionally NOT clearing preview_stroke
        
        // Clear elements_to_refresh that we've accumulated
        if !self.elements_to_refresh.is_empty() {
            info!("Clearing {} elements marked for refresh", self.elements_to_refresh.len());
            self.elements_to_refresh.clear();
        }
    }
    
    // Enhanced method to force texture refresh for all elements
    pub fn force_texture_refresh(&mut self, ctx: &egui::Context) {
        // Increment the texture counter to generate new keys
        self.texture_id_counter += 100; // Large increment to ensure unique keys
        
        // Clear the texture cache entirely
        self.texture_cache.clear();
        
        // Generate new keys for all tracked elements
        for element_id in self.elements_rendered_last_frame.iter() {
            self.texture_id_counter += 1;
            self.texture_keys.insert(*element_id, self.texture_id_counter);
            info!("Generating new texture key for element {}: {}", 
                 element_id, self.texture_id_counter);
        }
        
        // Force egui to drop all textures and repaint
        ctx.request_repaint();
        
        info!("Forced texture refresh for all elements");
    }
    
    // Add method to force refresh for a specific element and its related textures
    pub fn force_texture_refresh_for_element(&mut self, element_id: usize) {
        // Clear base element
        self.texture_cache.remove(&element_id);
        self.texture_id_counter += 1;
        self.texture_keys.insert(element_id, self.texture_id_counter);
        
        // Clear preview variant (offset by 1000000)
        let preview_id = element_id + 1000000;
        self.texture_cache.remove(&preview_id);
        self.texture_id_counter += 1;
        self.texture_keys.insert(preview_id, self.texture_id_counter);
        
        // Clear drag variant (offset by 2000000)
        let drag_id = element_id + 2000000;
        self.texture_cache.remove(&drag_id);
        self.texture_id_counter += 1;
        self.texture_keys.insert(drag_id, self.texture_id_counter);
        
        info!("Forced refresh for element {} and its preview variants", element_id);
    }

    // Enhanced method to invalidate a texture for an element
    pub fn invalidate_texture(&mut self, element_id: usize) {
        // Remove from cache to force recreation on next render
        self.texture_cache.remove(&element_id);
        
        // Generate a unique key for this element to prevent Egui from reusing the texture
        self.texture_id_counter += 1;
        self.texture_keys.insert(element_id, self.texture_id_counter);
        
        // Add to refresh list if not already there
        if !self.elements_to_refresh.contains(&element_id) {
            self.elements_to_refresh.push(element_id);
            info!("Marked element {} for texture refresh with new key {}", 
                 element_id, self.texture_id_counter);
        }
    }
    
    // Add a method to handle element updates
    pub fn handle_element_update(&mut self, element: &ElementType) {
        // Use the get_stable_id method which is public
        self.invalidate_texture(element.get_stable_id());
    }
    
    // Add a debug visualization for texture state
    pub fn draw_debug_overlay(&self, ui: &mut egui::Ui) {
        ui.label(format!("Active textures: {}", self.texture_cache.len()));
        ui.label(format!("Texture counter: {}", self.texture_id_counter));
        ui.label(format!("Elements in current frame: {}", self.elements_rendered_this_frame.len()));
        ui.label(format!("Elements in previous frame: {}", self.elements_rendered_last_frame.len()));
        
        // Show a few sample texture keys if available
        if !self.texture_keys.is_empty() {
            ui.label("Sample texture keys:");
            for (_i, (element_id, key)) in self.texture_keys.iter().take(3).enumerate() {
                ui.label(format!("  Element {}: Key {}", element_id, key));
            }
        }
    }
}