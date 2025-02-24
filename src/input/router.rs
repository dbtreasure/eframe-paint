use crate::command::{Command, CommandHistory};
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::stroke::Stroke;
use egui;

use super::InputEvent;

/// Routes input events to the appropriate handlers based on the current editor state
pub fn route_event(
    event: &InputEvent,
    state: &mut EditorState,
    document: &mut Document,
    command_history: &mut CommandHistory,
    renderer: &mut Renderer,
) {
    match event {
        InputEvent::PointerDown { location, button } 
            if *button == egui::PointerButton::Primary && location.is_in_canvas => {
            // Start a new stroke
            let stroke = Stroke::new(
                egui::Color32::BLACK,
                2.0,
            );
            *state = EditorState::start_drawing(stroke);
        }
        InputEvent::PointerMove { location, held_buttons } if location.is_in_canvas => {
            // Add point to stroke if we're drawing
            if held_buttons.contains(&egui::PointerButton::Primary) {
                if let EditorState::Drawing { current_stroke } = state {
                    current_stroke.add_point(location.position);
                    // Update preview
                    renderer.set_preview_stroke(Some(current_stroke.clone()));
                }
            }
        }
        InputEvent::PointerUp { location: _, button } 
            if *button == egui::PointerButton::Primary => {
            // Finish the stroke and add it to command history
            if let Some(stroke) = state.take_stroke() {
                command_history.execute(Command::AddStroke(stroke), document);
            }
            renderer.set_preview_stroke(None);
        }
        _ => {}
    }
} 