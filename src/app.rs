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
    /// Stores the last resize preview rectangle when a resize operation is in progress.
    /// This is needed because the resize preview is cleared when the mouse is released,
    /// but we still need the preview information to finalize the resize operation.
    last_resize_preview: Option<egui::Rect>,
    /// Stores the active corner being used for resizing.
    /// This is needed because the active corner is cleared when the mouse is released,
    /// but we still need the corner information to finalize the resize operation.
    last_active_corner: Option<crate::widgets::resize_handle::Corner>,
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
            last_resize_preview: None,
            last_active_corner: None,
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

    // Add a debug method to directly select an image by index
    pub fn debug_select_image_by_index(&mut self, index: usize) {
        if index < self.document.images().len() {
            let image = &self.document.images()[index];
            let image_id = image.id();
            let mut ids = HashSet::new();
            ids.insert(image_id);
            
            log::info!("🔍 DEBUG: Directly selecting image with ID: {}", image_id);
            
            self.state = self.state.builder()
                .with_selected_element_ids(ids)
                .build();
                
            log::info!("🔍 DEBUG: Updated selection IDs: {:?}", self.state.selected_ids());
        } else {
            log::warn!("🔍 DEBUG: Cannot select image at index {}, only {} images available", 
                     index, self.document.images().len());
        }
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

        // Process events with a temporary UI
        for event in events.clone() {  // Clone to avoid borrowing issues
            // Create a temporary UI for each event
            egui::CentralPanel::default().show(ctx, |ui| {
                route_event(
                    &event,
                    &mut self.state,
                    &mut self.document,
                    &mut self.command_history,
                    &mut self.renderer,
                    &mut self.central_panel,
                    panel_rect,
                    ui,
                );
            });
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
        
        // Check if we need to finalize a resize operation
        if self.processing_resize && 
           ui.ctx().input(|i| !i.pointer.any_down()) && 
           (self.renderer.get_resize_preview().is_some() || self.last_resize_preview.is_some()) {
            
            // Store the final rectangle before it's cleared
            let final_rect = self.renderer.get_resize_preview().or(self.last_resize_preview);
            
            if let Some(rect) = final_rect {
                // Get the selected element
                if let Some(element_id) = self.state.selected_ids().iter().next() {
                    // Find the element in the document
                    if let Some(element) = self.document.find_element_by_id(*element_id) {
                        // Force resize on mouse release
                        log::info!("Forced resize on mouse release: element={}", element_id);
                        
                        // Get the active corner from our stored value or default to BottomRight
                        let corner = self.last_active_corner
                            .unwrap_or(crate::widgets::resize_handle::Corner::BottomRight);
                        
                        // Determine the new position based on the corner
                        let new_position = match corner {
                            crate::widgets::resize_handle::Corner::TopLeft => rect.left_top(),
                            crate::widgets::resize_handle::Corner::TopRight => rect.right_top(),
                            crate::widgets::resize_handle::Corner::BottomLeft => rect.left_bottom(),
                            crate::widgets::resize_handle::Corner::BottomRight => rect.right_bottom(),
                        };
                        
                        // Create a resize command
                        let cmd = crate::command::Command::ResizeElement {
                            element_id: *element_id,
                            corner,
                            new_position,
                            original_element: Some(element.clone()),
                        };
                        
                        // Execute the command (this will add it to the command history)
                        self.execute_command(cmd);
                    }
                }
            }
            
            // Reset the resize state
            self.renderer.set_resize_preview(None);
            self.last_resize_preview = None;
            self.last_active_corner = None;
            self.renderer.clear_all_active_handles();
            self.processing_resize = false;
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
        
        if let Some((element_id, corner, pos)) = resize_result {
            // Get the element
            if let Some(element) = self.document.find_element_by_id(element_id) {
                // Get the original rect
                let original_rect = crate::geometry::hit_testing::compute_element_rect(&element);
                
                // Compute the new rectangle based on the corner and new position
                let new_rect = Renderer::compute_resized_rect(original_rect, corner, pos);
                
                // Set the resize preview
                self.renderer.set_resize_preview(Some(new_rect));
                
                // Store the resize preview and active corner
                self.last_resize_preview = Some(new_rect);
                self.last_active_corner = Some(corner);
                
                // Set the flag to indicate we're in the middle of a resize operation
                self.processing_resize = true;
            }
        }
    }
    
    /// Update renderer with current state snapshot
    fn update_renderer_state(&mut self) {
        // This method would contain any state-dependent renderer updates
        // Currently, we don't need to do anything specific here, but this is where
        // we would update any cached renderer state based on the editor state
        
        // Reset element-related state but preserve preview strokes and drag preview
        // Don't clear element state here as it interferes with drag preview
        // self.renderer.clear_all_element_state();
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
    
    // Helper method to get the first selected element
    pub fn get_first_selected_element(&self) -> Option<ElementType> {
        let selected_ids = self.state.selected_ids();
        if selected_ids.is_empty() {
            return None;
        }
        
        // Get the first ID from the set
        let first_id = *selected_ids.iter().next().unwrap();
        
        // Find the element in the document
        self.document.find_element_by_id(first_id)
    }
    
    // Helper function to resize image data
    fn resize_image_data(original_data: &[u8], original_width: usize, original_height: usize, 
                        new_width: usize, new_height: usize) -> Vec<u8> {
        // If dimensions match, return the original data
        if original_width == new_width && original_height == new_height {
            return original_data.to_vec();
        }
        
        // Create a new buffer for the resized image
        let mut new_data = Vec::with_capacity(new_width * new_height * 4);
        
        // Simple nearest-neighbor scaling
        for y in 0..new_height {
            for x in 0..new_width {
                // Map new coordinates to original image coordinates
                let orig_x = (x * original_width) / new_width;
                let orig_y = (y * original_height) / new_height;
                
                // Calculate pixel index in original data
                let orig_idx = (orig_y * original_width + orig_x) * 4;
                
                // Copy the pixel if it's within bounds
                if orig_idx + 3 < original_data.len() {
                    new_data.push(original_data[orig_idx]);     // R
                    new_data.push(original_data[orig_idx + 1]); // G
                    new_data.push(original_data[orig_idx + 2]); // B
                    new_data.push(original_data[orig_idx + 3]); // A
                } else {
                    // Use a default color (red) if out of bounds
                    new_data.push(255); // R
                    new_data.push(0);   // G
                    new_data.push(0);   // B
                    new_data.push(255); // A
                }
            }
        }
        
        new_data
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Begin frame - prepare renderer for tracking what elements are rendered
        self.renderer.begin_frame();
        
        // Check if we need to reset the processing_resize flag
        if !self.renderer.any_handles_active() {
            if self.processing_resize {
                log::debug!("Resetting processing_resize flag (no active handles)");
                self.processing_resize = false;
            }
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
                    
                    // Add a new button for resizing selected images
                    if ui.button("🔍 Resize Selected Image (3/4 width, 1/2 height)").clicked() {
                        // Get selected elements
                        let selected_ids = self.state.selected_ids();
                        log::info!("🔍 DEBUG WINDOW: Selected IDs: {:?}", selected_ids);
                        
                        let selected_elements: Vec<ElementType> = self.state.selected_ids()
                            .iter()
                            .filter_map(|id| self.document.find_element_by_id(*id))
                            .collect();
                        
                        log::info!("🔍 DEBUG WINDOW: Selected elements count: {}", selected_elements.len());
                        
                        // Find the first selected image
                        let selected_image = selected_elements.iter()
                            .filter_map(|element| {
                                if let ElementType::Image(image) = element {
                                    Some(image)
                                } else {
                                    None
                                }
                            })
                            .next();
                        
                        if let Some(image) = selected_image {
                            let image_id = image.id();
                            let image_data = image.data().to_vec();
                            let original_size = image.size();
                            let original_pos = image.position();
                            
                            log::info!("🔍 DEBUG WINDOW: Resizing selected image: ID={}, size={:?}, pos={:?}",
                                     image_id, original_size, original_pos);
                            
                            // Calculate new size: 3/4 width, 1/2 height
                            let new_size = egui::vec2(
                                original_size.x * 0.75, // 3/4 width
                                original_size.y * 0.5   // 1/2 height
                            );
                            
                            // Keep the same position
                            let new_pos = original_pos;
                            
                            // Resize the image data to match the new dimensions
                            let original_width = original_size.x as usize;
                            let original_height = original_size.y as usize;
                            let new_width = new_size.x as usize;
                            let new_height = new_size.y as usize;
                            
                            log::info!("🔍 DEBUG WINDOW: Resizing image data from {}x{} to {}x{}", 
                                     original_width, original_height, new_width, new_height);
                            
                            let resized_data = Self::resize_image_data(
                                &image_data, 
                                original_width, 
                                original_height, 
                                new_width, 
                                new_height
                            );
                            
                            log::info!("🔍 DEBUG WINDOW: Resized data length: {} (expected: {})", 
                                     resized_data.len(), new_width * new_height * 4);
                            
                            // Create a properly sized image with resized data
                            let new_image = crate::image::Image::new_ref_with_id(
                                image_id,
                                resized_data,
                                new_size,
                                new_pos
                            );
                            
                            log::info!("🔍 DEBUG WINDOW: New image: size={:?}, pos={:?}",
                                     new_image.size(), new_image.position());
                            
                            // Directly replace in the document
                            let replaced = self.document.replace_image_by_id(image_id, new_image);
                            log::info!("🔍 DEBUG WINDOW: Direct replacement {}", 
                                     if replaced { "SUCCEEDED" } else { "FAILED" });
                            
                            // Force document modification and redraw
                            for _ in 0..10 {
                                self.document.mark_modified();
                            }
                            self.last_rendered_version = 0;
                            self.renderer.reset_state();
                            ctx.request_repaint();
                        } else {
                            log::info!("🔍 DEBUG WINDOW: No selected image to resize");
                            
                            // FALLBACK: If no image is selected, use the first image in the document
                            if !self.document.images().is_empty() {
                                let image = &self.document.images()[0];
                                let image_id = image.id();
                                let image_data = image.data().to_vec();
                                let original_size = image.size();
                                let original_pos = image.position();
                                
                                log::info!("🔍 DEBUG WINDOW: Using first image as fallback: ID={}, size={:?}, pos={:?}",
                                         image_id, original_size, original_pos);
                                
                                // Calculate new size: 3/4 width, 1/2 height
                                let new_size = egui::vec2(
                                    original_size.x * 0.75, // 3/4 width
                                    original_size.y * 0.5   // 1/2 height
                                );
                                
                                // Keep the same position
                                let new_pos = original_pos;
                                
                                // Resize the image data to match the new dimensions
                                let original_width = original_size.x as usize;
                                let original_height = original_size.y as usize;
                                let new_width = new_size.x as usize;
                                let new_height = new_size.y as usize;
                                
                                log::info!("🔍 DEBUG WINDOW: Resizing image data from {}x{} to {}x{}", 
                                         original_width, original_height, new_width, new_height);
                                
                                let resized_data = Self::resize_image_data(
                                    &image_data, 
                                    original_width, 
                                    original_height, 
                                    new_width, 
                                    new_height
                                );
                                
                                log::info!("🔍 DEBUG WINDOW: Resized data length: {} (expected: {})", 
                                         resized_data.len(), new_width * new_height * 4);
                                
                                // Create a properly sized image with resized data
                                let new_image = crate::image::Image::new_ref_with_id(
                                    image_id,
                                    resized_data,
                                    new_size,
                                    new_pos
                                );
                                
                                log::info!("🔍 DEBUG WINDOW: New image: size={:?}, pos={:?}",
                                         new_image.size(), new_image.position());
                                
                                // Directly replace in the document
                                let replaced = self.document.replace_image_by_id(image_id, new_image);
                                log::info!("🔍 DEBUG WINDOW: Direct replacement {}", 
                                         if replaced { "SUCCEEDED" } else { "FAILED" });
                                
                                // Force document modification and redraw
                                for _ in 0..10 {
                                    self.document.mark_modified();
                                }
                                self.last_rendered_version = 0;
                                self.renderer.reset_state();
                                ctx.request_repaint();
                            }
                        }
                    }
                    
                    // Add a button that directly resizes the first image without relying on selection
                    if ui.button("🔄 Directly Resize First Image (60% size)").clicked() {
                        if !self.document.images().is_empty() {
                            let original_image = &self.document.images()[0];
                            let image_id = original_image.id();
                            let image_data = original_image.data().to_vec();
                            let original_size = original_image.size();
                            let original_pos = original_image.position();
                            
                            log::info!("🔄 DIRECT RESIZE: Original image: ID={}, size={:?}, pos={:?}",
                                     image_id, original_size, original_pos);
                            
                            // Create a completely new image with different dimensions but keep the data
                            let new_size = original_size * 0.6; // 60% size
                            let new_pos = original_pos;
                            
                            // Resize the image data to match the new dimensions
                            let original_width = original_size.x as usize;
                            let original_height = original_size.y as usize;
                            let new_width = new_size.x as usize;
                            let new_height = new_size.y as usize;
                            
                            log::info!("🔄 DIRECT RESIZE: Resizing image data from {}x{} to {}x{}", 
                                     original_width, original_height, new_width, new_height);
                            
                            let resized_data = Self::resize_image_data(
                                &image_data, 
                                original_width, 
                                original_height, 
                                new_width, 
                                new_height
                            );
                            
                            log::info!("🔄 DIRECT RESIZE: Resized data length: {} (expected: {})", 
                                     resized_data.len(), new_width * new_height * 4);
                            
                            // Create a properly sized image with resized data
                            let new_image = crate::image::Image::new_ref_with_id(
                                image_id,
                                resized_data,
                                new_size,
                                new_pos
                            );
                            
                            log::info!("🔄 DIRECT RESIZE: New image: size={:?}, pos={:?}",
                                     new_image.size(), new_image.position());
                            
                            // Directly replace in the document
                            let replaced = self.document.replace_image_by_id(image_id, new_image);
                            log::info!("🔄 DIRECT RESIZE: Replacement {}", 
                                     if replaced { "SUCCEEDED" } else { "FAILED" });
                            
                            // Force document modification and redraw
                            for _ in 0..10 {
                                self.document.mark_modified();
                            }
                            self.last_rendered_version = 0;
                            self.renderer.reset_state();
                            ctx.request_repaint();
                        } else {
                            log::info!("🔄 DIRECT RESIZE: No images to resize");
                        }
                    }
                    
                    // Add a button to directly select the first image
                    if ui.button("🎯 Select First Image").clicked() {
                        self.debug_select_image_by_index(0);
                        ctx.request_repaint();
                    }
                    
                    // Add a button to translate the selected image 30px left and 30px up (direct replacement)
                    if ui.button("⬅️⬆️ Translate Image (-30px, -30px)").clicked() {
                        // Try to get an image to translate
                        let mut image_id = None;
                        let mut image_data = Vec::new();
                        let mut image_size = egui::Vec2::ZERO;
                        let mut image_position = egui::Pos2::ZERO;
                        
                        // First check if there's a selected image
                        if let Some(element) = self.get_first_selected_element() {
                            if let Some(img) = element.as_image() {
                                image_id = Some(img.id());
                                image_data = img.data().to_vec();
                                image_size = img.size();
                                image_position = img.position();
                            }
                        }
                        
                        // If no selected image, use the first image in the document
                        if image_id.is_none() && !self.document.images().is_empty() {
                            let img = &self.document.images()[0];
                            image_id = Some(img.id());
                            image_data = img.data().to_vec();
                            image_size = img.size();
                            image_position = img.position();
                        }
                        
                        // If we found an image, translate it
                        if let Some(id) = image_id {
                            log::info!("⬅️⬆️ TRANSLATE: Original image: ID={}, size={:?}, pos={:?}",
                                     id, image_size, image_position);
                            
                            // Create a new position 30px left and 30px up
                            let new_pos = image_position - egui::vec2(30.0, 30.0);
                            
                            log::info!("⬅️⬆️ TRANSLATE: New position: {:?}", new_pos);
                            
                            // Create a TranslateImage command that properly handles undo/redo
                            let command = crate::command::Command::TranslateImage {
                                image_id: id,
                                old_position: image_position,
                                new_position: new_pos,
                                image_data: image_data,
                                image_size: image_size,
                            };
                            
                            // Execute the command (this will handle the translation and record it for undo/redo)
                            self.execute_command(command);
                            
                            // Force document modification and redraw
                            for _ in 0..10 {
                                self.document.mark_modified();
                            }
                            self.last_rendered_version = 0;
                            self.renderer.reset_state();
                            
                            // Clear the selection
                            self.state = self.state.update_selection(|_| Vec::new());
                            
                            log::info!("⬅️⬆️ TRANSLATE COMMAND: Translated image {} by [-30.0, -30.0]", id);
                            ctx.request_repaint();
                        } else {
                            log::info!("⬅️⬆️ TRANSLATE: No image to translate");
                        }
                    }
                    
                    // Add a button to translate the selected image using the command system (supports undo/redo)
                    if ui.button("⬅️⬆️ Translate Image with Command").clicked() {
                        // Try to get an image to translate
                        let mut image_id = None;
                        let mut image_data = Vec::new();
                        let mut image_size = egui::Vec2::ZERO;
                        let mut image_position = egui::Pos2::ZERO;
                        
                        // First check if there's a selected image
                        if let Some(element) = self.get_first_selected_element() {
                            if let Some(img) = element.as_image() {
                                image_id = Some(img.id());
                                image_data = img.data().to_vec();
                                image_size = img.size();
                                image_position = img.position();
                            }
                        }
                        
                        // If no selected image, use the first image in the document
                        if image_id.is_none() && !self.document.images().is_empty() {
                            let img = &self.document.images()[0];
                            image_id = Some(img.id());
                            image_data = img.data().to_vec();
                            image_size = img.size();
                            image_position = img.position();
                        }
                        
                        // If we found an image, translate it
                        if let Some(id) = image_id {
                            log::info!("⬅️⬆️ TRANSLATE COMMAND: Original image: ID={}, size={:?}, pos={:?}",
                                     id, image_size, image_position);
                            
                            // Create a new position 30px left and 30px up
                            let new_pos = image_position - egui::vec2(30.0, 30.0);
                            
                            log::info!("⬅️⬆️ TRANSLATE COMMAND: New position: {:?}", new_pos);
                            
                            // Create a TranslateImage command that properly handles undo/redo
                            let command = crate::command::Command::TranslateImage {
                                image_id: id,
                                old_position: image_position,
                                new_position: new_pos,
                                image_data: image_data,
                                image_size: image_size,
                            };
                            
                            // Execute the command (this will handle the translation and record it for undo/redo)
                            self.execute_command(command);
                            
                            // Force document modification and redraw
                            for _ in 0..10 {
                                self.document.mark_modified();
                            }
                            self.last_rendered_version = 0;
                            self.renderer.reset_state();
                            
                            // Clear the selection
                            self.state = self.state.update_selection(|_| Vec::new());
                            
                            log::info!("⬅️⬆️ TRANSLATE COMMAND: Translated image {} by [-30.0, -30.0]", id);
                            ctx.request_repaint();
                        } else {
                            log::info!("⬅️⬆️ TRANSLATE COMMAND: No image to translate");
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