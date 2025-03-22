// src/renderer.rs
use crate::element::{Element, ElementType};
use crate::state::EditorModel;
use crate::texture_manager::TextureManager;
use crate::widgets::{Corner, ResizeHandle};
use eframe::egui;
use log::info;
use std::collections::HashMap;

/// Represents a stroke being previewed as it's drawn
pub struct StrokePreview {
    points: Vec<egui::Pos2>,
    thickness: f32,
    color: egui::Color32,
}

impl StrokePreview {
    pub fn new(points: Vec<egui::Pos2>, thickness: f32, color: egui::Color32) -> Self {
        Self {
            points,
            thickness,
            color,
        }
    }

    pub fn points(&self) -> &[egui::Pos2] {
        &self.points
    }

    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    pub fn color(&self) -> egui::Color32 {
        self.color
    }
}

pub struct Renderer {
    _gl: Option<std::sync::Arc<eframe::glow::Context>>,
    preview_stroke: Option<StrokePreview>,
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
    // Texture manager for caching element textures
    texture_manager: TextureManager,
    // Reference to the editor model for finding elements
    editor_model: Option<*const EditorModel>,
}

impl Renderer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone();
        let ctx = cc.egui_ctx.clone();

        // Initialize texture manager with a reasonable cache size
        let texture_manager = TextureManager::new(100);

        Self {
            _gl: gl,
            preview_stroke: None,
            active_handles: HashMap::new(),
            resize_preview: None,
            drag_preview: None,
            frame_counter: 0,
            elements_rendered_this_frame: std::collections::HashSet::new(),
            ctx: Some(ctx),
            texture_manager,
            editor_model: None,
        }
    }

    /// Set a reference to the editor model for element lookups
    pub fn set_editor_model_ref(&mut self, editor_model: &EditorModel) {
        // Store a raw pointer to the editor model for element lookups
        // Safety: We're just storing a pointer for lookups, and won't modify anything
        // The pointer will be valid as long as the editor model exists
        self.editor_model = Some(editor_model as *const EditorModel);
    }

    /// Find an element by ID in the editor model
    ///
    /// Returns None if no editor model is set or the element doesn't exist
    pub fn find_element(&self, element_id: usize) -> Option<&ElementType> {
        // Safety: We only dereference the pointer if it's valid
        // And we only read from it, never modify
        self.editor_model
            .and_then(|model| unsafe { (*model).find_element_by_id(element_id) })
    }

    // Get a reference to the stored context
    pub fn get_ctx(&self) -> &egui::Context {
        self.ctx.as_ref().expect("Context should be initialized")
    }

    pub fn begin_frame(&mut self) {
        // Increment frame counter
        self.frame_counter += 1;

        // Start a new frame in the texture manager
        self.texture_manager.begin_frame();

        // Clear element tracking for this frame
        self.elements_rendered_this_frame.clear();
    }

    pub fn end_frame(&mut self, _ctx: &egui::Context) {
        // Nothing to do here - texture cleanup handled by TextureManager
    }

    pub fn set_preview_stroke(&mut self, stroke: Option<StrokePreview>) {
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

    /// Draw any element through the TextureManager
    pub fn draw_element(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        element: &mut dyn Element,
    ) {
        let element_id = element.id();
        let texture_version = element.texture_version();

        // Skip if we've already rendered this element this frame
        if self.elements_rendered_this_frame.contains(&element_id) {
            return;
        }

        // Check if we're currently dragging anything, and if this element is part of the selection
        // Don't render original elements during drag operations
        if self.drag_preview.is_some() {
            // Skip drawing if potentially being dragged
            if ctx.input(|i| i.pointer.primary_down()) {
                info!("🚫 Skipping element render during drag: {}", element_id);
                self.elements_rendered_this_frame.insert(element_id);
                return;
            }
        }

        // Get the element's rectangle
        let rect = element.rect();

        // Get or create a texture for this element
        match self.texture_manager.get_or_create_texture(
            element_id,
            texture_version,
            || element.generate_texture(ctx),
            ctx,
        ) {
            Ok(texture_id) => {
                // Draw the element as a textured rectangle
                painter.image(
                    texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            Err(_) => {
                // Fallback drawing if texture generation failed
                // Draw a placeholder rectangle
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(200));
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::RED));

                // Use direct drawing method if available
                element.draw(painter);
            }
        }

        // Mark as rendered in this frame
        self.elements_rendered_this_frame.insert(element_id);
    }

    /// Invalidate texture for an element
    pub fn invalidate_element_texture(&mut self, element_id: usize) {
        self.texture_manager.invalidate_element(element_id);
    }

    /// Draw a stroke preview (not from an Element)
    fn draw_stroke_preview(&self, painter: &egui::Painter, preview: &StrokePreview) {
        let points = preview.points();
        if points.len() < 2 {
            return;
        }

        for points in points.windows(2) {
            painter.line_segment(
                [points[0], points[1]],
                egui::Stroke::new(preview.thickness(), preview.color()),
            );
        }
    }

    fn draw_selection_box(&self, ui: &mut egui::Ui, element: &ElementType) -> Vec<egui::Response> {
        // Get the element's bounding rectangle using compute_element_rect
        let rect = crate::element::compute_element_rect(element);

        // Draw the selection box with a more visible stroke
        ui.painter().rect_stroke(
            rect,
            0.0,                                                           // no rounding
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
            ui.painter()
                .circle_filled(pos, handle_size, egui::Color32::from_rgb(200, 200, 200));

            ui.painter().circle_stroke(
                pos,
                handle_size,
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            );
        }

        // We don't need to return responses here anymore since we handle them in process_resize_interactions
        Vec::new()
    }

    // Update the render method to use the TextureManager
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        editor_model: &mut EditorModel,
        rect: egui::Rect,
    ) -> Option<(usize, Corner, egui::Pos2)> {
        // Update our reference to the editor model
        self.set_editor_model_ref(editor_model);
        // Get the selected elements from the editor_model
        let selected_ids: Vec<usize> = editor_model.selected_ids().iter().copied().collect();

        // Process interactions first before drawing
        let resize_info = self.process_resize_interactions_for_ids(ui, editor_model, &selected_ids);

        // Draw background
        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);

        // Collection to track which elements we've already drawn
        let mut drawn_element_ids = std::collections::HashSet::new();

        // Check if we have active resize or drag previews
        let any_resize_active = self.resize_preview.is_some();
        let any_drag_active = self.drag_preview.is_some();

        // Get the context for rendering
        let ctx = self.get_ctx().clone();

        // First render non-selected elements
        // Z-order: First images (background), then strokes (foreground)

        // Draw all non-selected elements
        for element_id in editor_model.all_element_ids() {
            // Skip selected elements
            if selected_ids.contains(&element_id) {
                continue;
            }

            // Get a mutable reference to the element
            if let Some(element) = editor_model.get_element_mut_by_id(element_id) {
                // Draw the element
                self.draw_element(&ctx, ui.painter(), element);
                drawn_element_ids.insert(element_id);
            }
        }

        // Draw selected elements with preview rendering if necessary
        for element_id in &selected_ids {
            // Skip if already drawn
            if drawn_element_ids.contains(element_id) {
                continue;
            }

            // For elements with active resize preview
            if any_resize_active && self.is_handle_active(*element_id) {
                // Get the preview rectangle
                let preview_rect = self.resize_preview.unwrap();

                // Draw the resize preview
                self.draw_resize_preview(
                    &ctx,
                    ui.painter(),
                    editor_model,
                    *element_id,
                    preview_rect,
                );
                drawn_element_ids.insert(*element_id);
            }
            // For elements with active drag preview
            else if any_drag_active {
                // Get the drag preview rectangle
                let preview_rect = self.drag_preview.unwrap();

                // Draw the drag preview
                self.draw_drag_preview(&ctx, ui.painter(), editor_model, *element_id, preview_rect);
                drawn_element_ids.insert(*element_id);
            } else {
                // Normal rendering for selected elements
                if let Some(element) = editor_model.get_element_mut_by_id(*element_id) {
                    self.draw_element(&ctx, ui.painter(), element);
                    drawn_element_ids.insert(*element_id);
                }
            }
        }

        // Draw preview stroke if any
        if let Some(preview) = &self.preview_stroke {
            self.draw_stroke_preview(ui.painter(), preview);
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
                    egui::Color32::from_rgb(200, 200, 200),
                );

                ui.painter().circle_stroke(
                    pos,
                    handle_size,
                    egui::Stroke::new(1.0, egui::Color32::BLACK),
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
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_premultiplied(30, 255, 120, 180),
                ),
            );

            // Draw the image content at the drag preview position
            // Find the selected image
            for element_id in &selected_ids {
                if let Some(element) = editor_model.get_element_by_id(*element_id) {
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
                        let uv =
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

                        // Draw preview at dragged position
                        ui.painter()
                            .image(texture.id(), preview_rect, uv, egui::Color32::WHITE);
                        break; // Only draw the first selected image
                    }
                }
            }
        } else {
            // If no preview is active, draw normal selection boxes
            for element_id in &selected_ids {
                if let Some(element) = editor_model.get_element_by_id(*element_id) {
                    self.draw_selection_box(ui, element);
                }
            }
        }

        resize_info
    }

    /// Draw a preview of an element being resized
    fn draw_resize_preview(
        &mut self,
        _ctx: &egui::Context,
        painter: &egui::Painter,
        editor_model: &EditorModel,
        element_id: usize,
        preview_rect: egui::Rect,
    ) {
        // Get the element
        if let Some(element) = editor_model.get_element_by_id(element_id) {
            match element {
                ElementType::Stroke(stroke) => {
                    // Get the original stroke rectangle for scaling calculation
                    let original_rect = element.rect();

                    // Create a temporary resized stroke for preview
                    let preview_stroke = StrokePreview::new(
                        stroke
                            .points()
                            .iter()
                            .map(|p| {
                                // Transform each point from original to preview rectangle
                                let rel_x = (p.x - original_rect.min.x) / original_rect.width();
                                let rel_y = (p.y - original_rect.min.y) / original_rect.height();
                                egui::pos2(
                                    preview_rect.min.x + rel_x * preview_rect.width(),
                                    preview_rect.min.y + rel_y * preview_rect.height(),
                                )
                            })
                            .collect(),
                        stroke.thickness() * (preview_rect.width() / original_rect.width()),
                        stroke.color(),
                    );

                    // Draw a highlight effect (halo)
                    if preview_stroke.points().len() >= 2 {
                        for points in preview_stroke.points().windows(2) {
                            painter.line_segment(
                                [points[0], points[1]],
                                egui::Stroke::new(
                                    preview_stroke.thickness() + 4.0,
                                    egui::Color32::from_rgba_premultiplied(150, 200, 255, 80),
                                ),
                            );
                        }
                    }

                    // Draw the stroke preview
                    self.draw_stroke_preview(painter, &preview_stroke);
                }
                ElementType::Image(_) => {
                    // For images, we just draw a rectangle at the preview position
                    // Ideally we would draw the actual image texture but resized

                    // Draw the preview rectangle with a semi-transparent overlay
                    painter.rect_filled(
                        preview_rect,
                        0.0,
                        egui::Color32::from_rgba_premultiplied(200, 200, 255, 180),
                    );

                    painter.rect_stroke(
                        preview_rect,
                        0.0,
                        egui::Stroke::new(
                            2.0,
                            egui::Color32::from_rgba_premultiplied(100, 100, 255, 200),
                        ),
                    );
                }
            }
        }
    }

    /// Draw a preview of an element being dragged
    fn draw_drag_preview(
        &mut self,
        _ctx: &egui::Context,
        painter: &egui::Painter,
        editor_model: &EditorModel,
        element_id: usize,
        preview_rect: egui::Rect,
    ) {
        // Get the element
        if let Some(element) = editor_model.get_element_by_id(element_id) {
            match element {
                ElementType::Stroke(stroke) => {
                    // Calculate the drag delta
                    let original_rect = element.rect();
                    let delta = preview_rect.min - original_rect.min;

                    // Create a temporary translated stroke for preview
                    let preview_stroke = StrokePreview::new(
                        stroke.points().iter().map(|p| *p + delta).collect(),
                        stroke.thickness(),
                        stroke.color(),
                    );

                    // Draw a highlight effect (halo)
                    if preview_stroke.points().len() >= 2 {
                        for points in preview_stroke.points().windows(2) {
                            painter.line_segment(
                                [points[0], points[1]],
                                egui::Stroke::new(
                                    preview_stroke.thickness() + 4.0,
                                    egui::Color32::from_rgba_premultiplied(150, 200, 255, 80),
                                ),
                            );
                        }
                    }

                    // Draw the stroke preview
                    self.draw_stroke_preview(painter, &preview_stroke);
                }
                ElementType::Image(_image) => {
                    // For images, we draw a semi-transparent rectangle at the drag position

                    // Draw the preview rectangle with a semi-transparent overlay
                    painter.rect_filled(
                        preview_rect,
                        0.0,
                        egui::Color32::from_rgba_premultiplied(200, 200, 255, 180),
                    );

                    painter.rect_stroke(
                        preview_rect,
                        0.0,
                        egui::Stroke::new(
                            2.0,
                            egui::Color32::from_rgba_premultiplied(100, 100, 255, 200),
                        ),
                    );

                    // Add a semi-transparent outline to indicate dragging
                    painter.rect_stroke(
                        preview_rect.expand(3.0),
                        0.0,
                        egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_premultiplied(30, 255, 120, 180),
                        ),
                    );
                }
            }
        }
    }

    pub fn process_resize_interactions_for_ids(
        &mut self,
        ui: &mut egui::Ui,
        editor_model: &EditorModel,
        selected_ids: &[usize],
    ) -> Option<(usize, Corner, egui::Pos2)> {
        // Convert IDs to elements
        let selected_elements: Vec<&ElementType> = selected_ids
            .iter()
            .filter_map(|id| editor_model.get_element_by_id(*id))
            .collect();

        self.process_resize_interactions(ui, &selected_elements)
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
            let element_id = element.id();

            // Get the element's rectangle
            let rect = crate::element::compute_element_rect(element);

            // Skip processing if element has zero size
            if rect.width() < 1.0 || rect.height() < 1.0 {
                continue;
            }

            // Process each corner
            for corner in &[
                Corner::TopLeft,
                Corner::TopRight,
                Corner::BottomLeft,
                Corner::BottomRight,
            ] {
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
                        let mouse_pos = response
                            .hover_pos()
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
                    if let Some(_final_rect) = self.resize_preview {
                        // Return the resize info so the selection tool can update the element
                        resize_info = Some((
                            element_id,
                            *corner,
                            response
                                .hover_pos()
                                .unwrap_or(response.interact_pointer_pos().unwrap()),
                        ));
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

    pub fn compute_resized_rect(
        original: egui::Rect,
        corner: Corner,
        new_pos: egui::Pos2,
    ) -> egui::Rect {
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
        if had_active_handles {
            self.resize_preview = None;
        }

        // Always clear drag preview to be safe
        self.drag_preview = None;

        // Invalidate texture for this element
        self.texture_manager.invalidate_element(element_id);
    }

    // A method to clear all element-related state (not preview strokes)
    pub fn clear_all_element_state(&mut self) {
        // Clear all state except preview strokes
        self.active_handles.clear();
        self.resize_preview = None;
    }

    // Enhanced method to reset all renderer state
    pub fn reset_state(&mut self) {
        self.active_handles.clear();
        self.resize_preview = None;
        self.drag_preview = None;
        self.preview_stroke = None;

        // Clear all textures
        self.texture_manager.clear_cache();

        // Reset frame counter
        self.frame_counter = 0;
    }

    // Add a method to handle element updates
    pub fn handle_element_update(&mut self, element: &ElementType) {
        // Use the element ID
        self.clear_element_state(element.id());
    }

    // Method specifically for clearing textures for an element
    pub fn invalidate_texture(&mut self, element_id: usize) {
        // Invalidate the texture in the texture manager
        self.texture_manager.invalidate_element(element_id);

        // Request a repaint to ensure changes are visible
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }

    // Add a debug visualization for texture state
    pub fn draw_debug_overlay(&self, ui: &mut egui::Ui) {
        ui.label(format!("Frame counter: {}", self.frame_counter));
        ui.label(format!(
            "Texture cache size: {}",
            self.texture_manager.cache_size()
        ));

        // Show the top 5 elements in the texture cache
        if self.texture_manager.cache_size() > 0 {
            ui.label("Recent texture updates:");

            // We can't actually see inside the texture manager's cache from here,
            // but in a real implementation, you could provide methods to inspect the cache
        }
    }
}
