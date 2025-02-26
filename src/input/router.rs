use crate::command::CommandHistory;
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::panels::CentralPanel;
use egui;

use super::InputEvent;

pub fn route_event(
    event: &InputEvent,
    state: &mut EditorState,
    document: &mut Document,
    command_history: &mut CommandHistory,
    renderer: &mut Renderer,
    central_panel: &CentralPanel,
    panel_rect: egui::Rect,
) {
    central_panel.handle_input_event(
        event, 
        state, 
        document, 
        command_history, 
        renderer,
        panel_rect,
    );
} 