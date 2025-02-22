use egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2, Ui};
use serde::{Serialize, Deserialize};
use crate::layer::Transform;

const HANDLE_SIZE: f32 = 8.0;
const ROTATION_HANDLE_OFFSET: f32 = 30.0;
const HANDLE_COLOR: Color32 = Color32::from_rgb(30, 144, 255);
const HANDLE_HOVER_COLOR: Color32 = Color32::from_rgb(135, 206, 250);
const HANDLE_ACTIVE_COLOR: Color32 = Color32::from_rgb(255, 165, 0);
const HANDLE_STROKE_WIDTH: f32 = 2.0;
const GRID_SIZE: f32 = 10.0;
const ANGLE_SNAP_THRESHOLD: f32 = 5.0; // degrees

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GizmoHandle {
    Move,
    ScaleTopLeft,
    ScaleTopRight,
    ScaleBottomLeft,
    ScaleBottomRight,
    Rotate,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SnapMode {
    None,
    Grid,
    Angle,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GizmoPreferences {
    pub snap_mode: SnapMode,
    pub grid_size: f32,
    pub angle_snap_degrees: f32,
    pub show_grid: bool,
    pub show_measurements: bool,
    pub maintain_aspect_ratio: bool,
}

impl Default for GizmoPreferences {
    fn default() -> Self {
        Self {
            snap_mode: SnapMode::None,
            grid_size: GRID_SIZE,
            angle_snap_degrees: 45.0,
            show_grid: false,
            show_measurements: true,
            maintain_aspect_ratio: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransformGizmo {
    bounds: Rect,
    active_handle: Option<GizmoHandle>,
    hovered_handle: Option<GizmoHandle>,
    initial_transform: Transform,
    current_transform: Transform,
    initial_pointer_pos: Option<Pos2>,
    pub completed_transform: Option<(Transform, Transform)>,
    pub is_active: bool,
    preferences: GizmoPreferences,
    initial_aspect_ratio: Option<f32>,
}

impl TransformGizmo {
    pub fn new(bounds: Rect, initial_transform: Transform) -> Self {
        Self {
            bounds,
            active_handle: None,
            hovered_handle: None,
            initial_transform: initial_transform.clone(),
            current_transform: initial_transform,
            initial_pointer_pos: None,
            completed_transform: None,
            is_active: false,
            preferences: GizmoPreferences::default(),
            initial_aspect_ratio: if bounds.is_positive() {
                Some(bounds.width() / bounds.height())
            } else {
                None
            },
        }
    }

    pub fn set_preferences(&mut self, preferences: GizmoPreferences) {
        self.preferences = preferences;
    }

    pub fn get_preferences(&self) -> &GizmoPreferences {
        &self.preferences
    }

    pub fn get_bounds(&self) -> Rect {
        self.bounds
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

    pub fn update_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
        if self.initial_aspect_ratio.is_none() && new_bounds.is_positive() {
            self.initial_aspect_ratio = Some(new_bounds.width() / new_bounds.height());
        }
    }

    fn snap_to_grid(&self, pos: Vec2) -> Vec2 {
        if matches!(self.preferences.snap_mode, SnapMode::Grid | SnapMode::Both) {
            Vec2::new(
                (pos.x / self.preferences.grid_size).round() * self.preferences.grid_size,
                (pos.y / self.preferences.grid_size).round() * self.preferences.grid_size,
            )
        } else {
            pos
        }
    }

    fn snap_to_angle(&self, angle: f32) -> f32 {
        if matches!(self.preferences.snap_mode, SnapMode::Angle | SnapMode::Both) {
            let snap = self.preferences.angle_snap_degrees.to_radians();
            (angle / snap).round() * snap
        } else {
            angle
        }
    }

    fn maintain_aspect_ratio(&self, scale: Vec2) -> Vec2 {
        if self.preferences.maintain_aspect_ratio {
            if let Some(ratio) = self.initial_aspect_ratio {
                let avg_scale = (scale.x + scale.y) / 2.0;
                Vec2::new(avg_scale, avg_scale * ratio)
            } else {
                scale
            }
        } else {
            scale
        }
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
        
        // Calculate rotation handle position
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

        // Draw grid if enabled
        if self.preferences.show_grid {
            let painter = ui.painter();
            let grid_color = Color32::from_rgba_premultiplied(255, 255, 255, 30);
            
            let min_x = (self.bounds.min.x / self.preferences.grid_size).floor() * self.preferences.grid_size;
            let max_x = (self.bounds.max.x / self.preferences.grid_size).ceil() * self.preferences.grid_size;
            let min_y = (self.bounds.min.y / self.preferences.grid_size).floor() * self.preferences.grid_size;
            let max_y = (self.bounds.max.y / self.preferences.grid_size).ceil() * self.preferences.grid_size;

            for x in (min_x as i32..=max_x as i32).step_by(self.preferences.grid_size as usize) {
                painter.line_segment(
                    [Pos2::new(x as f32, min_y), Pos2::new(x as f32, max_y)],
                    Stroke::new(1.0, grid_color),
                );
            }

            for y in (min_y as i32..=max_y as i32).step_by(self.preferences.grid_size as usize) {
                painter.line_segment(
                    [Pos2::new(min_x, y as f32), Pos2::new(max_x, y as f32)],
                    Stroke::new(1.0, grid_color),
                );
            }
        }

        // First draw the bounding box
        {
            let painter = ui.painter();
            painter.add(Shape::rect_stroke(
                self.bounds,
                0.0,
                Stroke::new(1.0, HANDLE_COLOR),
            ));

            // Draw rotation line
            painter.add(Shape::line_segment(
                [center, rotation_pos],
                Stroke::new(1.0, HANDLE_COLOR),
            ));

            // Draw measurements if enabled
            if self.preferences.show_measurements {
                let text = format!("{:.0}x{:.0}", self.bounds.width(), self.bounds.height());
                painter.text(
                    self.bounds.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::default(),
                    Color32::WHITE,
                );

                if let Some(handle) = self.active_handle {
                    if handle == GizmoHandle::Rotate {
                        let angle = self.current_transform.rotation.to_degrees();
                        let text = format!("{:.1}°", angle);
                        painter.text(
                            rotation_pos,
                            egui::Align2::CENTER_BOTTOM,
                            text,
                            egui::FontId::default(),
                            Color32::WHITE,
                        );
                    }
                }
            }
        }

        // Draw handles and handle input
        for (handle_type, pos) in handles.iter() {
            let handle_rect = Rect::from_center_size(
                *pos,
                Vec2::splat(HANDLE_SIZE),
            );

            let handle_response = ui.allocate_rect(handle_rect, egui::Sense::drag());
            
            // Update hover state
            if handle_response.hovered() {
                self.hovered_handle = Some(*handle_type);
            } else if self.hovered_handle == Some(*handle_type) {
                self.hovered_handle = None;
            }

            let is_active = self.active_handle == Some(*handle_type);
            let is_hovered = self.hovered_handle == Some(*handle_type);
            
            let color = if is_active {
                HANDLE_ACTIVE_COLOR
            } else if is_hovered {
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
            } else if handle_response.drag_stopped() {
                if self.is_active {
                    self.end_transform();
                }
            } else if let Some(current_pos) = handle_response.hover_pos() {
                if handle_response.dragged() && self.active_handle == Some(*handle_type) {
                    let delta = current_pos - self.initial_pointer_pos.unwrap_or(*pos);
                    match handle_type {
                        GizmoHandle::Move => {
                            let snapped_delta = self.snap_to_grid(delta);
                            self.current_transform.position = self.initial_transform.position + snapped_delta;
                            *transform = self.current_transform;
                            changed = true;
                        }
                        GizmoHandle::Rotate => {
                            let initial_angle = (self.initial_pointer_pos.unwrap_or(*pos) - center).angle();
                            let current_angle = (current_pos - center).angle();
                            let angle_delta = current_angle - initial_angle;
                            self.current_transform.rotation = self.snap_to_angle(
                                self.initial_transform.rotation + angle_delta
                            );
                            *transform = self.current_transform;
                            changed = true;
                        }
                        GizmoHandle::ScaleTopLeft |
                        GizmoHandle::ScaleTopRight |
                        GizmoHandle::ScaleBottomLeft |
                        GizmoHandle::ScaleBottomRight => {
                            let scale_delta = Vec2::new(delta.x, delta.y) / 100.0;
                            let new_scale = self.maintain_aspect_ratio(
                                self.initial_transform.scale + scale_delta
                            );
                            self.current_transform.scale = new_scale.max(Vec2::splat(0.1));
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
            hovered_handle: None,
            initial_transform: Transform::default(),
            current_transform: Transform::default(),
            initial_pointer_pos: None,
            completed_transform: None,
            is_active: false,
            preferences: GizmoPreferences::default(),
            initial_aspect_ratio: None,
        }
    }
} 