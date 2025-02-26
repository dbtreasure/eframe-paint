use crate::command::{Command, CommandHistory};
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::stroke::Stroke;
use crate::panels::CentralPanel;
use egui;

use super::{InputEvent, PanelKind};

/// Routes input events to the appropriate handlers based on the current editor state
pub fn route_event(
    event: &InputEvent,
    state: &mut EditorState,
    document: &mut Document,
    command_history: &mut CommandHistory,
    renderer: &mut Renderer,
    central_panel: &CentralPanel,
    panel_rect: egui::Rect,
) {
    // Delegate to the central panel's input handler
    central_panel.handle_input_event(
        event, 
        state, 
        document, 
        command_history, 
        renderer,
        panel_rect,
    );
    
    // In the future, we could add more panel handlers here
    // For example:
    // if let Some(tools_panel) = tools_panel {
    //     tools_panel.handle_input_event(event, ...);
    // }
} 