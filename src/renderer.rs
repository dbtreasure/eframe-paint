// src/renderer.rs
use eframe::egui;
use crate::stroke::{Stroke, StrokeRef};
use crate::document::Document;
use crate::image::Image;
use crate::state::ElementType;
use crate::widgets::{ResizeHandle, Corner};
use std::collections::HashMap;
use log::info;

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
        info!("Setting active handle for element {}", element_id);
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
        // Get the element's bounding rectangle
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
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
        ];
        
        for pos in corners {
            // Use the simplified drawing method
            ResizeHandle::draw_simple_handle(ui, pos, handle_size);
        }
        
        // We don't need to return responses here anymore since we handle them in process_resize_interactions
        Vec::new()
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
        // Process interactions first before drawing
        let resize_info = self.process_resize_interactions(ui, selected_elements);
        
        // Draw background
        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
        
        // Draw document contents
        for stroke in document.strokes() {
            self.draw_stroke(ui.painter(), stroke);
        }
        
        for image in document.images() {
            self.draw_image(ctx, ui.painter(), image);
        }
        
        // Draw preview stroke if any
        if let Some(preview) = &self.preview_stroke {
            self.draw_stroke(ui.painter(), preview);
        }
        
        // Draw selection boxes for selected elements
        for element in selected_elements {
            self.draw_selection_box(ui, element);
        }
        
        resize_info
    }

    fn process_resize_interactions(
        &mut self,
        ui: &mut egui::Ui,
        selected_elements: &[ElementType],
    ) -> Option<(usize, Corner, egui::Pos2)> {
        let mut resize_info = None;

        for element in selected_elements {
            // Get element ID first
            let element_id = match element {
                ElementType::Stroke(s) => std::sync::Arc::as_ptr(s) as usize,
                ElementType::Image(i) => i.id(),
            };
            
            // Get the element's bounding rectangle
            let rect = crate::geometry::hit_testing::compute_element_rect(element);
            info!("Image bounding box: [{:?} - {:?}]", rect.min, rect.max);
            
            // Create and show resize handles at each corner
            let handle_size = crate::geometry::hit_testing::RESIZE_HANDLE_RADIUS;
            
            let corners = [
                (rect.left_top(), Corner::TopLeft),
                (rect.right_top(), Corner::TopRight),
                (rect.left_bottom(), Corner::BottomLeft),
                (rect.right_bottom(), Corner::BottomRight),
            ];
            
            // Process each corner's handle
            for (pos, corner) in corners {
                // Create the handle for interaction (now includes drawing)
                let handle = crate::widgets::ResizeHandle::new(
                    element_id,
                    corner,
                    pos,
                    handle_size, // Use the same size as the visual
                );
                
                // Get the response for interaction (now includes drawing)
                let response = handle.show(ui);
                
                if response.dragged() {
                    info!("Dragging handle for element {}", element_id);
                    self.set_active_handle(element_id, corner);
                    resize_info = Some((element_id, corner, response.interact_pointer_pos().unwrap()));
                }
                
                if response.clicked() {
                    info!("Clicked handle for element {}", element_id);
                    self.set_active_handle(element_id, corner);
                }
                
                if response.drag_stopped() {
                    info!("Drag stopped for element {}", element_id);
                    self.clear_active_handle(element_id);
                }
            }
        }
        
        resize_info
    }
}