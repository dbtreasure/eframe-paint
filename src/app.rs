use eframe::egui;
use crate::renderer::Renderer;
use crate::document::Document;
use crate::state::EditorState;
use crate::command::{Command, CommandHistory};
use crate::panels::{central_panel, tools_panel, CentralPanel};
use crate::input::{InputHandler, route_event};
use crate::tools::{ToolType, new_draw_stroke_tool, new_selection_tool, Tool};
use crate::file_handler::FileHandler;
use crate::state::ElementType;
use crate::widgets::resize_handle::Corner;

/// Main application state
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
    processing_resize: bool,
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
            document: Document::new(),
            state: EditorState::new(),
            command_history: CommandHistory::new(),
            input_handler: InputHandler::new(),
            central_panel: CentralPanel::new(),
            central_panel_rect: egui::Rect::NOTHING,
            available_tools,
            file_handler: FileHandler::new(),
            last_rendered_version: 0,
            processing_resize: false,
        }
    }

    pub fn command_history(&self) -> &CommandHistory {
        &self.command_history
    }
    
    pub fn available_tools(&self) -> &[ToolType] {
        &self.available_tools
    }
    
    pub fn set_active_tool(&mut self, tool_name: &str) -> Result<(), String> {
        log::info!("Setting active tool to {}", tool_name);
        
        // Find the tool in available_tools
        let tool = self.available_tools.iter()
            .find(|t| t.name() == tool_name)
            .ok_or_else(|| format!("Unknown tool: {}", tool_name))?
            .clone();
        
        // If we're already using this tool type, don't do anything
        if let Some(current_tool) = self.state.active_tool() {
            if current_tool.name() == tool.name() {
                log::info!("Tool {} is already active, not changing", tool_name);
                return Ok(());
            }
        }
        
        // First, save the current tool's configuration if there is one
        let current_config = self.state.active_tool()
            .map(|tool| tool.get_config());
        
        // Deactivate the current tool if there is one
        if let Some(current_tool) = self.state.active_tool() {
            log::info!("Deactivating current tool: {}", current_tool.name());
            // We need to clone the current tool to avoid borrowing issues
            let mut tool_clone = current_tool.clone();
            tool_clone.deactivate(&self.document);
        }
        
        // Reset the renderer's state completely when switching tools
        self.renderer.reset_state();
        
        // Create a clone of the tool to activate
        let mut tool_clone = tool.clone();
        
        // Apply the saved configuration if the tool types match
        if let Some(config) = current_config {
            if config.tool_name() == tool_clone.name() {
                // Apply config to the tool
                tool_clone.apply_config(&*config);
                log::info!("Applied saved configuration to tool");
            }
        }
        
        // Activate the new tool
        log::info!("Activating new tool: {}", tool_clone.name());
        tool_clone.activate(&self.document);
        
        // Update the state with the new tool
        self.state = self.state
            .update_tool(|_| Some(tool_clone))
            .update_selection(|_| vec![]);
        
        Ok(())
    }
    
    pub fn set_active_tool_by_name(&mut self, tool_name: &str) {
        // This is a wrapper around set_active_tool that ignores errors
        if let Err(err) = self.set_active_tool(tool_name) {
            log::warn!("Failed to set active tool: {}", err);
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
        // Reset renderer state for elements but preserve preview strokes
        self.renderer.reset_element_state();
        
        // Handle other commands normally
        self.command_history.execute(command, &mut self.document);
        
        // Force a render update
        self.last_rendered_version = 0;
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
        // Reset the renderer's state completely
        self.renderer.reset_state();
        self.command_history.undo(&mut self.document);
        // Force a render update
        self.last_rendered_version = 0;
    }

    pub fn redo(&mut self) {
        // Reset the renderer's state completely
        self.renderer.reset_state();
        self.command_history.redo(&mut self.document);
        // Force a render update
        self.last_rendered_version = 0;
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
            self.execute_command(cmd.clone());
        }
        
        result
    }

    /// Render the document using the renderer
    pub fn render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, rect: egui::Rect) {
        // Always update renderer state to ensure proper rendering
        self.update_renderer_state();
        self.last_rendered_version = self.state.version();
        
        // This method avoids borrowing conflicts by managing access to document and renderer internally
        let selected_elements = self.state.selected_elements();
        
        // Render and get resize info
        let resize_result = self.renderer.render(
            ctx,
            ui,
            rect,
            &self.document,
            selected_elements,
        );
        
        if let Some((element_id, corner, new_position)) = resize_result {
            // Update the resize preview while dragging, but don't create commands
            if let Some(element) = self.document.get_element_by_id(element_id) {
                log::info!("Updating resize preview for element {}", element_id);
                self.renderer.set_resize_preview(Some(
                    Renderer::compute_resized_rect(
                        crate::geometry::hit_testing::compute_element_rect(&element),
                        corner,
                        new_position
                    )
                ));
                
                // Set the flag to indicate we're in the middle of a resize operation
                self.processing_resize = true;
            } else {
                log::warn!("Element with id {} not found when updating resize preview", element_id);
            }
            
            log::info!("Updated resize preview for element={}, corner={:?}, pos={:?}", 
                      element_id, corner, new_position);
        }
        
        // Check if we should finalize a resize operation
        // This happens when the mouse is released after a resize operation
        let pointer = ui.ctx().input(|i| i.pointer.clone());
        if pointer.any_released() && self.processing_resize {
            log::info!("Drag released, executing resize command");
            
            // When resize_result is None but we were resizing, it means drag was released
            // Get the current preview rectangle and create a final resize command
            if let Some(final_rect) = self.renderer.get_resize_preview() {
                // Find which element we were resizing
                if let Some(element) = self.state.selected_elements().first() {
                    let element_id = match element {
                        ElementType::Image(img) => img.id(),
                        ElementType::Stroke(s) => std::sync::Arc::as_ptr(s) as usize,
                    };
                    
                    // Get the active corner if available, otherwise default to BottomRight
                    let corner = self.renderer.get_active_corner(element_id)
                        .cloned()
                        .unwrap_or(Corner::BottomRight);
                        
                    // Determine the final position based on the corner
                    let new_position = match corner {
                        Corner::TopLeft => final_rect.left_top(),
                        Corner::TopRight => final_rect.right_top(),
                        Corner::BottomLeft => final_rect.left_bottom(),
                        Corner::BottomRight => final_rect.right_bottom(),
                    };
                    
                    // Create the final resize command
                    let cmd = Command::ResizeElement {
                        element_id,
                        corner,
                        new_position,
                        original_element: self.document.get_element_by_id(element_id),
                    };
                    
                    // Execute the command
                    self.execute_command(cmd);
                    
                    // Update the editor state with the resized element
                    self.state = self.state.update_selection(|elements| {
                        elements.iter()
                            .map(|e| match e {
                                ElementType::Image(img) if img.id() == element_id => {
                                    // Find the updated image in the document
                                    if let Some(updated_img) = self.document.find_image_by_id(element_id) {
                                        ElementType::Image(updated_img.clone())
                                    } else {
                                        // If not found (should never happen), keep the original
                                        e.clone()
                                    }
                                },
                                _ => e.clone()
                            })
                            .collect::<Vec<_>>()
                    });
                    
                    // Force a render update for the UI to reflect the change
                    self.last_rendered_version = 0;
                }
            }
            
            // Clear the resize preview and reset the flag
            self.renderer.set_resize_preview(None);
            self.renderer.clear_all_active_handles();
            self.processing_resize = false;
            log::info!("Resize command processed and preview cleared");
        }
    }
    
    /// Update renderer with current state snapshot
    fn update_renderer_state(&mut self) {
        // This method would contain any state-dependent renderer updates
        // Currently, we don't need to do anything specific here, but this is where
        // we would update any cached renderer state based on the editor state
        
        // Reset element-related state but preserve preview strokes
        self.renderer.reset_element_state();
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
        // Reset the resize processing flag at the start of each frame
        self.processing_resize = false;
        
        // Handle input events
        self.handle_input(ctx);
        
        // Handle file drops
        self.handle_dropped_files(ctx);
        self.preview_files_being_dropped(ctx);
        
        tools_panel(self, ctx);
        central_panel(self, ctx);
    }
}