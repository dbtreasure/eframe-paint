use egui::{Context, PointerButton, Pos2, Rect};

mod router;
pub use router::route_event;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelKind {
    Central,
    Tools,
    Global,
}

#[derive(Debug, Clone, Copy)]
pub struct InputLocation {
    pub position: Pos2,
    pub panel: PanelKind,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    PointerDown {
        location: InputLocation,
        button: PointerButton,
    },
    PointerUp {
        location: InputLocation,
        button: PointerButton,
    },
    PointerMove {
        location: InputLocation,
        held_buttons: Vec<PointerButton>,
    },
    PointerEnter {
        location: InputLocation,
    },
    PointerLeave {
        last_known_location: InputLocation,
    },
}

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

    pub fn set_central_panel_rect(&mut self, rect: Rect) {
        self.central_panel_rect = Some(rect);
    }

    pub fn set_tools_panel_rect(&mut self, rect: Rect) {
        self.tools_panel_rect = Some(rect);
    }

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

        PanelKind::Global
    }

    fn make_location(&self, pos: Pos2) -> InputLocation {
        InputLocation {
            position: pos,
            panel: self.determine_panel(pos),
        }
    }

    /// Static method to process input events for a specific panel
    /// This is useful when we don't need to track state across frames
    pub fn process_input_static(ctx: &Context, panel_rect: Rect) -> Vec<InputEvent> {
        let mut events = Vec::new();
        let mut last_pointer_pos = None;

        // Helper function to determine panel
        let determine_panel = |pos: Pos2| -> PanelKind {
            if panel_rect.contains(pos) {
                PanelKind::Central
            } else {
                PanelKind::Global
            }
        };

        // Helper function to create location
        let make_location = |pos: Pos2| -> InputLocation {
            InputLocation {
                position: pos,
                panel: determine_panel(pos),
            }
        };

        ctx.input(|input| {
            if let Some(pos) = input.pointer.hover_pos() {
                // If position changed, this is a move
                if Some(pos) != last_pointer_pos {
                    let mut held_buttons = Vec::new();
                    for button in [
                        PointerButton::Primary,
                        PointerButton::Secondary,
                        PointerButton::Middle,
                    ] {
                        if input.pointer.button_down(button) {
                            held_buttons.push(button);
                        }
                    }
                    events.push(InputEvent::PointerMove {
                        location: make_location(pos),
                        held_buttons,
                    });
                }

                last_pointer_pos = Some(pos);
            }

            for button in [
                PointerButton::Primary,
                PointerButton::Secondary,
                PointerButton::Middle,
            ] {
                if input.pointer.button_pressed(button) {
                    if let Some(pos) = input.pointer.hover_pos() {
                        events.push(InputEvent::PointerDown {
                            location: make_location(pos),
                            button,
                        });
                    }
                }
                if input.pointer.button_released(button) {
                    if let Some(pos) = input.pointer.hover_pos() {
                        events.push(InputEvent::PointerUp {
                            location: make_location(pos),
                            button,
                        });
                    }
                }
            }
        });

        events
    }

    pub fn process_input(&mut self, ctx: &Context) -> Vec<InputEvent> {
        let mut events = Vec::new();

        ctx.input(|input| {
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
                    for button in [
                        PointerButton::Primary,
                        PointerButton::Secondary,
                        PointerButton::Middle,
                    ] {
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

            for button in [
                PointerButton::Primary,
                PointerButton::Secondary,
                PointerButton::Middle,
            ] {
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
