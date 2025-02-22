use egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2, Painter, Ui};
use crate::layer::Transform;

const HANDLE_SIZE: f32 = 8.0;
const ROTATION_HANDLE_OFFSET: f32 = 30.0;
const HANDLE_COLOR: Color32 = Color32::from_rgb(30, 144, 255);
const HANDLE_HOVER_COLOR: Color32 = Color32::from_rgb(135, 206, 250);
const HANDLE_STROKE_WIDTH: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoHandle {
    Move,
    ScaleTopLeft,
    ScaleTopRight,
    ScaleBottomLeft,
    ScaleBottomRight,
    Rotate,
}

#[derive(Debug, Clone)]
pub struct TransformGizmo {
    bounds: Rect,
    active_handle: Option<GizmoHandle>,
    initial_transform: Transform,
    current_transform: Transform,
    initial_pointer_pos: Option<Pos2>,
    pub completed_transform: Option<(Transform, Transform)>,
    pub is_active: bool,
}

impl TransformGizmo {
    pub fn new(bounds: Rect, initial_transform: Transform) -> Self {
        Self {
            bounds,
            active_handle: None,
            initial_transform: initial_transform.clone(),
            current_transform: initial_transform,
            initial_pointer_pos: None,
            completed_transform: None,
            is_active: false,
        }
    }

    pub fn begin_transform(&mut self, handle: GizmoHandle, pointer_pos: Pos2) {
        self.active_handle = Some(handle);
        self.initial_pointer_pos = Some(pointer_pos);
        self.is_active = true;
    }

    pub fn end_transform(&mut self) {
        if self.is_active && self.current_transform != self.initial_transform {
            self.completed_transform = Some((
                self.initial_transform.clone(),
                self.current_transform.clone()
            ));
        }
        self.active_handle = None;
        self.initial_pointer_pos = None;
        self.is_active = false;
    }

    /// Updates the bounds of the gizmo to match the transformed shape
    pub fn update_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    pub fn update(
        &mut self,
        ui: &mut Ui,
        transform: &mut Transform,
    ) -> bool {
        let mut changed = false;

        // Handle positions
        let center = self.bounds.center();
        let top_left = self.bounds.left_top();
        let top_right = self.bounds.right_top();
        let bottom_left = self.bounds.left_bottom();
        let bottom_right = self.bounds.right_bottom();
        
        // Calculate rotation handle position at 12 o'clock
        let rotation_pos = Pos2::new(
            center.x,
            self.bounds.min.y - ROTATION_HANDLE_OFFSET,
        );

        // Handle positions with their types
        let handles = [
            (GizmoHandle::Move, center),
            (GizmoHandle::ScaleTopLeft, top_left),
            (GizmoHandle::ScaleTopRight, top_right),
            (GizmoHandle::ScaleBottomLeft, bottom_left),
            (GizmoHandle::ScaleBottomRight, bottom_right),
            (GizmoHandle::Rotate, rotation_pos),
        ];

        // First draw the bounding box
        {
            let painter = ui.painter();
            painter.add(Shape::rect_stroke(
                self.bounds,
                0.0,
                Stroke::new(1.0, HANDLE_COLOR),
            ));

            // Draw rotation line from center to rotation handle
            painter.add(Shape::line_segment(
                [center, rotation_pos],
                Stroke::new(1.0, HANDLE_COLOR),
            ));
        }

        // Draw handles and handle input
        for (handle_type, pos) in handles.iter() {
            let handle_rect = Rect::from_center_size(
                *pos,
                Vec2::splat(HANDLE_SIZE),
            );

            let handle_response = ui.allocate_rect(handle_rect, egui::Sense::drag());
            let is_active = self.active_handle == Some(*handle_type);
            let color = if handle_response.hovered() || is_active {
                HANDLE_HOVER_COLOR
            } else {
                HANDLE_COLOR
            };

            // Draw the handle
            {
                let painter = ui.painter();
                painter.add(Shape::rect_filled(
                    handle_rect,
                    0.0,
                    color,
                ));
            }

            // Handle input
            if handle_response.drag_started() {
                self.begin_transform(*handle_type, handle_response.hover_pos().unwrap_or(*pos));
                self.current_transform = *transform;
            } else if handle_response.drag_released() {
                if self.is_active {
                    self.end_transform();
                }
            } else if let Some(current_pos) = handle_response.hover_pos() {
                if handle_response.dragged() && self.active_handle == Some(*handle_type) {
                    let delta = current_pos - self.initial_pointer_pos.unwrap_or(*pos);
                    match handle_type {
                        GizmoHandle::Move => {
                            self.current_transform.position = self.initial_transform.position + Vec2::new(delta.x, delta.y);
                            *transform = self.current_transform;
                            changed = true;
                        }
                        GizmoHandle::Rotate => {
                            let initial_angle = (self.initial_pointer_pos.unwrap_or(*pos) - center).angle();
                            let current_angle = (current_pos - center).angle();
                            self.current_transform.rotation = self.initial_transform.rotation + (current_angle - initial_angle);
                            *transform = self.current_transform;
                            changed = true;
                        }
                        GizmoHandle::ScaleTopLeft |
                        GizmoHandle::ScaleTopRight |
                        GizmoHandle::ScaleBottomLeft |
                        GizmoHandle::ScaleBottomRight => {
                            let scale_delta = Vec2::new(delta.x, delta.y) / 100.0; // Scale factor
                            self.current_transform.scale = self.initial_transform.scale + scale_delta;
                            // Ensure minimum scale
                            self.current_transform.scale = self.current_transform.scale.max(Vec2::splat(0.1));
                            *transform = self.current_transform;
                            changed = true;
                        }
                    }
                }
            }
        }

        changed
    }
}

impl Default for TransformGizmo {
    fn default() -> Self {
        Self {
            bounds: Rect::NOTHING,
            active_handle: None,
            initial_transform: Transform::default(),
            current_transform: Transform::default(),
            initial_pointer_pos: None,
            completed_transform: None,
            is_active: false,
        }
    }
} 