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
use crate::element::Element;
use crate::widgets::resize_handle::Corner;
use std::collections::HashSet;

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
        // Find the tool by name
        let tool = self.available_tools.iter()
            .find(|t| t.name() == tool_name)
            .ok_or_else(|| format!("Tool '{}' not found", tool_name))?
            .clone();
        
        // If we have a current tool, deactivate it
        if let Some(current_tool) = self.state.active_tool() {
            let mut tool_clone = current_tool.clone();
            tool_clone.deactivate(&self.document);
        }
        
        // Clone the new tool and activate it
        let mut tool_clone = tool.clone();
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
        let element_id = element.id();
        let mut ids = HashSet::new();
        ids.insert(element_id);
        self.state = self.state.builder()
            .with_selected_element_ids(ids)
            .build();
    }

    pub fn execute_command(&mut self, command: Command) {
        log::info!("Executing command: {:?}", command);
        
        // Remember the element type for selection update
        let element_type = match &command {
            Command::ResizeElement { original_element, .. } => original_element.as_ref().map(|e| e.clone()),
            Command::MoveElement { original_element, .. } => original_element.as_ref().map(|e| e.clone()),
            _ => None,
        };
        
        // Step 1: Reset tool state, but retain reference to the selection
        if let Some(ToolType::Selection(tool)) = self.state.active_tool().cloned() {
            // Create a mutable copy
            let mut tool_copy = tool.clone();
            tool_copy.cancel_interaction();  // Completely reset interaction state
            tool_copy.clear_preview(&mut self.renderer);
            
            // Update the tool in the state
            self.state = self.state.update_tool(|_| Some(ToolType::Selection(tool_copy)));
        }
        
        // Step 2: Execute the command on the document
        self.command_history.execute(command.clone(), &mut self.document);
        
        // Step 3: Mark document as modified to force refresh
        self.document.mark_modified();
        
        // Step 4: Update selection state to track the transformed element
        if let Some(element) = element_type {
            // Find the updated element in the document using unified lookup
            let element_id = element.id();
            let updated_element = self.document.find_element_by_id(element_id);
            
            // Update the selection with the updated element
            if let Some(updated) = updated_element {
                let mut ids = HashSet::new();
                ids.insert(updated.id());
                self.state = self.state.builder()
                    .with_selected_element_ids(ids)
                    .build();
            }
        }
        
        // Step 5: Invalidate textures in the renderer
        command.invalidate_textures(&mut self.renderer);
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
        
        // Undo the command
        self.command_history.undo(&mut self.document);
        
        // Force a render update
        self.last_rendered_version = 0;
    }

    pub fn redo(&mut self) {
        // Reset the renderer's state completely
        self.renderer.reset_state();
        
        // Redo the command
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
        
        // Check if we have an active resize preview that needs to be finalized
        // This is a direct approach in case the pointer released event isn't being caught properly
        if let Some(preview_rect) = self.renderer.get_resize_preview() {
            // Get pointer state to check if mouse is released
            let pointer_released = ui.ctx().input(|i| !i.pointer.any_down());
            
            if pointer_released && self.processing_resize {
                log::info!("🔍 Direct resize detection: Preview active and pointer released!");
                
                // Find which element we're resizing
                if let Some(element_id) = self.state.selected_ids().iter().next() {
                    // Get the active corner if available, default to BottomRight
                    let corner = self.renderer.get_active_corner(*element_id)
                        .cloned()
                        .unwrap_or(crate::widgets::resize_handle::Corner::BottomRight);
                    
                    // Use the corner of the preview rect as the new position
                    let new_position = match corner {
                        crate::widgets::resize_handle::Corner::TopLeft => preview_rect.left_top(),
                        crate::widgets::resize_handle::Corner::TopRight => preview_rect.right_top(),
                        crate::widgets::resize_handle::Corner::BottomLeft => preview_rect.left_bottom(),
                        crate::widgets::resize_handle::Corner::BottomRight => preview_rect.right_bottom(),
                    };
                    
                    log::info!("🛠️ Direct resize finalization for element {}: corner={:?}, pos={:?}", 
                             element_id, corner, new_position);
                    
                    // Store element ID in a local variable
                    let element_id_copy = *element_id;
                    
                    // Get the original element
                    if let Some(element) = self.document.find_element(element_id_copy) {
                        log::info!("🔍 Direct resize - Found element to resize: {:?}", element);
                        
                        // For images, directly create a new one with the updated dimensions
                        if let ElementType::Image(image) = &element {
                            log::info!("📸 Direct resize for image - original size: {:?}, position: {:?}", 
                                     image.size(), image.position());
                            
                            // Create a new image with updated dimensions
                            let new_image = crate::image::Image::new_ref_with_id(
                                image.id(),
                                image.data().to_vec(), 
                                preview_rect.size(),
                                preview_rect.min
                            );
                            
                            log::info!("📸 New image dimensions: size={:?}, position={:?}", 
                                     new_image.size(), new_image.position());
                            
                            // Replace the image directly
                            let replaced = self.document.replace_image_by_id(element_id_copy, new_image);
                            log::info!("📸 Direct image replacement {}", if replaced { "SUCCEEDED" } else { "FAILED!" });
                            
                            // Force multiple document modifications to ensure redraw
                            for _ in 0..5 {
                                self.document.mark_modified();
                            }
                        } else {
                            // For other elements, use the command pattern
                            let cmd = crate::command::Command::ResizeElement {
                                element_id: element_id_copy,
                                corner,
                                new_position,
                                original_element: Some(element.clone()),
                            };
                            
                            // Execute the command
                            self.execute_command(cmd);
                        }
                    } else {
                        log::error!("❌ Could not find element {} for direct resize", element_id_copy);
                    }
                    
                    // Reset state
                    self.renderer.set_resize_preview(None);
                    self.renderer.clear_all_active_handles();
                    self.processing_resize = false;
                    
                    log::info!("✅ Direct resize completed for element {}", element_id_copy);
                    
                    // Force a repaint
                    ctx.request_repaint();
                }
            }
        }
        
        // Check if any resize is in progress and the mouse was released
        // We'll look at both the processing_resize flag and the current resize_preview
        if self.processing_resize || self.renderer.get_resize_preview().is_some() {
            let pointer_released = ui.ctx().input(|i| !i.pointer.any_down());
            
            if pointer_released {
                log::info!("🖱️ Resize operation: Mouse released detected (processing_resize={})", 
                         self.processing_resize);
                
                // If we have an active resize preview, process it now
                if let Some(final_rect) = self.renderer.get_resize_preview() {
                    // Check for selected element
                    if let Some(element_id) = self.state.selected_ids().iter().next() {
                        // Create a direct resize command
                        let element_id_copy = *element_id;
                        
                        // Get the element
                        if let Some(element) = self.document.find_element(element_id_copy) {
                            log::info!("📏 Forced resize on mouse release: element={}", element_id_copy);
                            
                            // Handle images directly
                            if let ElementType::Image(image) = &element {
                                // For images, do a direct replacement
                                let new_image = crate::image::Image::new_ref_with_id(
                                    image.id(),
                                    image.data().to_vec(),
                                    final_rect.size(),
                                    final_rect.min
                                );
                                
                                let replaced = self.document.replace_image_by_id(element_id_copy, new_image);
                                log::info!("📊 Forced image resize on release: {}", 
                                         if replaced { "SUCCESS" } else { "FAILED" });
                                
                                // Force document to update
                                for _ in 0..10 {
                                    self.document.mark_modified();
                                }
                            } else {
                                // For other element types, use the command
                                let corner = self.renderer.get_active_corner(element_id_copy)
                                    .cloned()
                                    .unwrap_or(crate::widgets::resize_handle::Corner::BottomRight);
                                
                                let new_position = match corner {
                                    crate::widgets::resize_handle::Corner::TopLeft => final_rect.left_top(),
                                    crate::widgets::resize_handle::Corner::TopRight => final_rect.right_top(),
                                    crate::widgets::resize_handle::Corner::BottomLeft => final_rect.left_bottom(),
                                    crate::widgets::resize_handle::Corner::BottomRight => final_rect.right_bottom(),
                                };
                                
                                // Create and execute the command
                                let cmd = Command::ResizeElement {
                                    element_id: element_id_copy,
                                    corner,
                                    new_position,
                                    original_element: Some(element.clone()),
                                };
                                
                                self.execute_command(cmd);
                            }
                        }
                    }
                    
                    // Reset resize state
                    self.renderer.set_resize_preview(None);
                    self.renderer.clear_all_active_handles();
                }
                
                // Reset the processing flag
                self.processing_resize = false;
            }
        }
        
        // Check if the document version has changed
        let doc_version = self.document.version();
        let should_redraw = doc_version != self.last_rendered_version;
        
        if should_redraw {
            log::info!("⚠️ Document version changed from {} to {}, forcing redraw", 
                      self.last_rendered_version, doc_version);
            // Force renderer to clear state when document changes
            self.renderer.reset_state();
            ctx.request_repaint();
        }
        
        // Update the version tracked
        self.last_rendered_version = doc_version;
        
        // Convert selected IDs to elements for rendering
        let selected_elements: Vec<ElementType> = self.state.selected_ids()
            .iter()
            .filter_map(|id| self.document.find_element_by_id(*id))
            .collect();
        
        // Render and get resize info
        let resize_result = self.renderer.render(
            ctx,
            ui,
            rect,
            &self.document,
            &selected_elements,
        );
        
        if let Some((element_id, corner, new_position)) = resize_result {
            // Update the resize preview while dragging, but don't create commands
            if let Some(element) = self.document.find_element_by_id(element_id) {
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
                log::info!("✅ Set processing_resize flag to TRUE for element {}", element_id);
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
            log::info!("🖱️ Drag released, executing resize command");
            log::info!("🖱️ Mouse position: {:?}", pointer.latest_pos());
            
            // When resize_result is None but we were resizing, it means drag was released
            // Get the current preview rectangle and create a final resize command
            if let Some(final_rect) = self.renderer.get_resize_preview() {
                // Find which element we were resizing
                if let Some(element_id) = self.state.selected_ids().iter().next() {
                    // Get the active corner if available, otherwise default to BottomRight
                    let corner = self.renderer.get_active_corner(*element_id)
                        .cloned()
                        .unwrap_or(Corner::BottomRight);
                        
                    // Determine the final position based on the corner
                    let new_position = match corner {
                        Corner::TopLeft => final_rect.left_top(),
                        Corner::TopRight => final_rect.right_top(),
                        Corner::BottomLeft => final_rect.left_bottom(),
                        Corner::BottomRight => final_rect.right_bottom(),
                    };
                    
                    // Store element_id in a local variable to avoid borrowing self
                    let element_id_copy = *element_id;
                    
                    // Get the original element directly
                    if let Some(element) = self.document.find_element(element_id_copy) {
                        log::info!("💠 Original element: {:?}", element);
                        
                        // Get the original rect
                        let original_rect = crate::geometry::hit_testing::compute_element_rect(&element);
                        log::info!("📐 Original rect: {:?}", original_rect);
                        
                        // Log the final rect
                        log::info!("📐 Final rect: {:?}", final_rect);
                        
                        // For image elements, directly create a new image with the updated size
                        if let ElementType::Image(image) = &element {
                            log::info!("🔄 DIRECT IMAGE RESIZE - Original image size: {:?}, position: {:?}", 
                                     image.size(), image.position());
                            
                            // Create a new image with updated rect
                            let new_image = crate::image::Image::new_ref_with_id(
                                image.id(),
                                image.data().to_vec(),
                                final_rect.size(),
                                final_rect.min
                            );
                            
                            log::info!("🔄 New image size: {:?}, position: {:?}", 
                                      new_image.size(), new_image.position());
                            
                            // Directly replace the image in the document
                            let replaced = self.document.replace_image_by_id(element_id_copy, new_image);
                            log::info!("🔄 Direct image replacement {}", if replaced { "SUCCEEDED" } else { "FAILED" });
                            
                            // Increment document version to force redraw
                            for _ in 0..5 {
                                self.document.mark_modified();
                            }
                            
                            // Force the state to update
                            self.last_rendered_version = 0;
                            self.renderer.reset_state();
                            ctx.request_repaint();
                        } else {
                            // For other elements, create a regular resize command
                            log::info!("📏 Creating resize command");
                            
                            // Create the resize command
                            let cmd = Command::ResizeElement {
                                element_id: element_id_copy,
                                corner,
                                new_position,
                                original_element: Some(element.clone()),
                            };
                            
                            // Execute the command
                            self.execute_command(cmd);
                        }
                    } else {
                        log::error!("❌ Could not find element {}", element_id_copy);
                    }
                    
                    // Force a complete renderer reset to clear any stale textures
                    self.renderer.reset_state();
                    
                    // Force a render update for the UI to reflect the change
                    self.last_rendered_version = 0;
                    
                    // Request a repaint to make sure the changes show immediately
                    ctx.request_repaint();
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
        self.renderer.clear_all_element_state();
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
        // Begin frame - prepare renderer for tracking what elements are rendered
        self.renderer.begin_frame();
        
        // Log current state before resetting
        log::info!("📊 Frame start - processing_resize flag: {}", self.processing_resize);
        
        // Only reset the flag if there are no active resize handles
        if !self.renderer.any_handles_active() {
            if self.processing_resize {
                log::info!("📊 Resetting processing_resize flag (no active handles)");
                self.processing_resize = false;
            }
        } else {
            log::info!("📊 Keeping processing_resize flag due to active handles");
        }
        
        // Handle input events
        self.handle_input(ctx);
        
        // Handle file drops
        self.handle_dropped_files(ctx);
        self.preview_files_being_dropped(ctx);
        
        tools_panel(self, ctx);
        central_panel(self, ctx);
        
        // Debug window to test direct image replacement
        if cfg!(debug_assertions) {
            egui::Window::new("Debug: Direct Image Editor")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    if ui.button("🔧 Direct Image Replacement").clicked() {
                        if !self.document.images().is_empty() {
                            let original_image = &self.document.images()[0];
                            let image_id = original_image.id();
                            let image_data = original_image.data().to_vec();
                            let data_size = image_data.len();
                            
                            log::info!("🔧 DEBUG WINDOW: Original image: ID={}, size={:?}, pos={:?}, data_len={}",
                                     image_id, original_image.size(), original_image.position(), data_size);
                            
                            // Create a completely new image with different dimensions but keep the data
                            let new_size = original_image.size() * 0.6; // 60% size
                            let new_pos = original_image.position() + egui::vec2(40.0, 40.0);
                            
                            // Debug the image data
                            let width = original_image.size().x as usize;
                            let height = original_image.size().y as usize;
                            let expected_bytes = width * height * 4;
                            
                            log::info!("🔧 Image dim check: {}x{} should have {} bytes, actual: {}",
                                     width, height, expected_bytes, data_size);
                            
                            // Check if the image data size matches dimensions
                            if data_size != expected_bytes {
                                log::warn!("⚠️ Image data size mismatch! Creating dummy data");
                                // Create a dummy image with solid color
                                let new_width = (new_size.x as usize).max(1);
                                let new_height = (new_size.y as usize).max(1);
                                let mut dummy_data = Vec::with_capacity(new_width * new_height * 4);
                                // Fill with blue color (RGBA: 0, 0, 255, 255)
                                for _ in 0..(new_width * new_height) {
                                    dummy_data.push(0);    // R
                                    dummy_data.push(0);    // G
                                    dummy_data.push(255);  // B
                                    dummy_data.push(255);  // A
                                }
                                
                                let new_image = crate::image::Image::new_ref_with_id(
                                    image_id,
                                    dummy_data,
                                    new_size,
                                    new_pos
                                );
                                
                                log::info!("🔧 Created dummy blue image with size={:?}, pos={:?}", 
                                         new_size, new_pos);
                                
                                // Directly replace in the document
                                let replaced = self.document.replace_image_by_id(image_id, new_image);
                                log::info!("🔧 DEBUG WINDOW: Dummy replacement {}", 
                                         if replaced { "SUCCEEDED" } else { "FAILED" });
                            } else {
                                // Create a properly sized image with original data
                                let new_image = crate::image::Image::new_ref_with_id(
                                    image_id,
                                    image_data,
                                    new_size,
                                    new_pos
                                );
                                
                                log::info!("🔧 DEBUG WINDOW: New image: size={:?}, pos={:?}",
                                         new_image.size(), new_image.position());
                                
                                // Directly replace in the document
                                let replaced = self.document.replace_image_by_id(image_id, new_image);
                                log::info!("🔧 DEBUG WINDOW: Direct replacement {}", 
                                         if replaced { "SUCCEEDED" } else { "FAILED" });
                            }
                            
                            // Force document modification and redraw
                            for _ in 0..10 {
                                self.document.mark_modified();
                            }
                            self.last_rendered_version = 0;
                            self.renderer.reset_state();
                            ctx.request_repaint();
                        } else {
                            log::info!("🔧 DEBUG WINDOW: No images to replace");
                        }
                    }
                });
        }
        
        // End frame - process rendered elements and cleanup orphaned textures
        self.renderer.end_frame(ctx);
        
        // Debug window for texture state
        if cfg!(debug_assertions) {
            egui::Window::new("Debug: Texture State")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.renderer.draw_debug_overlay(ui);
                });
        }
    }
}