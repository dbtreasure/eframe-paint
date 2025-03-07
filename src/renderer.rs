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
    // Frame counter for debugging and unique texture names
    frame_counter: u64,
    // Track elements rendered this frame to prevent duplicates
    elements_rendered_this_frame: std::collections::HashSet<usize>,
    // Store a reference to the egui context for repaint requests
    ctx: Option<egui::Context>,
}

impl Renderer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone();
        let ctx = cc.egui_ctx.clone();
        
        Self {
            _gl: gl,
            preview_stroke: None,
            active_handles: HashMap::new(),
            resize_preview: None,
            drag_preview: None,
            frame_counter: 0,
            elements_rendered_this_frame: std::collections::HashSet::new(),
            ctx: Some(ctx),
        }
    }
    
    // Get a reference to the stored context
    pub fn get_ctx(&self) -> &egui::Context {
        self.ctx.as_ref().expect("Context should be initialized")
    }
    
    pub fn begin_frame(&mut self) {
        // Increment frame counter
        self.frame_counter += 1;
        
        // Clear element tracking for this frame
        self.elements_rendered_this_frame.clear();
        
        info!("Begin frame {}", self.frame_counter);
    }
    
    pub fn end_frame(&mut self, _ctx: &egui::Context) {
        // Nothing to do here anymore - egui handles texture cleanup automatically
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

    fn draw_stroke(&mut self, painter: &egui::Painter, stroke: &Stroke) {
        let points = stroke.points();
        if points.len() < 2 {
            info!("⚠️ Stroke {} has less than 2 points, skipping", stroke.id());
            return;
        }

        info!("Drawing stroke with ID: {}, thickness: {}, color: {:?}, {} points", 
             stroke.id(), stroke.thickness(), stroke.color(), points.len());

        for points in points.windows(2) {
            painter.line_segment(
                [points[0], points[1]],
                egui::Stroke::new(stroke.thickness(), stroke.color()),
            );
        }
        
        // Mark this stroke as rendered in this frame
        self.elements_rendered_this_frame.insert(stroke.id());
        
        info!("✅ Stroke {} successfully drawn with {} segments", 
             stroke.id(), points.len() - 1);
    }

    // Enhanced draw_image method with logging
    fn draw_image(&mut self, ctx: &egui::Context, painter: &egui::Painter, image: &Image) {
        // Log image drawing
        info!("Drawing image with ID: {}, size: {:?}, position: {:?}", 
             image.id(), image.size(), image.position());
        
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
        } else {
            // Fallback to a red placeholder if data is invalid
            info!("⚠️ Invalid image data: expected {} bytes, got {}", width * height * 4, data.len());
            egui::ColorImage::new([width, height], egui::Color32::RED)
        };
        
        // Always increment frame counter to ensure unique texture names
        self.frame_counter += 1;
        
        // Create a unique texture name based on image ID and frame counter
        let unique_texture_name = format!("img_{}_{}", image.id(), self.frame_counter);
        
        info!("Creating texture with name: {}", unique_texture_name);
        
        // Load the texture - egui will automatically free it at the end of the frame
        let texture = ctx.load_texture(
            unique_texture_name,
            color_image,
            egui::TextureOptions::default(),
        );
        
        // Draw the image
        let image_rect = image.rect();
        painter.image(
            texture.id(),
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        
        // Mark this image as rendered in this frame
        self.elements_rendered_this_frame.insert(image.id());
        
        info!("✅ Image {} successfully drawn at rect: {:?}", image.id(), image_rect);
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
        // Log document contents for debugging
        info!("🖌️ Rendering document with {} strokes and {} images", 
             document.strokes().len(), document.images().len());
        
        // Log stroke IDs for debugging
        if !document.strokes().is_empty() {
            let stroke_ids: Vec<_> = document.strokes().iter().map(|s| s.id()).collect();
            info!("🖌️ Stroke IDs to render: {:?}", stroke_ids);
        }
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
                            egui::ColorImage::new([width, height], egui::Color32::RED)
                        };
                        
                        // Create a unique texture name for this preview
                        let unique_preview_name = format!("preview_{}_{}", 
                                                         image.id(), 
                                                         self.frame_counter);
                        
                        // Load the texture with the unique name
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
                        
                        // Create a unique texture name for this drag preview
                        let unique_drag_name = format!("drag_{}_{}", 
                                                      image.id(), 
                                                      self.frame_counter);
                        
                        // Load the texture with the unique name
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
        
        // Log how many strokes and images are in the document
        info!("📑 Document contains {} strokes and {} images", 
             document.strokes().len(), document.images().len());
             
        // Log stroke IDs for debugging
        if !document.strokes().is_empty() {
            let stroke_ids: Vec<_> = document.strokes().iter().map(|s| s.id()).collect();
            info!("🔢 Stroke IDs to render: {:?}", stroke_ids);
        }
            
        // Reset drawn elements tracking
        drawn_element_ids.clear();
        
        // First draw all images (to ensure they're at the back)
        for image in document.images() {
            let image_id = image.id();
            
            info!("🖼️ Drawing image with ID: {}", image_id);
            self.draw_image(ctx, ui.painter(), image);
            drawn_element_ids.insert(image_id);
        }
        
        // Then draw strokes (on top of images)
        for stroke in document.strokes() {
            let stroke_id = stroke.id();
            
            info!("✏️ Drawing stroke with ID: {}", stroke_id);
            self.draw_stroke(ui.painter(), stroke);
            drawn_element_ids.insert(stroke_id);
        }
        
        // Log what was actually drawn
        info!("🎨 Drew {} elements this frame", drawn_element_ids.len());
        
        // Draw preview stroke if any
        if let Some(preview) = &self.preview_stroke {
            // Need to handle preview stroke differently to avoid borrow issues
            let points = preview.points();
            if points.len() >= 2 {
                info!("Drawing preview stroke with {} points", points.len());
                
                for points in points.windows(2) {
                    ui.painter().line_segment(
                        [points[0], points[1]],
                        egui::Stroke::new(preview.thickness(), preview.color()),
                    );
                }
                
                info!("✅ Preview stroke successfully drawn");
            }
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
        info!("Clearing element state for element ID: {}", element_id);
        
        // Check if this element has active handles before removing them
        let had_active_handles = self.active_handles.contains_key(&element_id);
        
        // Remove any active handles for this element
        self.active_handles.remove(&element_id);
        
        // Clear resize preview if this element had active handles
        // This logic was broken - it was checking AFTER removing the handle!
        if had_active_handles {
            info!("Clearing resize preview because element {} had active handles", element_id);
            self.resize_preview = None;
        }
        
        // Always clear drag preview to be safe
        self.drag_preview = None;
    }
    
    // A method to clear all element-related state (not preview strokes)
    pub fn clear_all_element_state(&mut self) {
        // Clear all state except preview strokes
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
        
        // Reset frame counter
        self.frame_counter = 0;
    }
    
    // Add a method to handle element updates
    pub fn handle_element_update(&mut self, element: &ElementType) {
        // Use the get_stable_id method which is public
        self.clear_element_state(element.get_stable_id());
    }
    
    // Add a debug visualization for texture state
    pub fn draw_debug_overlay(&self, ui: &mut egui::Ui) {
        ui.label(format!("Frame counter: {}", self.frame_counter));
    }
}