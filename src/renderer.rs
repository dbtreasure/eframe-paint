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
    // Flag to suppress selection drawing during resize/drag operations
    suppress_selection_drawing: bool,
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
            suppress_selection_drawing: false,
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
        
        // If no previews are active but suppression is still enabled, reset it
        // This ensures we don't get stuck in a state where selection boxes aren't drawn
        if self.drag_preview.is_none() && self.resize_preview.is_none() && self.preview_stroke.is_none() {
            self.suppress_selection_drawing = false;
        }
    }

    pub fn end_frame(&mut self, _ctx: &egui::Context) {
        // Nothing to do here - texture cleanup handled by TextureManager
    }

    /// Set a stroke preview for the renderer to display.
    /// This is typically used while drawing a new stroke before it's committed.
    ///
    /// @param points The points that make up the stroke path
    /// @param thickness The thickness of the stroke
    /// @param color The color of the stroke
    pub fn set_stroke_preview(&mut self, points: Vec<egui::Pos2>, thickness: f32, color: egui::Color32) {
        self.preview_stroke = Some(StrokePreview::new(points, thickness, color));
        
        // Request a repaint to ensure the preview is rendered immediately
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }
    
    /// Clear any active stroke preview.
    pub fn clear_stroke_preview(&mut self) {
        self.preview_stroke = None;
    }
    
    /// Set a resize preview rectangle for the renderer to display.
    /// This is typically used during element resize operations.
    ///
    /// @param rect Optional rectangle representing the resize preview, or None to clear
    pub fn set_resize_preview(&mut self, rect: Option<egui::Rect>) {
        self.resize_preview = rect;
        
        // Update selection drawing suppression based on preview state
        self.suppress_selection_drawing = rect.is_some();
        
        // Request a repaint to ensure the preview is rendered immediately
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }
    
    /// Get the current resize preview rectangle, if any.
    pub fn get_resize_preview(&self) -> Option<egui::Rect> {
        self.resize_preview
    }
    
    /// Set a drag preview rectangle for the renderer to display.
    /// This is typically used during element drag operations.
    ///
    /// @param rect Optional rectangle representing the drag preview, or None to clear
    pub fn set_drag_preview(&mut self, rect: Option<egui::Rect>) {
        self.drag_preview = rect;

        // Update selection drawing suppression based on preview state
        self.suppress_selection_drawing = rect.is_some();

        // Request a repaint to ensure the preview is rendered immediately
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }
    
    /// Set an active resize handle for the renderer to highlight.
    ///
    /// @param element_id The ID of the element being resized
    /// @param corner The corner that should be highlighted, or None to clear
    pub fn set_active_handle(&mut self, element_id: usize, corner: Option<Corner>) {
        if let Some(c) = corner {
            self.active_handles.insert(element_id, c);
        } else {
            self.active_handles.remove(&element_id);
        }
        
        // Request a repaint to ensure the handle highlight is rendered immediately
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }
    
    /// Check if an element has any active resize handles.
    ///
    /// @param element_id The ID of the element to check
    /// @return True if the element has any active handles
    pub fn is_handle_active(&self, element_id: usize) -> bool {
        self.active_handles.contains_key(&element_id)
    }
    
    /// Get the active handle for an element, if any.
    ///
    /// @param element_id The ID of the element to check
    /// @return The active corner handle, if any
    pub fn get_active_handle(&self, element_id: usize) -> Option<&Corner> {
        self.active_handles.get(&element_id)
    }
    
    /// Check if any elements have active resize handles.
    ///
    /// @return True if any elements have active handles
    pub fn any_handles_active(&self) -> bool {
        !self.active_handles.is_empty()
    }
    
    /// Clear all active resize handles.
    pub fn clear_active_handles(&mut self) {
        self.active_handles.clear();
    }
    
    /// Clear all preview visualizations at once.
    /// This is typically called after command execution or tool reset.
    pub fn clear_all_previews(&mut self) {
        self.preview_stroke = None;
        self.resize_preview = None;
        self.drag_preview = None;
        self.active_handles.clear();
        
        // Reset the suppress selection drawing flag
        self.suppress_selection_drawing = false;
        
        // Request a repaint to ensure the UI updates immediately
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }

    /// Draw any element through the TextureManager
    pub fn draw_element(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        element: &mut dyn Element,
        force_draw: bool,  // New parameter to force drawing even if already rendered
    ) {
        let element_id = element.id();
        let texture_version = element.texture_version();

        // Skip if we've already rendered this element this frame, unless force_draw is true
        if !force_draw && self.elements_rendered_this_frame.contains(&element_id) {
            return;
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

        // Only mark as rendered if not force_draw
        if !force_draw {
            self.elements_rendered_this_frame.insert(element_id);
        }
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

        Vec::new()
    }

    /// Render all active previews (stroke, resize, drag, handles)
    /// This is called by the main render method to display all preview visuals
    fn render_previews(&mut self, ui: &mut egui::Ui, panel_rect: egui::Rect) {
        // Render stroke preview if active
        if let Some(preview) = &self.preview_stroke {
            self.draw_stroke_preview(ui.painter(), preview);
        }
        
        // Only draw one type of preview at a time, prioritizing resize over drag
        if let Some(rect) = self.resize_preview {
            // Find the element being resized
            let mut active_element_id = None;
            for (element_id, _) in &self.active_handles {
                active_element_id = Some(*element_id);
                break;
            }
            
            // Draw the resize preview for this element
            if let Some(element_id) = active_element_id {
                if let Some(editor_model) = self.editor_model {
                    // Safety: We only dereference the pointer if it's valid
                    let editor_model = unsafe { &*editor_model };
                    self.draw_resize_preview(
                        ui.ctx(),
                        ui.painter(),
                        editor_model,
                        element_id,
                        rect,
                    );
                }
            }
        } else if let Some(rect) = self.drag_preview {
            // For drag preview, first draw the element texture at the preview position
            if let Some(editor_model) = self.editor_model {
                let editor_model = unsafe { &*editor_model };
                // Get the first selected element
                if let Some(element_id) = editor_model.selected_ids().iter().next() {
                    if let Some(mut element) = editor_model.get_element_by_id(*element_id).cloned() {
                        // Temporarily move the element to the preview position
                        // Use compute_element_rect to match exactly what the selection tool uses
                        let original_rect = crate::element::compute_element_rect(&element);
                        let offset = rect.min - original_rect.min;
                        element.translate(offset).ok();
                        
                        // Draw the element at the preview position
                        self.draw_element(ui.ctx(), ui.painter(), &mut element, true);
                    }
                }
            }
            
            // Draw a semi-transparent blue overlay
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_premultiplied(100, 150, 255, 80),
            );
            
            // Draw a visible outline
            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(30, 120, 255)),
            );
            
            // Draw handles at the corners for consistency with resize
            let handle_size = crate::element::RESIZE_HANDLE_RADIUS / 2.0;
            let corners = [
                (rect.left_top(), Corner::TopLeft),
                (rect.right_top(), Corner::TopRight),
                (rect.left_bottom(), Corner::BottomLeft),
                (rect.right_bottom(), Corner::BottomRight),
            ];
            
            for (pos, _corner) in corners {
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
        }
        
        // Only draw active handles if we're not showing any other preview
        if !self.suppress_selection_drawing && !self.active_handles.is_empty() {
            for (element_id, corner) in &self.active_handles {
                if let Some(element) = self.find_element(*element_id) {
                    let element_rect = crate::element::compute_element_rect(element);
                    let handle_size = crate::element::RESIZE_HANDLE_RADIUS;
                    
                    // Get the position of the corner
                    let pos = match corner {
                        Corner::TopLeft => element_rect.left_top(),
                        Corner::TopRight => element_rect.right_top(),
                        Corner::BottomLeft => element_rect.left_bottom(),
                        Corner::BottomRight => element_rect.right_bottom(),
                    };
                    
                    // Draw active handle with a highlight color
                    ui.painter().circle_filled(
                        pos,
                        handle_size,
                        egui::Color32::from_rgb(100, 200, 255), // Bright blue for active handle
                    );
                    ui.painter().circle_stroke(
                        pos,
                        handle_size,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                }
            }
        }
    }

    // Update the render method to call render_previews
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

        // Get the context for rendering
        let ctx = self.get_ctx().clone();

        // Check if we have any active previews
        let has_preview = self.resize_preview.is_some() || self.drag_preview.is_some();

        // Draw non-selected elements first
        for element_id in editor_model.all_element_ids() {
            if !selected_ids.contains(&element_id) {
                if let Some(element) = editor_model.get_element_mut_by_id(element_id) {
                    self.draw_element(&ctx, ui.painter(), element, false);
                }
            }
        }

        // Only draw selected elements and selection boxes if there's no preview active
        if !has_preview {
            // Draw selected elements
            for element_id in &selected_ids {
                if let Some(element) = editor_model.get_element_mut_by_id(*element_id) {
                    self.draw_element(&ctx, ui.painter(), element, true);
                }
            }

            // Draw selection boxes for selected elements
            for element_id in &selected_ids {
                if let Some(element) = editor_model.find_element_by_id(*element_id) {
                    self.draw_selection_box(ui, element);
                }
            }
        }

        // Render all previews (stroke, resize, drag, handles) on top
        self.render_previews(ui, rect);

        // Return resize info
        resize_info
    }

    /// Draw a preview of an element being resized
    fn draw_resize_preview(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        editor_model: &EditorModel,
        element_id: usize,
        preview_rect: egui::Rect,
    ) {
        // Get the element
        if let Some(element) = editor_model.get_element_by_id(element_id) {
            // Clone the element so we can modify it
            if let Some(mut cloned_element) = editor_model.get_element_by_id(element_id).cloned() {
                // We need to account for padding differences
                // The original element rect with padding
                let original_padded_rect = crate::element::compute_element_rect(element);
                // The original element rect without padding
                let original_raw_rect = element.rect();
                
                // Calculate the padding on each side
                let padding_left = original_raw_rect.min.x - original_padded_rect.min.x;
                let padding_top = original_raw_rect.min.y - original_padded_rect.min.y;
                let padding_right = original_padded_rect.max.x - original_raw_rect.max.x;
                let padding_bottom = original_padded_rect.max.y - original_raw_rect.max.y;
                
                // Create a preview rect that accounts for the padding
                // (subtract padding from the preview rect to get the raw rect for resize)
                let resize_rect = egui::Rect::from_min_max(
                    egui::pos2(
                        preview_rect.min.x + padding_left,
                        preview_rect.min.y + padding_top
                    ),
                    egui::pos2(
                        preview_rect.max.x - padding_right,
                        preview_rect.max.y - padding_bottom
                    )
                );
                
                // Resize the cloned element using the adjusted rect
                if let Err(err) = cloned_element.resize(resize_rect) {
                    log::error!("Failed to resize element for preview: {}", err);
                }
                
                // Draw the transformed element using the texture system
                self.draw_element(ctx, painter, &mut cloned_element, true);
            }
            
            // Draw the preview outline using the full padded rect
            painter.rect_stroke(
                preview_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(30, 120, 255)),
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
                painter.circle_filled(
                    pos,
                    handle_size,
                    egui::Color32::from_rgb(200, 200, 200),
                );
                painter.circle_stroke(
                    pos,
                    handle_size,
                    egui::Stroke::new(1.0, egui::Color32::BLACK),
                );
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
                        self.set_active_handle(element_id, Some(*corner));
                    }

                    // Always update active handle to the current corner being dragged
                    if self.is_handle_active(element_id) {
                        self.set_active_handle(element_id, Some(*corner));

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

                    self.set_active_handle(element_id, None);
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
        self.clear_all_previews();

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

    /// Get access to the editor model reference
    pub fn get_editor_model(&self) -> Option<*const EditorModel> {
        self.editor_model
    }
}
