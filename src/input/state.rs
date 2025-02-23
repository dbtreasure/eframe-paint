use eframe::egui::{self, Pos2, Vec2, Key, Modifiers};
use std::collections::HashSet;

/// Represents the current state of all input devices
#[derive(Debug, Clone)]
pub struct InputState {
    // Mouse/Pointer state
    pub pointer_pos: Option<Pos2>,
    pub pointer_delta: Vec2,
    pub pointer_pressed: bool,
    pub pointer_released: bool,
    pub pointer_double_clicked: bool,
    pub scroll_delta: Vec2,
    
    // Pressure sensitivity
    pub pressure: Option<f32>,
    pub tilt: Option<Vec2>,
    
    // Keyboard state
    pub modifiers: Modifiers,
    pub pressed_keys: HashSet<Key>,
    pub typed_chars: Vec<char>,
    
    // Touch/Gesture state
    pub touch_points: Vec<TouchPoint>,
    pub pinch_scale: Option<f32>,
    pub rotation_delta: Option<f32>,
    
    // General state
    pub consumed: bool,  // Whether this input has been handled
}

/// Represents a single touch point
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub id: u64,
    pub pos: Pos2,
    pub phase: TouchPhase,
}

/// Phase of a touch interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pointer_pos: None,
            pointer_delta: Vec2::ZERO,
            pointer_pressed: false,
            pointer_released: false,
            pointer_double_clicked: false,
            scroll_delta: Vec2::ZERO,
            pressure: None,
            tilt: None,
            modifiers: Modifiers::default(),
            pressed_keys: HashSet::new(),
            typed_chars: Vec::new(),
            touch_points: Vec::new(),
            pinch_scale: None,
            rotation_delta: None,
            consumed: false,
        }
    }
}

impl InputState {
    /// Create a new InputState from egui raw input
    pub fn from_egui(ctx: &egui::Context) -> Self {
        let mut state = Self::default();
        
        ctx.input(|i| {
            // Pointer state
            state.pointer_pos = i.pointer.hover_pos();
            state.pointer_delta = i.pointer.delta();
            state.pointer_pressed = i.pointer.primary_pressed();
            state.pointer_released = i.pointer.primary_released();
            state.pointer_double_clicked = i.pointer.press_origin().is_some() && i.pointer.press_start_time().is_some();
            state.scroll_delta = i.raw_scroll_delta;
            
            // Keyboard state
            state.modifiers = i.modifiers;
            state.pressed_keys = i.keys_down.clone();
            state.typed_chars = i.events.iter()
                .filter_map(|e| match e {
                    egui::Event::Text(text) => Some(text.chars().next().unwrap_or('\0')),
                    _ => None,
                })
                .collect();
            
            // Multi-touch state
            if let Some(touch) = i.pointer.latest_pos() {
                state.touch_points = vec![TouchPoint {
                    id: 0,
                    pos: touch,
                    phase: if i.pointer.primary_pressed() {
                        TouchPhase::Started
                    } else if i.pointer.primary_released() {
                        TouchPhase::Ended
                    } else {
                        TouchPhase::Moved
                    },
                }];
            }
        });
        
        state
    }
    
    /// Returns true if any modifier key is pressed
    pub fn has_modifier(&self) -> bool {
        self.modifiers.ctrl || self.modifiers.alt || self.modifiers.shift || self.modifiers.command
    }
    
    /// Returns true if a specific key is pressed
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }
    
    /// Returns true if this is the start of a drag operation
    pub fn is_drag_start(&self) -> bool {
        self.pointer_pressed && self.pointer_pos.is_some()
    }
    
    /// Returns true if this is during a drag operation
    pub fn is_dragging(&self) -> bool {
        self.pointer_pressed && self.pointer_delta != Vec2::ZERO
    }
    
    /// Returns true if this is the end of a drag operation
    pub fn is_drag_end(&self) -> bool {
        self.pointer_released
    }
    
    /// Mark this input as consumed
    pub fn consume(&mut self) {
        self.consumed = true;
    }
    
    /// Returns true if this input has been consumed
    pub fn is_consumed(&self) -> bool {
        self.consumed
    }
} 