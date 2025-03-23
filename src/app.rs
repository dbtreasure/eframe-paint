use crate::command::{Command, CommandHistory};
use crate::element::{ElementType, compute_element_rect};
use crate::file_handler::FileHandler;
use crate::panels::{CentralPanel, central_panel, tools_panel};
use crate::renderer::Renderer;
use crate::state::EditorModel;
use crate::tools::{Tool, ToolType, new_draw_stroke_tool, new_selection_tool};
use eframe::egui;

/// Main application state
pub struct PaintApp {
    renderer: Renderer,
    editor_model: EditorModel,
    command_history: CommandHistory,
    central_panel: CentralPanel,
    central_panel_rect: egui::Rect,
    available_tools: Vec<ToolType>,
    file_handler: FileHandler,
    last_rendered_version: u64,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Create all available tools
        let available_tools = vec![
            ToolType::DrawStroke(new_draw_stroke_tool()),
            ToolType::Selection(new_selection_tool()),
        ];

        Self {
            renderer: Renderer::new(cc),
            editor_model: EditorModel::new(),
            command_history: CommandHistory::new(),
            central_panel: CentralPanel::new(),
            central_panel_rect: egui::Rect::NOTHING,
            available_tools,
            file_handler: FileHandler::new(),
            last_rendered_version: 0,
        }
    }

    pub fn command_history(&self) -> &CommandHistory {
        &self.command_history
    }

    pub fn available_tools(&self) -> &[ToolType] {
        &self.available_tools
    }

    pub fn set_active_tool(&mut self, tool_name: &str) -> Result<(), String> {
        // Find the tool by name
        let tool = self
            .available_tools
            .iter()
            .find(|t| t.name() == tool_name)
            .ok_or_else(|| format!("Tool '{}' not found", tool_name))?
            .clone();

        // If we have a current tool, deactivate it
        let current_tool = self.editor_model.active_tool();
        let mut tool_clone = current_tool.clone();
        tool_clone.deactivate(&self.editor_model);
        
        // Clear any previews from the current tool
        tool_clone.clear_preview(&mut self.renderer);

        // Clone the new tool and activate it
        let mut new_tool_clone = tool.clone();
        new_tool_clone.activate(&self.editor_model);

        // Update the editor_model with the new tool
        self.editor_model.update_tool(|_| new_tool_clone.clone());

        Ok(())
    }

    pub fn set_active_tool_by_name(&mut self, tool_name: &str) {
        // This is a wrapper around set_active_tool that ignores errors
        if let Err(err) = self.set_active_tool(tool_name) {
            log::warn!("Failed to set active tool: {}", err);
        }
    }

    pub fn active_tool(&self) -> &ToolType {
        self.editor_model.active_tool()
    }

    pub fn active_tool_mut(&mut self) -> &mut ToolType {
        self.editor_model.active_tool_mut()
    }

    pub fn editor_model(&self) -> &EditorModel {
        &self.editor_model
    }

    /// Execute a command and update tool state
    pub fn execute_command(&mut self, command: Command) {
        log::info!("Executing command: {:?}", command);

        // Remember the element ID for selection update
        let element_id = match &command {
            Command::ResizeElement { element_id, .. } => Some(*element_id),
            Command::MoveElement { element_id, .. } => Some(*element_id),
            _ => None,
        };

        // Step 1: Reset the active tool's interaction state
        let mut tool = self.editor_model.active_tool().clone();
        tool.reset_interaction_state();
        tool.clear_preview(&mut self.renderer);
        self.editor_model.update_tool(|_| tool);

        // Step 2: Execute the command on editor_model and handle any errors
        let _ = self
            .command_history
            .execute(command.clone(), &mut self.editor_model)
            .map_err(|err| log::warn!("Command execution failed: {}", err));

        // Step 3: Update selection state to track the transformed element
        if let Some(id) = element_id {
            // Update editor_model with the element ID if it still exists
            if self.editor_model.contains_element(id) {
                self.editor_model.with_selected_element_id(Some(id));
            }
        }

        // Step 4: Invalidate textures in the renderer
        command.invalidate_textures(&mut self.renderer);
    }

    pub fn set_central_panel_rect(&mut self, rect: egui::Rect) {
        self.central_panel_rect = rect;
    }

    pub fn undo(&mut self) {
        // Reset the renderer's state completely
        self.renderer.reset_state();

        // Undo the command on editor_model and handle any errors
        let _ = self
            .command_history
            .undo(&mut self.editor_model)
            .map_err(|err| log::info!("Undo operation: {}", err));

        // Force a render update
        self.last_rendered_version = 0;
    }

    pub fn redo(&mut self) {
        // Reset the renderer's state completely
        self.renderer.reset_state();

        // Redo the command on editor_model and handle any errors
        let _ = self
            .command_history
            .redo(&mut self.editor_model)
            .map_err(|err| log::info!("Redo operation: {}", err));

        // Force a render update
        self.last_rendered_version = 0;
    }

    pub fn handle_tool_ui(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        // Clone the editor_model to avoid borrowing issues
        let editor_model_clone = self.editor_model.clone();
        let mut tool = self.active_tool().clone();
        let cmd = tool.ui(ui, &editor_model_clone);
        
        // Update the tool in the editor model
        self.editor_model.update_tool(|_| tool);
        
        cmd
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        // Use the file handler to check for and process dropped files
        if self.file_handler.check_for_dropped_files(ctx) {
            // Process dropped files and get commands to execute
            let commands = self
                .file_handler
                .process_dropped_files(ctx, self.central_panel_rect);

            // Execute each command
            for command in commands {
                self.execute_command(command);
            }
        }
    }

    fn preview_files_being_dropped(&self, ctx: &egui::Context) {
        self.file_handler.preview_files_being_dropped(ctx);
    }

    pub fn get_first_selected_element(&self) -> Option<ElementType> {
        // Use editor_model's selected_element method directly
        self.editor_model.selected_element().cloned()
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Begin frame - prepare renderer for tracking what elements are rendered
        self.renderer.begin_frame();

        // Handle file drops
        self.handle_dropped_files(ctx);
        self.preview_files_being_dropped(ctx);

        // Show the tools panel
        tools_panel(self, ctx);

        // Show the central panel for editing
        let panel_rect = central_panel(
            &mut self.editor_model,
            &mut self.command_history,
            &mut self.renderer,
            ctx,
        );

        // Store the panel rect for future use
        self.set_central_panel_rect(panel_rect);

        // End frame - process rendered elements and cleanup orphaned textures
        self.renderer.end_frame(ctx);
    }
}
