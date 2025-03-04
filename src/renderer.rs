// src/renderer.rs
use eframe::egui;
use crate::stroke::{Stroke, StrokeRef};
use crate::document::Document;
use crate::image::Image;
use crate::state::ElementType;
use crate::widgets::{ResizeHandle, Corner};
use std::collections::HashMap;

pub struct Renderer {
    _gl: Option<std::sync::Arc<eframe::glow::Context>>,
    preview_stroke: Option<StrokeRef>,
    // Track active resize handles
    active_handles: HashMap<usize, Corner>,
}

impl Renderer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone();
        
        Self {
            _gl: gl,
            preview_stroke: None,
            active_handles: HashMap::new(),
        }
    }

    pub fn set_preview_stroke(&mut self, stroke: Option<StrokeRef>) {
        self.preview_stroke = stroke;
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
        // Use the image's unique ID for caching instead of memory address
        let image_id = image.id();
        
        // Create a new texture from the image data every time
        let width = image.size().x as usize;
        let height = image.size().y as usize;
        
        // Create the color image from RGBA data
        let color_image = if image.data().len() == width * height * 4 {
            // Data is already in RGBA format
            egui::ColorImage::from_rgba_unmultiplied(
                [width, height],
                image.data(),
            )
        } else {
            // If data is not in the expected format, create a placeholder
            egui::ColorImage::new([width, height], egui::Color32::RED)
        };
        
        // Load the texture (this will be automatically freed at the end of the frame)
        let texture = ctx.load_texture(
            format!("image_{}", image_id),
            color_image,
            egui::TextureOptions::default(),
        );
        
        let texture_id = texture.id();
        
        // Draw the image at its position with its size
        let rect = image.rect();
        
        // Use the full texture (UV coordinates from 0,0 to 1,1)
        let uv = egui::Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 1.0)
        );
        
        painter.image(texture_id, rect, uv, egui::Color32::WHITE);
    }

    // Replace the old draw_corner_button with our new resize handle widget
    fn draw_resize_handle(&self, ui: &mut egui::Ui, element_id: usize, corner: Corner, pos: egui::Pos2) -> egui::Response {
        // Create a resize handle with a size of 15.0 (matching the old handle size)
        let handle = ResizeHandle::new(element_id, corner, pos, 15.0);
        
        // Show the handle and return the response
        handle.show(ui)
    }

    // Update the draw_selection_box method to use our new resize handle widget
    fn draw_selection_box(&self, ui: &mut egui::Ui, element: &ElementType) -> Vec<egui::Response> {
        let rect = match element {
            ElementType::Stroke(stroke_ref) => {
                // For strokes, calculate bounding box from points
                let points = stroke_ref.points();
                if points.is_empty() {
                    return Vec::new();
                }
                
                // Find min and max coordinates to create bounding box
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                
                for point in points {
                    min_x = min_x.min(point.x);
                    min_y = min_y.min(point.y);
                    max_x = max_x.max(point.x);
                    max_y = max_y.max(point.y);
                }
                
                // Add padding based on stroke thickness
                let padding = stroke_ref.thickness() + 2.0;
                min_x -= padding;
                min_y -= padding;
                max_x += padding;
                max_y += padding;
                
                egui::Rect::from_min_max(
                    egui::pos2(min_x, min_y),
                    egui::pos2(max_x, max_y),
                )
            },
            ElementType::Image(image_ref) => {
                // For images, use the image's rect with some padding
                let rect = image_ref.rect();
                let padding = 2.0;
                egui::Rect::from_min_max(
                    egui::pos2(rect.min.x - padding, rect.min.y - padding),
                    egui::pos2(rect.max.x + padding, rect.max.y + padding),
                )
            }
        };
        
        // Draw red selection box
        ui.painter().rect_stroke(
            rect,
            0.0, // no rounding
            egui::Stroke::new(2.0, egui::Color32::RED),
        );
        
        // Get the element ID
        let element_id = match element {
            ElementType::Stroke(s) => {
                // For strokes, we don't have an ID, so use the Arc pointer value
                std::sync::Arc::as_ptr(s) as usize
            },
            ElementType::Image(i) => i.id(),
        };
        
        // Draw corner handles and collect responses
        let mut responses = Vec::new();
        
        // Define corners with their positions
        let corners = [
            (Corner::TopLeft, rect.left_top()),
            (Corner::TopRight, rect.right_top()),
            (Corner::BottomLeft, rect.left_bottom()),
            (Corner::BottomRight, rect.right_bottom()),
        ];
        
        // Draw each handle and collect responses
        for (corner, pos) in corners {
            let response = self.draw_resize_handle(ui, element_id, corner, pos);
            responses.push(response);
        }
        
        responses
    }

    // Update the render method to handle resize handle interactions
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        document: &Document,
        selected_elements: &[ElementType],
    ) -> Option<(usize, Corner, egui::Pos2)> {
        // Draw background
        ui.painter().rect_filled(
            rect,
            0.0,
            egui::Color32::WHITE,
        );

        // Draw all images in the document
        for (_i, image_ref) in document.images().iter().enumerate() {
            self.draw_image(ctx, ui.painter(), image_ref);
        }

        // Draw all strokes in the document
        for (_i, stroke_ref) in document.strokes().iter().enumerate() {
            self.draw_stroke(ui.painter(), stroke_ref);
        }

        // Draw the preview stroke if there is one
        if let Some(preview) = &self.preview_stroke {
            self.draw_stroke(ui.painter(), preview);
        }
        
        // Track if any handle was interacted with
        let mut resize_info = None;
        
        // Draw selection boxes and handles for selected elements
        for element in selected_elements {
            let responses = self.draw_selection_box(ui, element);
            
            // Get the element ID
            let element_id = match element {
                ElementType::Stroke(s) => {
                    // For strokes, we don't have an ID, so use the Arc pointer value
                    std::sync::Arc::as_ptr(s) as usize
                },
                ElementType::Image(i) => i.id(),
            };
            
            // Check for handle interactions
            for (i, response) in responses.iter().enumerate() {
                let corner = match i {
                    0 => Corner::TopLeft,
                    1 => Corner::TopRight,
                    2 => Corner::BottomLeft,
                    3 => Corner::BottomRight,
                    _ => continue,
                };
                
                // If a handle is being dragged, update the active handle and return resize info
                if response.dragged() {
                    self.set_active_handle(element_id, corner);
                    resize_info = Some((element_id, corner, response.interact_pointer_pos().unwrap()));
                }
                
                // If a handle was clicked, set it as active
                if response.clicked() {
                    self.set_active_handle(element_id, corner);
                }
                
                // If a handle was released, clear it
                if response.drag_stopped() {
                    self.clear_active_handle(element_id);
                }
            }
        }
        
        resize_info
    }

    /// Update any cached state based on the current editor state
    pub fn update_state_snapshot(&mut self, _state: &crate::state::EditorState) {
        // This method is called when the editor state version changes
        // Currently, we don't need to cache anything specific from the state,
        // but this is where we would update any renderer-specific caches
        // based on the editor state
    }
}