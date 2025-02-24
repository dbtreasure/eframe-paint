use egui::{Pos2, PointerButton, Context, Rect, Key, Modifiers};
use crate::command::Command;
use crate::stroke::Stroke;

/// Represents the location where an input event occurred
#[derive(Debug, Clone, Copy)]
pub struct InputLocation {
    /// The position in screen coordinates
    pub position: Pos2,
    /// Whether this position is within the canvas bounds
    pub is_in_canvas: bool,
}

/// Represents different types of input events that can occur in the application
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse button was pressed
    PointerDown {
        location: InputLocation,
        button: PointerButton,
    },
    /// Mouse button was released
    PointerUp {
        location: InputLocation,
        button: PointerButton,
    },
    /// Mouse moved (with or without buttons pressed)
    PointerMove {
        location: InputLocation,
        /// Buttons that are currently held down
        held_buttons: Vec<PointerButton>,
    },
    /// Mouse entered the application window
    PointerEnter {
        location: InputLocation,
    },
    /// Mouse left the application window
    PointerLeave {
        last_known_location: InputLocation,
    },
    /// Key was pressed
    KeyDown {
        key: egui::Key,
        modifiers: egui::Modifiers,
    },
    /// Key was released
    KeyUp {
        key: egui::Key,
        modifiers: egui::Modifiers,
    },
}

impl InputEvent {
    /// Helper to check if an input event occurred within the canvas
    pub fn is_in_canvas(&self) -> bool {
        match self {
            InputEvent::PointerDown { location, .. } |
            InputEvent::PointerUp { location, .. } |
            InputEvent::PointerMove { location, .. } |
            InputEvent::PointerEnter { location, .. } => location.is_in_canvas,
            InputEvent::PointerLeave { last_known_location, .. } => last_known_location.is_in_canvas,
            _ => false,
        }
    }
}

/// Handles converting raw egui input into our domain-specific InputEvents
pub struct InputHandler {
    last_pointer_pos: Option<Pos2>,
    canvas_rect: Rect,
}

impl InputHandler {
    pub fn new(canvas_rect: Rect) -> Self {
        Self {
            last_pointer_pos: None,
            canvas_rect,
        }
    }

    /// Update the canvas rectangle (e.g. if window is resized)
    pub fn set_canvas_rect(&mut self, rect: Rect) {
        self.canvas_rect = rect;
    }

    /// Creates an InputLocation from a position
    fn make_location(&self, pos: Pos2) -> InputLocation {
        InputLocation {
            position: pos,
            is_in_canvas: self.canvas_rect.contains(pos),
        }
    }

    /// Process raw egui input and generate our InputEvents
    pub fn process_input(&mut self, ctx: &Context) -> Vec<InputEvent> {
        let mut events = Vec::new();
        
        // Handle pointer input
        ctx.input(|input| {
            // Track pointer position
            if let Some(pos) = input.pointer.hover_pos() {
                // If we didn't have a position before, this is a pointer enter
                if self.last_pointer_pos.is_none() {
                    events.push(InputEvent::PointerEnter {
                        location: self.make_location(pos),
                    });
                }
                
                // If position changed, this is a move
                if Some(pos) != self.last_pointer_pos {
                    let mut held_buttons = Vec::new();
                    for button in [PointerButton::Primary, PointerButton::Secondary, PointerButton::Middle] {
                        if input.pointer.button_down(button) {
                            held_buttons.push(button);
                        }
                    }
                    events.push(InputEvent::PointerMove {
                        location: self.make_location(pos),
                        held_buttons,
                    });
                }
                
                self.last_pointer_pos = Some(pos);
            } else if self.last_pointer_pos.is_some() {
                // Pointer left the window
                events.push(InputEvent::PointerLeave {
                    last_known_location: self.make_location(self.last_pointer_pos.unwrap()),
                });
                self.last_pointer_pos = None;
            }

            // Handle button presses
            for button in [PointerButton::Primary, PointerButton::Secondary, PointerButton::Middle] {
                if input.pointer.button_pressed(button) {
                    if let Some(pos) = input.pointer.hover_pos() {
                        events.push(InputEvent::PointerDown {
                            location: self.make_location(pos),
                            button,
                        });
                    }
                }
                if input.pointer.button_released(button) {
                    if let Some(pos) = input.pointer.hover_pos() {
                        events.push(InputEvent::PointerUp {
                            location: self.make_location(pos),
                            button,
                        });
                    }
                }
            }

            // Handle key events
            for event in &input.raw.events {
                match event {
                    egui::Event::Key {
                        key,
                        pressed,
                        modifiers,
                        ..
                    } => {
                        events.push(if *pressed {
                            InputEvent::KeyDown {
                                key: *key,
                                modifiers: *modifiers,
                            }
                        } else {
                            InputEvent::KeyUp {
                                key: *key,
                                modifiers: *modifiers,
                            }
                        });
                    }
                    _ => {}
                }
            }
        });

        events
    }
} 