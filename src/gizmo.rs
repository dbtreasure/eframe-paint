use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Vec2, Ui};
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
    initial_pointer_pos: Option<Pos2>,
    pub completed_transform: Option<(Transform, Transform)>,
}

impl TransformGizmo {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            active_handle: None,
            initial_transform: Transform::default(),
            initial_pointer_pos: None,
            completed_transform: None,
        }
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

        // Draw the bounding box
        ui.painter().rect_stroke(
            self.bounds,
            0.0,
            Stroke::new(1.0, HANDLE_COLOR),
        );

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

        // Draw handles
        let handles = [
            (GizmoHandle::Move, center),
            (GizmoHandle::ScaleTopLeft, top_left),
            (GizmoHandle::ScaleTopRight, top_right),
            (GizmoHandle::ScaleBottomLeft, bottom_left),
            (GizmoHandle::ScaleBottomRight, bottom_right),
            (GizmoHandle::Rotate, rotation_pos),
        ];

        // Draw rotation line from center to rotation handle
        ui.painter().line_segment(
            [center, rotation_pos],
            Stroke::new(1.0, HANDLE_COLOR),
        );

        for (handle_type, pos) in handles.iter() {
            let handle_rect = Rect::from_center_size(
                *pos,
                Vec2::splat(HANDLE_SIZE),
            );

            let handle_response = ui.allocate_rect(handle_rect, Sense::drag());
            let is_active = self.active_handle == Some(*handle_type);
            let color = if handle_response.hovered() || is_active {
                HANDLE_HOVER_COLOR
            } else {
                HANDLE_COLOR
            };

            // Draw the handle
            match handle_type {
                GizmoHandle::Move => {
                    ui.painter().circle_filled(*pos, HANDLE_SIZE / 2.0, color);
                }
                GizmoHandle::Rotate => {
                    ui.painter().circle_stroke(*pos, HANDLE_SIZE / 2.0, Stroke::new(HANDLE_STROKE_WIDTH, color));
                }
                _ => {
                    ui.painter().rect_filled(handle_rect, 0.0, color);
                }
            }

            if handle_response.drag_started() {
                self.active_handle = Some(*handle_type);
                self.initial_transform = *transform;
                self.initial_pointer_pos = handle_response.hover_pos();
            }

            if let (Some(handle), Some(initial_pos)) = (self.active_handle, self.initial_pointer_pos) {
                if handle == *handle_type {
                    if let Some(current_pos) = handle_response.hover_pos() {
                        match handle {
                            GizmoHandle::Move => {
                                let delta = current_pos - initial_pos;
                                transform.position = self.initial_transform.position + delta;
                                changed = true;
                            }
                            GizmoHandle::Rotate => {
                                let center = self.bounds.center();
                                
                                // Calculate vectors from center to points in screen space
                                let initial_vec = initial_pos - center;
                                let current_vec = current_pos - center;
                                
                                // Calculate angles in screen space (y-axis points down)
                                let initial_angle = (-initial_vec.y).atan2(initial_vec.x);
                                let current_angle = (-current_vec.y).atan2(current_vec.x);
                                
                                // Calculate angle delta and normalize to [-π, π]
                                let mut angle_delta = current_angle - initial_angle;
                                if angle_delta > std::f32::consts::PI {
                                    angle_delta -= 2.0 * std::f32::consts::PI;
                                } else if angle_delta < -std::f32::consts::PI {
                                    angle_delta += 2.0 * std::f32::consts::PI;
                                }

                                // Draw debug visualization
                                let radius = ROTATION_HANDLE_OFFSET;
                                
                                // Draw the initial angle line
                                let initial_point = Pos2::new(
                                    center.x + radius * initial_angle.cos(),
                                    center.y - radius * initial_angle.sin()
                                );
                                ui.painter().line_segment(
                                    [center, initial_point],
                                    Stroke::new(1.0, Color32::RED)
                                );

                                // Draw the current angle line
                                let current_point = Pos2::new(
                                    center.x + radius * current_angle.cos(),
                                    center.y - radius * current_angle.sin()
                                );
                                ui.painter().line_segment(
                                    [center, current_point],
                                    Stroke::new(1.0, Color32::GREEN)
                                );

                                // Draw the angle arc
                                let points: Vec<Pos2> = (0..=30).map(|i| {
                                    let t = i as f32 / 30.0;
                                    let angle = initial_angle + t * angle_delta;
                                    Pos2::new(
                                        center.x + (radius * 0.8) * angle.cos(),
                                        center.y - (radius * 0.8) * angle.sin()
                                    )
                                }).collect();
                                ui.painter().add(Shape::line(
                                    points,
                                    Stroke::new(1.0, Color32::from_rgb(135, 206, 250))
                                ));
                                
                                // Update rotation and mark as changed
                                transform.rotation = self.initial_transform.rotation + angle_delta;
                                changed = true;
                            }
                            _ => {
                                // Scale handles
                                let _center = self.bounds.center();
                                
                                // Get the fixed point (corner being dragged)
                                let fixed_point = match handle {
                                    GizmoHandle::ScaleTopLeft => self.bounds.right_bottom(),
                                    GizmoHandle::ScaleTopRight => self.bounds.left_bottom(),
                                    GizmoHandle::ScaleBottomLeft => self.bounds.right_top(),
                                    GizmoHandle::ScaleBottomRight => self.bounds.left_top(),
                                    _ => unreachable!(),
                                };

                                // Calculate vectors from fixed point to initial and current positions
                                let initial_vec = initial_pos - fixed_point;
                                let current_vec = current_pos - fixed_point;

                                // Calculate scale factors for each axis independently
                                let scale_x = current_vec.x / initial_vec.x;
                                let scale_y = current_vec.y / initial_vec.y;

                                // If shift is held, use the larger scale factor to maintain aspect ratio
                                let scale = if ui.input(|i| i.modifiers.shift) {
                                    let max_scale = scale_x.abs().max(scale_y.abs());
                                    Vec2::new(
                                        max_scale * scale_x.signum(),
                                        max_scale * scale_y.signum()
                                    )
                                } else {
                                    Vec2::new(scale_x, scale_y)
                                };

                                // Apply the scale relative to the fixed point
                                transform.scale = self.initial_transform.scale * scale;

                                // Ensure minimum scale
                                transform.scale = transform.scale.max(Vec2::splat(0.1));

                                // Preserve rotation while scaling
                                transform.rotation = self.initial_transform.rotation;

                                changed = true;
                            }
                        }
                    }
                    // Check if the drag gesture has ended for the active handle
                    if handle_response.drag_stopped() {
                        self.completed_transform = Some((self.initial_transform.clone(), transform.clone()));
                        self.active_handle = None;
                        self.initial_pointer_pos = None;
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
            bounds: egui::Rect::NOTHING,
            active_handle: None,
            initial_transform: Transform::default(),
            initial_pointer_pos: None,
            completed_transform: None,
        }
    }
} 