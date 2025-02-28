use eframe::egui;
use crate::command::Command;
use crate::image::Image;

use log;

#[cfg(feature = "image_support")]
use image;

pub struct FileHandler {
    dropped_files: Vec<egui::DroppedFile>,
    processed_files: Vec<String>,
}

impl FileHandler {
    pub fn new() -> Self {
        Self {
            dropped_files: Vec::new(),
            processed_files: Vec::new(),
        }
    }
    
    /// Process any newly dropped files from the UI context
    /// Returns true if any new files were processed
    pub fn check_for_dropped_files(&mut self, ctx: &egui::Context) -> bool {
        // Check for newly dropped files
        let mut new_dropped_files = false;
        
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                // Get a reference to dropped files without cloning
                self.dropped_files = i.raw.dropped_files.clone();
                new_dropped_files = true;
            }
        });
        
        new_dropped_files
    }
    
    /// Process the dropped files and return commands to execute
    pub fn process_dropped_files(&mut self, ctx: &egui::Context, central_panel_rect: egui::Rect) -> Vec<Command> {
        let mut commands = Vec::new();
        
        // Skip if we have no files to process
        if self.dropped_files.is_empty() {
            return commands;
        }
        
        // Process the files in the queue
        for file in &self.dropped_files {
            let file_name = if let Some(path) = &file.path {
                path.display().to_string()
            } else if !file.name.is_empty() {
                file.name.clone()
            } else {
                "unknown".to_owned()
            };

            // Skip if we've already processed this file
            if self.processed_files.contains(&file_name) {
                continue;
            }

            // Check if it's an image file
            if self.is_image_file(file) {
                // Process the image file
                if let Some(cmd) = self.process_image_file(file, file_name.clone(), central_panel_rect, ctx) {
                    commands.push(cmd);
                    // Add to processed files list
                    self.processed_files.push(file_name);
                }
            } else {
                log::warn!("Dropped file is not a supported type: {}", file_name);
            }
        }
        
        commands
    }
    
    /// Check if a file is an image based on MIME type or extension
    fn is_image_file(&self, file: &egui::DroppedFile) -> bool {
        if !file.mime.is_empty() {
            file.mime.starts_with("image/")
        } else if let Some(path) = &file.path {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp")
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// Process an image file and return a command to add it to the document
    fn process_image_file(
        &self, 
        file: &egui::DroppedFile, 
        file_name: String, 
        panel_rect: egui::Rect,
        ctx: &egui::Context
    ) -> Option<Command> {
        // Try to get image data
        if let Some(bytes) = &file.bytes {
            log::info!("Processing image from memory: {} ({} bytes)", file_name, bytes.len());
            self.create_image_from_bytes(bytes, panel_rect, ctx)
        } else if let Some(path) = &file.path {
            // For native platforms, we can load the file from the path
            #[cfg(not(target_arch = "wasm32"))]
            {
                log::info!("Processing image from path: {}", path.display());
                match std::fs::read(path) {
                    Ok(bytes) => self.create_image_from_bytes(&bytes, panel_rect, ctx),
                    Err(err) => {
                        log::error!("Failed to read image file: {}: {}", path.display(), err);
                        None
                    }
                }
            }
            
            // For WASM, we can't read from the filesystem
            #[cfg(target_arch = "wasm32")]
            {
                log::warn!("File path access not supported on WASM: {}", file_name);
                None
            }
        } else {
            log::warn!("Dropped file has no accessible data: {}", file_name);
            None
        }
    }
    
    /// Create an image from bytes and return a command to add it to the document
    #[cfg(feature = "image_support")]
    fn create_image_from_bytes(&self, bytes: &[u8], panel_rect: egui::Rect, ctx: &egui::Context) -> Option<Command> {
        // Try to decode the image using the image crate
        match image::load_from_memory(bytes) {
            Ok(img) => {
                log::debug!("Successfully decoded image: {}x{}", img.width(), img.height());
                
                // Convert to RGBA
                let rgba_image = img.to_rgba8();
                let width = rgba_image.width() as f32;
                let height = rgba_image.height() as f32;
                
                // Validate panel rect
                if panel_rect.width() <= 0.0 || panel_rect.height() <= 0.0 {
                    log::error!("Invalid panel rect: {:?}", panel_rect);
                    return None;
                }
                
                // Center the image in the panel
                let panel_center = panel_rect.center();
                let position = egui::pos2(
                    panel_center.x - (width / 2.0),
                    panel_center.y - (height / 2.0)
                );
                
                // Get the raw RGBA data
                let raw_data = rgba_image.into_raw();
                
                // Verify data size
                let expected_size = (width as usize) * (height as usize) * 4;
                if raw_data.len() != expected_size {
                    log::error!("Image data size mismatch: expected {} bytes, got {} bytes", 
                        expected_size, raw_data.len());
                    return None;
                }
                
                // Create an image object
                let image = Image::new_ref(
                    raw_data,
                    egui::vec2(width, height),
                    position,
                );
                
                // Create a command to add the image
                let command = Command::AddImage(image);
                
                // Request a repaint to show the image
                ctx.request_repaint();
                
                Some(command)
            },
            Err(err) => {
                log::error!("Failed to decode image: {}", err);
                None
            }
        }
    }
    
    // Placeholder implementation when image support is not enabled
    #[cfg(not(feature = "image_support"))]
    fn create_image_from_bytes(&self, _bytes: &[u8], _panel_rect: egui::Rect, _ctx: &egui::Context) -> Option<Command> {
        log::warn!("Image support is not enabled in this build");
        None
    }
    
    /// Preview files being dragged over the application
    pub fn preview_files_being_dropped(&self, ctx: &egui::Context) {
        use egui::{Align2, Color32, Id, LayerId, Order};
        
        // Check if there are any files being hovered
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            // Get information about the hovered files
            let text = ctx.input(|i| {
                let mut text = "Dropping files:\n".to_owned();
                for file in &i.raw.hovered_files {
                    if let Some(path) = &file.path {
                        text += &format!("\n{}", path.display());
                    } else {
                        text += "\n(Path not available)";
                    }
                }
                text
            });

            // Create an overlay to show the files being dropped
            let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            
            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                ctx.style().text_styles.get(&egui::TextStyle::Heading).unwrap().clone(),
                Color32::WHITE,
            );
        }
    }
    
    /// Clear all processed files
    pub fn clear_processed_files(&mut self) {
        self.dropped_files.clear();
        self.processed_files.clear();
    }
} 