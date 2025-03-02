use eframe::egui;
use crate::renderer::Renderer;
use crate::document::Document;
use crate::state::EditorState;
use crate::command::{Command, CommandHistory};
use crate::panels::{central_panel, tools_panel, CentralPanel};
use crate::input::{InputHandler, route_event};
use crate::tools::{ToolType, new_draw_stroke_tool, new_selection_tool};
use crate::file_handler::FileHandler;
use crate::state::ElementType;

pub struct PaintApp {
    renderer: Renderer,
    document: Document,
    state: EditorState,
    command_history: CommandHistory,
    input_handler: InputHandler,
    central_panel: CentralPanel,
    central_panel_rect: egui::Rect,
    available_tools: Vec<ToolType>,
    file_handler: FileHandler,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Create all available tools
        let available_tools = vec![
            ToolType::Selection(new_selection_tool()),
            ToolType::DrawStroke(new_draw_stroke_tool()),
            // Add more tools here as they are implemented
        ];
        
        Self {
            renderer: Renderer::new(cc),
            document: Document::new(),
            state: EditorState::new(),
            command_history: CommandHistory::new(),
            input_handler: InputHandler::new(),
            central_panel: CentralPanel::new(),
            central_panel_rect: egui::Rect::NOTHING,
            available_tools,
            file_handler: FileHandler::new(),
        }
    }

    pub fn command_history(&self) -> &CommandHistory {
        &self.command_history
    }
    
    pub fn available_tools(&self) -> &[ToolType] {
        &self.available_tools
    }
    
    pub fn set_active_tool_by_name(&mut self, tool_name: &str) {
        // Find the tool with the matching name
        for tool in &self.available_tools {
            if tool.name() == tool_name {
                // Set the tool as active
                self.set_active_tool(tool.clone());
                break;
            }
        }
    }

    fn set_active_tool(&mut self, tool: ToolType) {
        // Deactivate current tool if there is one
        let mut state_builder = self.state.builder();
        
        if let Some(current_tool) = state_builder.take_active_tool() {
            // Check if the current tool is a selection tool
            let is_selection_tool = current_tool.is_selection_tool();
            
            // Deactivate the current tool and discard it
            current_tool.deactivate(&self.document);
            
            // If we're deactivating a selection tool, clear the selected elements
            if is_selection_tool {
                state_builder = state_builder.with_selected_elements(vec![]);
            }
        }
        
        // Activate the new tool
        let activated_tool = tool.activate(&self.document);
        
        // Update the state with the activated tool
        self.state = state_builder
            .with_active_tool(Some(activated_tool))
            .build();
    }

    pub fn active_tool(&self) -> Option<&ToolType> {
        self.state.active_tool()
    }

    pub fn set_active_element(&mut self, element: ElementType) {
        // Clone the element for use in multiple places
        let element_clone = element.clone();
        
        // Update the state with the selected element
        self.state = self.state.builder()
            .with_selected_elements(vec![element])
            .build();
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
        let mut state_builder = self.state.builder();
        
        if let Some(mut tool) = state_builder.take_active_tool() {
            // Use the tool to handle UI
            let result = tool.ui(ui, &self.document);
            
            // Update the state with the potentially modified tool
            self.state = state_builder
                .with_active_tool(Some(tool))
                .build();
            
            // If the tool returned a command, execute it
            if let Some(cmd) = &result {
                self.command_history.execute(cmd.clone(), &mut self.document);
            }
            
            result
        } else {
            None
        }
    }

    /// Render the document using the renderer
    pub fn render(&mut self, ctx: &egui::Context, painter: &egui::Painter, rect: egui::Rect) {
        // This method avoids borrowing conflicts by managing access to document and renderer internally
        let selected_elements = self.state.selected_elements();
        self.renderer.render(ctx, painter, rect, &self.document, selected_elements);
    }

    /// Handle dropped files
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        // Use the file handler to check for and process dropped files
        if self.file_handler.check_for_dropped_files(ctx) {
            // Process dropped files and get commands to execute
            let commands = self.file_handler.process_dropped_files(ctx, self.central_panel_rect);
            
            // Execute each command
            for command in commands {
                self.execute_command(command);
            }
        }
    }

    /// Preview files being dragged over the application
    fn preview_files_being_dropped(&self, ctx: &egui::Context) {
        self.file_handler.preview_files_being_dropped(ctx);
    }

    pub fn document(&self) -> &Document {
        &self.document
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle drag and drop files
        self.handle_dropped_files(ctx);
        self.preview_files_being_dropped(ctx);
        
        self.handle_input(ctx);
        
        tools_panel(self, ctx);
        central_panel(self, ctx);
    }
}