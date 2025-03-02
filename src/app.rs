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
use crate::error::TransitionError;

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
                match self.set_active_tool(tool.clone()) {
                    Ok(_) => {},
                    Err(e) => {
                        // In a real application, we would log this error or show it to the user
                        println!("Error setting active tool: {}", e);
                    }
                }
                break;
            }
        }
    }

    fn set_active_tool(&mut self, tool: ToolType) -> Result<(), TransitionError> {
        // Get the current state name for validation
        let current_state = self.state.active_tool()
            .map(|t| t.current_state_name())
            .unwrap_or("no-tool");
        
        // Validate transition
        if !self.tool_pool.validate_transition(current_state, &tool)? {
            return Err(TransitionError::InvalidStateTransition {
                from: current_state,
                to: tool.name(),
                state: tool.current_state_name().to_string()
            });
        }
        
        // First, check if we need to clear selection
        let mut clear_selection = false;
        
        // Check if the current tool is a selection tool
        if let Some(current_tool) = self.state.active_tool() {
            if current_tool.is_selection_tool() {
                clear_selection = true;
            }
        }
        
        // State retention logic
        let (new_state, old_tool) = self.state.take_active_tool();
        self.state = new_state;
        
        if let Some(old_tool) = old_tool {
            // Deactivate the old tool
            let deactivated_tool = old_tool.deactivate(&self.document);
            
            // Retain the state for future restoration
            self.tool_pool.retain_state(deactivated_tool);
        }
        
        // Pool retrieval with fallback
        let tool_name = tool.name();
        let mut activated_tool = self.tool_pool.get(tool_name)
            .unwrap_or_else(|| tool.activate(&self.document));
            
        // State restoration
        if let Some(retained) = self.tool_pool.get_retained_state(activated_tool.name()) {
            activated_tool.restore_state(retained);
        }
        
        // Update the state with the new tool
        self.state = self.state.update_tool(|_| Some(activated_tool));
        
        // If we need to clear selection, do it in a separate update
        if clear_selection {
            self.state = self.state.update_selection(|_| vec![]);
        }
        
        Ok(())
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