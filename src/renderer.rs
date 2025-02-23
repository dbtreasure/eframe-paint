// src/renderer.rs
use eframe::egui::{self, Color32};
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tool {
    Brush,
    Eraser,
    Selection,
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tool::Brush => write!(f, "Brush"),
            Tool::Eraser => write!(f, "Eraser"),
            Tool::Selection => write!(f, "Selection"),
        }
    }
}

#[derive(Clone)]
pub struct Renderer {
    // We'll add fields here as needed for future rendering features
    initialized: bool,
    // Add new fields for tool state
    current_tool: Tool,
    brush_color: Color32,
    brush_thickness: f32,
    ctx: egui::Context,
    selection_mode: crate::selection::SelectionMode,
    current_painter: Option<egui::Painter>,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer")
            .field("initialized", &self.initialized)
            .field("current_tool", &self.current_tool)
            .field("brush_color", &self.brush_color)
            .field("brush_thickness", &self.brush_thickness)
            .field("selection_mode", &self.selection_mode)
            .field("current_painter", &"<painter>")
            .finish()
    }
}

impl PartialEq for Renderer {
    fn eq(&self, other: &Self) -> bool {
        self.initialized == other.initialized &&
        self.current_tool == other.current_tool &&
        self.brush_color == other.brush_color &&
        self.brush_thickness == other.brush_thickness &&
        self.selection_mode == other.selection_mode
    }
}

impl Renderer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            initialized: true,
            current_tool: Tool::Brush,
            brush_color: Color32::BLACK,
            brush_thickness: 2.0,
            ctx: cc.egui_ctx.clone(),
            selection_mode: crate::selection::SelectionMode::Rectangle,
            current_painter: None,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set the current painter for this frame
    pub fn set_painter(&mut self, painter: egui::Painter) {
        self.current_painter = Some(painter);
    }

    /// Get the current painter
    pub fn painter(&self) -> Option<&egui::Painter> {
        self.current_painter.as_ref()
    }

    /// Get a mutable reference to the UI context
    pub fn ui(&mut self) -> Option<&mut egui::Ui> {
        // Note: This is a placeholder - we need to store the Ui reference somewhere
        // or get it from the current frame context
        None
    }

    // Add this helper function for consistent tool buttons
    fn tool_button(&mut self, ui: &mut egui::Ui, label: &str, is_selected: bool) -> bool {
        let button_size = egui::vec2(40.0, 40.0);
        let mut clicked = false;
        
        ui.allocate_ui_with_layout(
            button_size,
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                if ui.selectable_label(is_selected, label).clicked() {
                    clicked = true;
                }
            }
        );
        
        clicked
    }

    pub fn render_tools_panel(&mut self, ui: &mut egui::Ui, document: &mut crate::Document) {
        // Configure spacing for the entire panel
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
        ui.spacing_mut().button_padding = egui::vec2(4.0, 4.0);
        
        // Tool buttons section
        ui.vertical_centered(|ui| {
            // Brush tool
            if self.tool_button(ui, "B", self.current_tool == Tool::Brush) {
                self.set_tool(Tool::Brush);
            }
            
            // Eraser tool
            if self.tool_button(ui, "E", self.current_tool == Tool::Eraser) {
                self.set_tool(Tool::Eraser);
            }
            
            // Rectangle selection tool
            if self.tool_button(
                ui,
                "S",
                self.current_tool == Tool::Selection && self.selection_mode == crate::selection::SelectionMode::Rectangle
            ) {
                self.set_tool(Tool::Selection);
                self.selection_mode = crate::selection::SelectionMode::Rectangle;
            }
            
            // Lasso selection tool
            if self.tool_button(
                ui,
                "L",
                self.current_tool == Tool::Selection && self.selection_mode == crate::selection::SelectionMode::Freeform
            ) {
                self.set_tool(Tool::Selection);
                self.selection_mode = crate::selection::SelectionMode::Freeform;
            }
        });
        
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
        
        // Color and thickness controls section
        ui.vertical_centered(|ui| {
            // Color picker button
            let color_button_size = egui::vec2(40.0, 40.0);
            ui.allocate_ui_with_layout(
                color_button_size,
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let mut color = self.brush_color;
                    ui.spacing_mut().interact_size = color_button_size;
                    egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut color,
                        egui::color_picker::Alpha::Opaque,
                    );
                    if color != self.brush_color {
                        self.brush_color = color;
                    }
                }
            );
            
            ui.add_space(4.0);
            
            // Thickness slider - custom layout to fit in the narrow panel
            let thickness_size = egui::vec2(40.0, 100.0);
            ui.allocate_ui_with_layout(
                thickness_size,
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.brush_thickness, 1.0..=50.0)
                            .vertical()
                            .fixed_decimals(0)
                    );
                }
            );
        });
        
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
        
        // Undo/Redo section
        ui.vertical_centered(|ui| {
            let action_button_size = egui::vec2(40.0, 40.0);
            
            // Undo button
            ui.allocate_ui_with_layout(
                action_button_size,
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    if ui.button("⟲").clicked() {
                        document.undo();
                    }
                }
            );
            
            // Redo button
            ui.allocate_ui_with_layout(
                action_button_size,
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    if ui.button("⟳").clicked() {
                        document.redo();
                    }
                }
            );
        });
    }

    pub fn render(&mut self, ctx: &egui::Context, painter: &egui::Painter, rect: egui::Rect) {
        // Draw a rectangle using the current brush color and alpha
        painter.rect_filled(
            rect,
            0.0,
            self.brush_color, // Use the selected color
        );
        
        // Request continuous rendering
        ctx.request_repaint();
    }

    // Add getters and setters for the new state
    pub fn current_tool(&self) -> Tool {
        self.current_tool
    }

    pub fn set_current_tool(&mut self, tool: Tool) {
        self.current_tool = tool;
    }

    pub fn brush_color(&self) -> Color32 {
        self.brush_color
    }

    pub fn set_brush_color(&mut self, color: Color32) {
        self.brush_color = color;
    }

    pub fn brush_thickness(&self) -> f32 {
        self.brush_thickness
    }

    pub fn set_brush_thickness(&mut self, thickness: f32) {
        self.brush_thickness = thickness;
    }

    pub fn create_texture(&self, image: egui::ColorImage, name: &str) -> egui::TextureHandle {
        self.ctx.load_texture(
            name,
            image,
            egui::TextureOptions::default()
        )
    }

    pub fn selection_mode(&self) -> crate::selection::SelectionMode {
        self.selection_mode
    }

    pub fn set_selection_mode(&mut self, mode: crate::selection::SelectionMode) {
        self.selection_mode = mode;
    }

    // Add a method to check if the tool changed
    pub fn set_tool(&mut self, tool: Tool) -> bool {
        let changed = self.current_tool != tool;
        self.current_tool = tool;
        changed
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            initialized: false,
            current_tool: Tool::Brush,
            brush_color: Color32::BLUE,
            brush_thickness: 5.0,
            ctx: egui::Context::default(),
            selection_mode: crate::selection::SelectionMode::Rectangle,
            current_painter: None,
        }
    }
}