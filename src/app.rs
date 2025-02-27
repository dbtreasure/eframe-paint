use eframe::egui;
use crate::renderer::Renderer;
use crate::document::Document;
use crate::state::EditorState;
use crate::command::{Command, CommandHistory};
use crate::panels::{central_panel, tools_panel, CentralPanel};
use crate::input::{InputHandler, route_event};
use crate::tools::{Tool, DrawStrokeTool};

pub struct PaintApp {
    renderer: Renderer,
    document: Document,
    state: EditorState,
    command_history: CommandHistory,
    input_handler: InputHandler,
    central_panel: CentralPanel,
    central_panel_rect: egui::Rect,
    available_tools: Vec<Box<dyn Tool>>,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Create all available tools
        let available_tools = vec![
            Box::new(DrawStrokeTool::new()) as Box<dyn Tool>,
            // Add more tools here as they are implemented
        ];
        
        Self {
            renderer: Renderer::new(cc),
            document: Document::new(),
            state: EditorState::default(),
            command_history: CommandHistory::new(),
            input_handler: InputHandler::new(),
            central_panel: CentralPanel::new(),
            central_panel_rect: egui::Rect::NOTHING,
            available_tools,
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn command_history(&self) -> &CommandHistory {
        &self.command_history
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn central_panel(&self) -> &CentralPanel {
        &self.central_panel
    }

    pub fn central_panel_rect(&self) -> egui::Rect {
        self.central_panel_rect
    }
    
    pub fn available_tools(&self) -> &[Box<dyn Tool>] {
        &self.available_tools
    }
    
    pub fn set_active_tool_by_name(&mut self, tool_name: &str) {
        // Find the tool with the matching name
        for tool in &self.available_tools {
            if tool.name() == tool_name {
                // Create a new instance of the tool based on the name
                if tool_name == "Draw Stroke" {
                    self.set_active_tool(DrawStrokeTool::new());
                }
                // Add more tool types here as they are implemented
                
                break;
            }
        }
    }

    pub fn set_active_tool<T: Tool + 'static>(&mut self, tool: T) {
        self.state.set_active_tool(tool, &self.document);
    }

    pub fn active_tool(&self) -> Option<&dyn Tool> {
        self.state.active_tool()
    }

    pub fn active_tool_mut(&mut self) -> Option<&mut dyn Tool> {
        self.state.active_tool_mut()
    }

    pub fn execute_command(&mut self, command: Command) {
        self.command_history.execute(command, &mut self.document);
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        let events = self.input_handler.process_input(ctx);
        let panel_rect = self.central_panel_rect;

        for event in events {
            let central_panel = &self.central_panel;
            
            route_event(
                &event,
                &mut self.state,
                &mut self.document,
                &mut self.command_history,
                &mut self.renderer,
                central_panel,
                panel_rect,
            );
        }
    }

    pub fn set_central_panel_rect(&mut self, rect: egui::Rect) {
        self.central_panel_rect = rect;
        self.input_handler.set_central_panel_rect(rect);
    }

    pub fn set_tools_panel_rect(&mut self, rect: egui::Rect) {
        self.input_handler.set_tools_panel_rect(rect);
    }

    pub fn undo(&mut self) {
        self.command_history.undo(&mut self.document);
    }

    pub fn redo(&mut self) {
        self.command_history.redo(&mut self.document);
    }

    pub fn handle_tool_ui(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        if let Some(tool) = self.state.active_tool_mut() {
            tool.ui(ui, &self.document)
        } else {
            None
        }
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        
        tools_panel(self, ctx);
        central_panel(self, ctx);
    }
}