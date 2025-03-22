use crate::command::{Command, CommandHistory};
use crate::panels::CentralPanel;
use crate::renderer::Renderer;
use crate::state::EditorModel;
use egui;

use super::{InputEvent, PanelKind};

pub fn route_event(
    event: &InputEvent,
    command_history: &mut CommandHistory,
    renderer: &mut Renderer,
    central_panel: &mut CentralPanel,
    panel_rect: egui::Rect,
    ui: &egui::Ui,
    editor_model: &mut EditorModel,
) {
    // Check if this is a pointer down event in the tools panel
    if let InputEvent::PointerDown { location, button } = event {
        if location.panel == PanelKind::Tools && *button == egui::PointerButton::Primary {
            // Clear the selection when clicking in the tools panel
            let clear_cmd = Command::new_clear_selection(editor_model);
            let _ = command_history
                .execute(clear_cmd, editor_model)
                .map_err(|err| log::info!("Clear selection failed: {}", err));
        }
    }

    // Route the event to the central panel
    central_panel.handle_input_event(
        event,
        command_history,
        renderer,
        panel_rect,
        ui,
        editor_model,
    );
}
