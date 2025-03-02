use eframe::egui;
use crate::renderer::Renderer;
use crate::document::Document;
use crate::state::EditorState;
use crate::command::{Command, CommandHistory};
use crate::panels::{central_panel, tools_panel, CentralPanel};
use crate::input::{InputHandler, route_event};
use crate::tools::{ToolType, new_draw_stroke_tool, new_selection_tool, ToolPool};
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
    last_rendered_version: u64,
    tool_pool: ToolPool,
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
            last_rendered_version: 0,
            tool_pool: ToolPool::new(),
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
        // First, check if we need to clear selection
        let mut clear_selection = false;
        
        // Check if the current tool is a selection tool
        if let Some(current_tool) = self.state.active_tool() {
            if current_tool.is_selection_tool() {
                clear_selection = true;
            }
        }
        
        // Validate the transition if needed
        if !self.tool_pool.can_transition(&tool) {
            // If transition is invalid, don't change the tool
            return;
        }
        
        // Use update_tool to handle deactivation and activation
        self.state = self.state.update_tool(|current_tool| {
            // Deactivate current tool if there is one and return it to the pool
            if let Some(current_tool) = current_tool {
                // Clone the tool since we can't move out of a shared reference
                let tool_clone = current_tool.clone();
                let deactivated_tool = tool_clone.deactivate(&self.document);
                self.tool_pool.return_tool(deactivated_tool);
            }
            
            // Try to get the tool from the pool first
            let tool_name = tool.name();
            let activated_tool = self.tool_pool.get(tool_name)
                .unwrap_or_else(|| {
                    // If not in pool, activate the new tool
                    tool.activate(&self.document)
                });
            
            Some(activated_tool)
        });
        
        // If we need to clear selection, do it in a separate update
        if clear_selection {
            self.state = self.state.update_selection(|_| vec![]);
        }
    }

    pub fn active_tool(&self) -> Option<&ToolType> {
        self.state.active_tool()
    }

    pub fn set_active_element(&mut self, element: ElementType) {
        // Update the state with the selected element using update_selection
        self.state = self.state.update_selection(|_| vec![element.clone()]);
    }

    pub fn execute_command(&mut self, command: Command) {
        self.command_history.execute(command, &mut self.document);
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        let events = self.input_handler.process_input(ctx);
        let panel_rect = self.central_panel_rect;

        for event in events {
            route_event(
                &event,
                &mut self.state,
                &mut self.document,
                &mut self.command_history,
                &mut self.renderer,
                &mut self.central_panel,
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
        let mut result = None;
        
        self.state = self.state.update_tool(|maybe_tool| {
            if let Some(tool) = maybe_tool {
                // Clone the tool since we can't move out of a shared reference
                let mut tool_clone = tool.clone();
                
                // Use the tool to handle UI
                result = tool_clone.ui(ui, &self.document);
                
                // Return the potentially modified tool
                Some(tool_clone)
            } else {
                None
            }
        });
        
        // If the tool returned a command, execute it
        if let Some(cmd) = &result {
            self.command_history.execute(cmd.clone(), &mut self.document);
        }
        
        result
    }

    /// Render the document using the renderer
    pub fn render(&mut self, ctx: &egui::Context, painter: &egui::Painter, rect: egui::Rect) {
        // Check if state has changed since last render
        if self.state.version() != self.last_rendered_version {
            // Update renderer with current state snapshot
            self.update_renderer_state();
            self.last_rendered_version = self.state.version();
        }
        
        // This method avoids borrowing conflicts by managing access to document and renderer internally
        let selected_elements = self.state.selected_elements();
        self.renderer.render(ctx, painter, rect, &self.document, selected_elements);
    }
    
    /// Update renderer with current state snapshot
    fn update_renderer_state(&mut self) {
        // This method would contain any state-dependent renderer updates
        // Currently, we don't need to do anything specific here, but this is where
        // we would update any cached renderer state based on the editor state
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
    
    pub fn state(&self) -> &EditorState {
        &self.state
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