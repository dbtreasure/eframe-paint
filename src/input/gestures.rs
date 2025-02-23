use eframe::egui::{self, Vec2};
use super::state::{InputState, TouchPoint, TouchPhase};
use crate::util::time;

/// Represents a recognized gesture
#[derive(Debug, Clone)]
pub enum Gesture {
    /// Pinch gesture for zooming
    Pinch {
        center: Vec2,
        scale: f32,
        velocity: f32,
    },
    /// Rotation gesture
    Rotate {
        center: Vec2,
        angle: f32,
        velocity: f32,
    },
    /// Pan gesture for moving the view
    Pan {
        delta: Vec2,
        velocity: Vec2,
    },
    /// Tap gesture
    Tap {
        position: Vec2,
        count: u8,
    },
    /// Long press gesture
    LongPress {
        position: Vec2,
        duration: f32,
    },
}

/// Configuration for gesture recognition
#[derive(Debug, Clone)]
pub struct GestureConfig {
    /// Minimum distance for pan gesture
    pub min_pan_distance: f32,
    /// Minimum scale difference for pinch gesture
    pub min_pinch_scale: f32,
    /// Minimum angle for rotation gesture (radians)
    pub min_rotation_angle: f32,
    /// Maximum time between taps for multi-tap (seconds)
    pub multi_tap_time: f32,
    /// Time required for long press (seconds)
    pub long_press_time: f32,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            min_pan_distance: 5.0,
            min_pinch_scale: 0.1,
            min_rotation_angle: 0.1,
            multi_tap_time: 0.3,
            long_press_time: 0.5,
        }
    }
}

/// Recognizes common touch and mouse gestures
#[derive(Debug)]
pub struct GestureRecognizer {
    config: GestureConfig,
    last_touch_points: Vec<TouchPoint>,
    initial_distance: Option<f32>,
    initial_angle: Option<f32>,
    gesture_start_time: Option<f32>,
    last_tap_time: Option<f32>,
    tap_count: u8,
    last_position: Option<Vec2>,
    accumulated_pan: Vec2,
    accumulated_rotation: f32,
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self {
            config: GestureConfig::default(),
            last_touch_points: Vec::new(),
            initial_distance: None,
            initial_angle: None,
            gesture_start_time: None,
            last_tap_time: None,
            tap_count: 0,
            last_position: None,
            accumulated_pan: Vec2::ZERO,
            accumulated_rotation: 0.0,
        }
    }

    /// Update gesture state and return recognized gesture if any
    pub fn update(&mut self, input: &InputState) -> Option<Gesture> {
        // Handle touch input
        if !input.touch_points.is_empty() {
            return self.handle_touch_input(input);
        }

        // Handle mouse input
        self.handle_mouse_input(input)
    }

    fn handle_touch_input(&mut self, input: &InputState) -> Option<Gesture> {
        match input.touch_points.len() {
            1 => self.handle_single_touch(input),
            2 => self.handle_multi_touch(input),
            _ => None,
        }
    }

    fn handle_single_touch(&mut self, input: &InputState) -> Option<Gesture> {
        let touch = &input.touch_points[0];
        
        match touch.phase {
            TouchPhase::Started => {
                self.gesture_start_time = Some(self.current_time());
                self.last_position = Some(touch.pos.to_vec2());
                self.accumulated_pan = Vec2::ZERO;
                None
            }
            TouchPhase::Moved => {
                if let Some(last_pos) = self.last_position {
                    let delta = touch.pos.to_vec2() - last_pos;
                    self.last_position = Some(touch.pos.to_vec2());
                    self.accumulated_pan += delta;
                    
                    if self.accumulated_pan.length() >= self.config.min_pan_distance {
                        let velocity = delta / self.delta_time().max(0.001);
                        Some(Gesture::Pan {
                            delta: self.accumulated_pan,
                            velocity,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            TouchPhase::Ended => {
                if let Some(start_time) = self.gesture_start_time {
                    let duration = self.current_time() - start_time;
                    if duration >= self.config.long_press_time {
                        Some(Gesture::LongPress {
                            position: touch.pos.to_vec2(),
                            duration,
                        })
                    } else {
                        self.handle_tap(touch.pos.to_vec2())
                    }
                } else {
                    None
                }
            }
            TouchPhase::Cancelled => None,
        }
    }

    fn handle_multi_touch(&mut self, input: &InputState) -> Option<Gesture> {
        let p1 = input.touch_points[0].pos.to_vec2();
        let p2 = input.touch_points[1].pos.to_vec2();
        
        let center = (p1 + p2) / 2.0;
        let current_distance = (p2 - p1).length();
        let current_angle = (p2 - p1).angle();
        
        match (input.touch_points[0].phase, input.touch_points[1].phase) {
            (TouchPhase::Started, _) | (_, TouchPhase::Started) => {
                self.initial_distance = Some(current_distance);
                self.initial_angle = Some(current_angle);
                None
            }
            (TouchPhase::Moved, _) | (_, TouchPhase::Moved) => {
                if let (Some(initial_distance), Some(initial_angle)) = (self.initial_distance, self.initial_angle) {
                    let scale = current_distance / initial_distance;
                    let rotation = current_angle - initial_angle;
                    
                    // Detect primary gesture based on which threshold is exceeded first
                    if (scale - 1.0).abs() >= self.config.min_pinch_scale {
                        Some(Gesture::Pinch {
                            center,
                            scale,
                            velocity: (scale - 1.0) / self.delta_time().max(0.001),
                        })
                    } else if rotation.abs() >= self.config.min_rotation_angle {
                        Some(Gesture::Rotate {
                            center,
                            angle: rotation,
                            velocity: rotation / self.delta_time().max(0.001),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn handle_mouse_input(&mut self, input: &InputState) -> Option<Gesture> {
        if input.pointer_pressed {
            self.gesture_start_time = Some(self.current_time());
            self.last_position = input.pointer_pos.map(|p| p.to_vec2());
            None
        } else if input.pointer_released {
            if let Some(start_time) = self.gesture_start_time {
                let duration = self.current_time() - start_time;
                if duration >= self.config.long_press_time {
                    input.pointer_pos.map(|pos| Gesture::LongPress {
                        position: pos.to_vec2(),
                        duration,
                    })
                } else {
                    input.pointer_pos.map(|pos| self.handle_tap(pos.to_vec2())).flatten()
                }
            } else {
                None
            }
        } else if input.pointer_pressed && input.pointer_delta.length() > 0.0 {
            Some(Gesture::Pan {
                delta: input.pointer_delta,
                velocity: input.pointer_delta / self.delta_time().max(0.001),
            })
        } else {
            None
        }
    }

    fn handle_tap(&mut self, position: Vec2) -> Option<Gesture> {
        let current_time = self.current_time();
        
        if let Some(last_time) = self.last_tap_time {
            if current_time - last_time <= self.config.multi_tap_time {
                self.tap_count += 1;
            } else {
                self.tap_count = 1;
            }
        } else {
            self.tap_count = 1;
        }
        
        self.last_tap_time = Some(current_time);
        
        Some(Gesture::Tap {
            position,
            count: self.tap_count,
        })
    }

    fn current_time(&self) -> f32 {
        time::current_time()
    }

    fn delta_time(&self) -> f32 {
        // Should be provided by the frame time
        1.0 / 60.0 // Default to 60 FPS for now
    }
} 