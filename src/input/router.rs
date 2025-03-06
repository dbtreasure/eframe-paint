use crate::command::CommandHistory;
use crate::document::Document;
use crate::renderer::Renderer;
use crate::state::EditorState;
use crate::panels::CentralPanel;
use egui;
use log::info;

use super::{InputEvent, PanelKind};

pub fn route_event(
    event: &InputEvent,
    state: &mut EditorState,
    document: &mut Document,
    command_history: &mut CommandHistory,
    renderer: &mut Renderer,
    central_panel: &mut CentralPanel,
    panel_rect: egui::Rect,
    use_unified_selection: bool,
) {
    // Check if this is a pointer down event in the tools panel
    if let InputEvent::PointerDown { location, button } = event {
        if location.panel == PanelKind::Tools && *button == egui::PointerButton::Primary {
            // Clear the selection when clicking in the tools panel
            *state = state.update_selection(|_| vec![]);
            
            // Also clear the unified selection tool state if it exists and is enabled
            if use_unified_selection {
                if let Some(selection_tool) = state.selection_tool_mut() {
                    info!("Canceling unified selection tool interaction from tools panel click");
                    selection_tool.cancel_interaction();
                }
            }
        }
    }

    // Route the event to the central panel
    central_panel.handle_input_event(
        event, 
        state, 
        document, 
        command_history, 
        renderer,
        panel_rect,
    );
} 