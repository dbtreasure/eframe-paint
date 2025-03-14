// src/renderer.rs
use eframe::egui;
use crate::stroke::{Stroke, StrokeRef};
use crate::image::ImageRef;
use crate::element::{ElementType, Element};
use crate::widgets::{ResizeHandle, Corner};
use crate::state::EditorModel;
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
        
        // Request a repaint to ensure the drag preview is rendered immediately
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
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
            return;
        }
        
        // Check if we're currently dragging anything, and if this stroke is part of the selection
        // Don't render original strokes during drag operations
        if self.drag_preview.is_some() {
            // Skip drawing strokes if they are potentially being dragged
            if let Some(ctx) = &self.ctx {
                // This is a more aggressive approach to prevent duplicate rendering during drag
                if ctx.input(|i| i.pointer.primary_down()) {
                    info!("🚫 Skipping stroke render during drag: {}", stroke.id());
                    self.elements_rendered_this_frame.insert(stroke.id());
                    return;
                }
            }
        }

        for points in points.windows(2) {
            painter.line_segment(
                [points[0], points[1]],
                egui::Stroke::new(stroke.thickness(), stroke.color()),
            );
        }
        
        // Mark this stroke as rendered in this frame
        self.elements_rendered_this_frame.insert(stroke.id());
    }

    // Enhanced draw_image method with logging
    pub fn draw_image(&mut self, ctx: &egui::Context, painter: &egui::Painter, image: &ImageRef) {
        let size = image.size();
        let pos = image.position();
        
        // Create the rect for the image
        let rect = egui::Rect::from_min_size(pos, size);
        
        // Get or create the texture
        let texture_id = self.get_or_create_texture(ctx, image);
        
        // Draw the image
        painter.image(
            texture_id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    fn draw_selection_box(&self, ui: &mut egui::Ui, element: &ElementType) -> Vec<egui::Response> {
        // Get the element's bounding rectangle using compute_element_rect
        let rect = crate::element::compute_element_rect(element);
        
        // Draw the selection box with a more visible stroke
        ui.painter().rect_stroke(
            rect,
            0.0, // no rounding
            egui::Stroke::new(2.0, egui::Color32::from_rgb(30, 120, 255)), // Thicker, brighter blue
        );
        
        // Draw the resize handles at each corner
        let handle_size = crate::element::RESIZE_HANDLE_RADIUS / 2.0;
        
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
        ui: &mut egui::Ui,
        editor_model: &EditorModel,
        rect: egui::Rect,
    ) -> Option<(usize, Corner, egui::Pos2)> {
        // Get the selected elements from the editor_model
        let selected_elements: Vec<&ElementType> = editor_model.selected_ids()
            .iter()
            .filter_map(|id| editor_model.get_element_by_id(*id))
            .collect();

        // Process interactions first before drawing
        let resize_info = self.process_resize_interactions(ui, &selected_elements);
        
        // Draw background
        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
        
        // Collection to track which elements we've already drawn
        // This prevents double rendering of the same element
        let mut drawn_element_ids = std::collections::HashSet::new();
        
        // Check if we have active resize or drag previews
        let any_resize_active = self.resize_preview.is_some();
        let any_drag_active = self.drag_preview.is_some();
        
        // For drag operations, track the elements being dragged to avoid duplicates
        let dragged_element_ids: Vec<usize> = if any_drag_active {
            selected_elements.iter().map(|e| e.get_stable_id()).collect()
        } else {
            Vec::new()
        };
        
        // Create a copy of selected_elements for later use
        let selected_elements_copy: Vec<&ElementType> = selected_elements.clone();
        
        // Draw selected elements with drag/resize preview if applicable
        for element in selected_elements {
            let element_id = element.get_stable_id();
            
            // Skip if we've already drawn this element
            if drawn_element_ids.contains(&element_id) {
                continue;
            }
            
            // Mark as drawn
            drawn_element_ids.insert(element_id);
            
            // Draw based on element type
            match element {
                ElementType::Stroke(stroke) => {
                    // For strokes with active resize preview
                    if any_resize_active && self.is_handle_active(element_id) {
                        // Get the preview rectangle
                        let preview_rect = self.resize_preview.unwrap();
                        
                        // Get the original stroke rectangle for scaling calculation
                        let original_rect = crate::element::compute_element_rect(element);
                        
                        // Create a temporary resized stroke for preview
                        let preview_stroke = crate::stroke::resize_stroke(stroke, original_rect, preview_rect);
                        
                        // First draw a highlight effect (halo)
                        if preview_stroke.points().len() >= 2 {
                            for points in preview_stroke.points().windows(2) {
                                ui.painter().line_segment(
                                    [points[0], points[1]],
                                    egui::Stroke::new(
                                        preview_stroke.thickness() + 4.0,
                                        egui::Color32::from_rgba_premultiplied(150, 200, 255, 80)
                                    ),
                                );
                            }
                        }
                        
                        // Draw the resized preview instead of original
                        self.draw_stroke(ui.painter(), &preview_stroke);
                    } 
                    // For strokes with active drag preview
                    else if any_drag_active {
                        // Get the drag preview rectangle
                        let preview_rect = self.drag_preview.unwrap();
                        
                        // Calculate the drag delta
                        let original_rect = crate::element::compute_element_rect(element);
                        let delta = preview_rect.min - original_rect.min;
                        
                        // Create a temporary translated stroke for preview
                        let translated_points = stroke.points().iter()
                            .map(|p| *p + delta)
                            .collect::<Vec<_>>();
                        
                        // Draw the translated stroke
                        let points = &translated_points;
                        if points.len() >= 2 {
                            // First draw highlight
                            for points in points.windows(2) {
                                ui.painter().line_segment(
                                    [points[0], points[1]],
                                    egui::Stroke::new(
                                        stroke.thickness() + 4.0,
                                        egui::Color32::from_rgba_premultiplied(150, 200, 255, 80)
                                    ),
                                );
                            }
                            
                            // Then draw stroke
                            for points in points.windows(2) {
                                ui.painter().line_segment(
                                    [points[0], points[1]],
                                    egui::Stroke::new(stroke.thickness(), stroke.color()),
                                );
                            }
                        }
                        
                        // Mark this stroke as rendered to prevent drawing it again in its original position
                        drawn_element_ids.insert(element_id);
                        self.elements_rendered_this_frame.insert(element_id);
                    } else {
                        // Draw normally
                        info!("Drawing stroke normally for ID: {}", element_id);
                        self.draw_stroke(ui.painter(), stroke);
                    }
                },
                ElementType::Image(image) => {
                    // For images with active resize preview
                    if any_resize_active && self.is_handle_active(element_id) {
                        // Get the preview rectangle
                        let preview_rect = self.resize_preview.unwrap();
                        
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
                        let texture = self.get_ctx().load_texture(
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
                        
                        // Draw preview border with a semi-transparent overlay
                        ui.painter().rect_stroke(
                            preview_rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(100, 100, 255, 180)),
                        );
                    } 
                    // For images with active drag preview
                    else if any_drag_active {
                        
                        // Get the preview rectangle
                        let preview_rect = self.drag_preview.unwrap();
                        
                        // Create texture for the dragged image
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [image.size().x as usize, image.size().y as usize],
                            image.data(),
                        );
                        
                        // Create a unique texture name for this drag preview
                        let unique_drag_name = format!("drag_preview_{}", self.frame_counter);
                        
                        // Load the texture with the unique name
                        let texture = self.get_ctx().load_texture(
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
                        
                        // Draw preview border with a more visible style
                        ui.painter().rect_stroke(
                            preview_rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(100, 100, 255, 180)),
                        );
                        
                        // Add a semi-transparent overlay to make it clear this is a preview
                        ui.painter().rect_stroke(
                            preview_rect.expand(3.0),
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(30, 255, 120, 180)),
                        );
                    } else {
                        // Store the context before calling draw_image
                        let ctx_copy = self.get_ctx().clone();
                        self.draw_image(&ctx_copy, ui.painter(), image);
                    }
                }
            }
        }
                 
        // First draw all images (to ensure they're at the back)
        // First handle rendering of normal (non-selected) images
        for image in editor_model.images() {
            let image_id = image.id();
            let is_selected = selected_elements_copy.iter().any(|e| e.id() == image_id);
            
            // Skip selected images - we'll handle those separately
            if !is_selected && !drawn_element_ids.contains(&image_id) {
                // Store the context before calling draw_image
                let ctx_copy = self.get_ctx().clone();
                self.draw_image(&ctx_copy, ui.painter(), image);
                drawn_element_ids.insert(image_id);
            }
        }
        
        // Draw all non-selected strokes
        for stroke in editor_model.strokes() {
            let stroke_id = stroke.id();
            let is_selected = selected_elements_copy.iter().any(|e| e.id() == stroke_id);
            let is_being_dragged = dragged_element_ids.contains(&stroke_id);
            
            // Skip strokes that are selected or already drawn
            if (is_selected || drawn_element_ids.contains(&stroke_id)) && !is_being_dragged {
                continue;
            }
            
            // Skip strokes being dragged
            if is_being_dragged {
                info!("Skipping original stroke during drag: {}", stroke_id);
                drawn_element_ids.insert(stroke_id);
                self.elements_rendered_this_frame.insert(stroke_id);
                continue;
            }
            
            // Draw normally
            info!("Drawing non-selected stroke: {}", stroke_id);
            self.draw_stroke(ui.painter(), stroke);
            drawn_element_ids.insert(stroke_id);
        }
        
        // Then handle rendering of selected images with special handling for preview
        for image in editor_model.images() {
            let image_id = image.id();
            let is_selected = selected_elements_copy.iter().any(|e| e.id() == image_id);
            
            // Skip if already drawn or not selected
            if !is_selected || drawn_element_ids.contains(&image_id) {
                continue;
            }
            
            // Images are already specifically handled above in the selected elements section,
            // but we mark them as drawn here to avoid duplicate rendering
            drawn_element_ids.insert(image_id);
        }

        // Draw preview stroke if any
        if let Some(preview) = &self.preview_stroke {
            // Need to handle preview stroke differently to avoid borrow issues
            let points = preview.points();
            if points.len() >= 2 {
                
                for points in points.windows(2) {
                    ui.painter().line_segment(
                        [points[0], points[1]],
                        egui::Stroke::new(preview.thickness(), preview.color()),
                    );
                }
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
            
            // Add a semi-transparent fill to make the resize preview more visible
            ui.painter().rect_filled(
                preview_rect,
                0.0, 
                egui::Color32::from_rgba_premultiplied(100, 150, 255, 20),
            );
            
            // Draw resize handles at preview rect corners
            let handle_size = crate::element::RESIZE_HANDLE_RADIUS / 2.0;
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
            
            // Add a semi-transparent fill to make the drag preview more visible
            ui.painter().rect_filled(
                preview_rect,
                0.0,
                egui::Color32::from_rgba_premultiplied(100, 150, 255, 40),
            );
            
            // Draw lighter outline to show it's being dragged
            ui.painter().rect_stroke(
                preview_rect.expand(2.0),
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(30, 255, 120, 180)),
            );
            
            // Draw the image content at the drag preview position
            // Find the selected image
            for element in &selected_elements_copy {
                if let ElementType::Image(image) = element {
                    // Create texture for the dragged image
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [image.size().x as usize, image.size().y as usize],
                        image.data(),
                    );
                    
                    // Create a unique texture name for this drag preview
                    let unique_drag_name = format!("drag_preview_box_{}", self.frame_counter);
                    
                    // Load the texture with the unique name
                    let texture = self.get_ctx().load_texture(
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
                    break; // Only draw the first selected image
                }
            }
        } else {
            // If no preview is active, draw normal selection boxes
            for element in selected_elements_copy {
                self.draw_selection_box(ui, element);
            }
        }
        
        resize_info
    }

    pub fn process_resize_interactions(
        &mut self,
        ui: &mut egui::Ui,
        selected_elements: &[&ElementType],
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
            let rect = crate::element::compute_element_rect(element);
            
            // Skip processing if element has zero size
            if rect.width() < 1.0 || rect.height() < 1.0 {
                continue;
            }
            
            // Process each corner
            for corner in &[Corner::TopLeft, Corner::TopRight, Corner::BottomLeft, Corner::BottomRight] {
                
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

                    // If this is a new drag (no active handles yet), set this as the active handle
                    if !self.is_handle_active(element_id) {
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
                    }
                }
                
                // Handle drag release - clear active handle for this element
                if response.drag_stopped() {

                    // Get the final resize preview rect
                    if let Some(final_rect) = self.resize_preview {
                        
                        // Return the resize info so the selection tool can update the element
                        resize_info = Some((element_id, *corner, response.hover_pos().unwrap_or(response.interact_pointer_pos().unwrap())));
                    }
                    
                    self.clear_active_handle(element_id);
                }
            }
        }
        
        // If no resize is in progress, clear all previews
        if resize_info.is_none() {
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

    // Enhanced method to clear the renderer's state for a specific element
    pub fn clear_element_state(&mut self, element_id: usize) {
        
        // Check if this element has active handles before removing them
        let had_active_handles = self.active_handles.contains_key(&element_id);
        
        // Remove any active handles for this element
        self.active_handles.remove(&element_id);
        
        // Clear resize preview if this element had active handles
        // This logic was broken - it was checking AFTER removing the handle!
        if had_active_handles {
            self.resize_preview = None;
        }
        
        // Always clear drag preview to be safe
        self.drag_preview = None;
    }
    
    // A method to clear all element-related state (not preview strokes)
    pub fn clear_all_element_state(&mut self) {
        // Clear all state except preview strokes and drag preview
        self.active_handles.clear();
        self.resize_preview = None;
        // Don't clear drag preview here
        // self.drag_preview = None;
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
    
    // Method specifically for clearing textures for an element
    pub fn clear_texture_for_element(&mut self, element_id: usize) {
        // For now, just delegate to clear_element_state
        // In the future, this could be more specific to textures only
        self.clear_element_state(element_id);
        
        // Request a repaint to ensure changes are visible
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }
    
    // Add a debug visualization for texture state
    pub fn draw_debug_overlay(&self, ui: &mut egui::Ui) {
        ui.label(format!("Frame counter: {}", self.frame_counter));
    }

    // Helper method to get or create a texture for an image
    fn get_or_create_texture(&mut self, ctx: &egui::Context, image: &ImageRef) -> egui::TextureId {
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
            egui::ColorImage::new([width, height], egui::Color32::RED)
        };
        
        // Always increment frame counter to ensure unique texture names
        self.frame_counter += 1;
        
        // Create a unique texture name based on image ID and frame counter
        let unique_texture_name = format!("img_{}_{}", image.id(), self.frame_counter);
        
        // Load the texture - egui will automatically free it at the end of the frame
        let texture = ctx.load_texture(
            unique_texture_name,
            color_image,
            egui::TextureOptions::default(),
        );
        
        // Mark this image as rendered in this frame
        self.elements_rendered_this_frame.insert(image.id());
        
        texture.id()
    }
}