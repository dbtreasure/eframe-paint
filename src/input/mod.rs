use egui::{Pos2, PointerButton, Context, Rect};

mod router;
pub use router::route_event;

/// Represents which panel an input event occurred in
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelKind {
    /// The central drawing canvas
    Central,
    /// The tools side panel
    Tools,
    /// For events not associated with a specific panel (like keyboard shortcuts)
    Global,
}

/// Represents the location where an input event occurred
#[derive(Debug, Clone, Copy)]
pub struct InputLocation {
    /// The position in screen coordinates
    pub position: Pos2,
    /// The panel in which the event occurred
    pub panel: PanelKind,
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
}

/// Handles converting raw egui input into our domain-specific InputEvents
pub struct InputHandler {
    last_pointer_pos: Option<Pos2>,
    central_panel_rect: Option<Rect>,
    tools_panel_rect: Option<Rect>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            last_pointer_pos: None,
            central_panel_rect: None,
            tools_panel_rect: None,
        }
    }

    /// Update the central panel rectangle
    pub fn set_central_panel_rect(&mut self, rect: Rect) {
        self.central_panel_rect = Some(rect);
    }

    /// Update the tools panel rectangle
    pub fn set_tools_panel_rect(&mut self, rect: Rect) {
        self.tools_panel_rect = Some(rect);
    }

    /// Determine which panel a position is in
    fn determine_panel(&self, pos: Pos2) -> PanelKind {
        if let Some(rect) = self.central_panel_rect {
            if rect.contains(pos) {
                return PanelKind::Central;
            }
        }
        
        if let Some(rect) = self.tools_panel_rect {
            if rect.contains(pos) {
                return PanelKind::Tools;
            }
        }
        
        // Default to global if not in any panel
        PanelKind::Global
    }

    /// Creates an InputLocation from a position
    fn make_location(&self, pos: Pos2) -> InputLocation {
        InputLocation {
            position: pos,
            panel: self.determine_panel(pos),
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
        });

        events
    }
} 