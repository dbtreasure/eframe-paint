use crate::command::Command;
use crate::renderer::Renderer;
use crate::state::EditorModel;
use crate::tools::{Tool, ToolConfig};
use crate::tools::draw_stroke_helper::DrawStrokeHelper;
use egui::{Color32, Pos2, Ui};
use log::info;
use std::any::Any;
use std::fmt;
use std::cell::RefCell;
// Use web-time instead of std::time for cross-platform compatibility
use web_time::Instant;

// Config for DrawStrokeTool
#[derive(Clone)]
pub struct DrawStrokeConfig {
    pub color: Color32,
    pub thickness: f32,
}

impl ToolConfig for DrawStrokeConfig {
    fn tool_name(&self) -> &'static str {
        "Draw Stroke"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// State enum for the DrawStrokeTool
#[derive(Clone)]
pub enum DrawStrokeState {
    Idle,
    Drawing { 
        stroke: DrawStrokeHelper,
        start_time: Instant, // Using web_time::Instant for WASM compatibility
    },
}

// Manual Debug implementation for DrawStrokeState
impl fmt::Debug for DrawStrokeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Drawing { stroke, start_time } => f
                .debug_struct("Drawing")
                .field("stroke_points", &stroke.points().len())
                .field("duration_ms", &start_time.elapsed().as_millis())
                .finish(),
        }
    }
}

// New consolidated DrawStrokeTool struct
#[derive(Debug, Clone)]
pub struct UnifiedDrawStrokeTool {
    pub state: DrawStrokeState,
    pub default_color: Color32,
    pub default_thickness: f32,
}

impl UnifiedDrawStrokeTool {
    pub fn new() -> Self {
        Self {
            state: DrawStrokeState::Idle,
            default_color: Color32::BLACK,
            default_thickness: 2.0,
        }
    }

    pub fn start_drawing(&mut self, pos: Pos2, color: Color32, thickness: f32) {
        info!("start_drawing called at position: {:?}", pos);

        let mut stroke = DrawStrokeHelper::new(color, thickness);
        stroke.add_point(pos);

        self.state = DrawStrokeState::Drawing { 
            stroke,
            start_time: Instant::now(),
        };

        info!(
            "Successfully created Drawing state with initial point at {:?}",
            pos
        );
    }

    pub fn add_point(&mut self, pos: Pos2) {
        if let DrawStrokeState::Drawing { stroke, .. } = &mut self.state {
            info!("add_point called with position: {:?}", pos);
            stroke.add_point(pos);
        }
    }

    pub fn finish_drawing(&mut self) -> Option<Command> {
        info!("finish_drawing called");

        if let DrawStrokeState::Drawing { stroke, .. } = &self.state {
            // Only finish if we have at least 2 points
            if stroke.points().len() >= 2 {
                // Get the stroke data
                let id = crate::id_generator::generate_id();
                let points = stroke.points().to_vec();
                let color = stroke.color();
                let thickness = stroke.thickness();

                // Create a stroke element using the element factory
                let element = crate::element::factory::create_stroke(id, points.clone(), thickness, color);

                // Create the command using the unified AddElement variant
                let command = Command::AddElement { element };

                // Reset to Idle state
                self.state = DrawStrokeState::Idle;

                info!(
                    "Successfully finished stroke with ID {} and {} points, generated command",
                    id,
                    points.len()
                );
                return Some(command);
            }
        }

        // If we can't finish (not in Drawing state or not enough points), just reset
        self.state = DrawStrokeState::Idle;
        info!("Reset to Idle state without generating command");
        None
    }

    // Get the current state name
    pub fn current_state_name(&self) -> &'static str {
        match self.state {
            DrawStrokeState::Idle => "Idle",
            DrawStrokeState::Drawing { .. } => "Drawing",
        }
    }
}

impl Tool for UnifiedDrawStrokeTool {
    fn name(&self) -> &'static str {
        "Draw Stroke"
    }

    fn activate(&mut self, _editor_model: &EditorModel) {
        // Reset to Idle state when activated
        self.state = DrawStrokeState::Idle;
        info!("DrawStrokeTool activated and reset to Idle state");
    }

    fn deactivate(&mut self, _editor_model: &EditorModel) {
        // Reset to Idle state when deactivated
        self.state = DrawStrokeState::Idle;
        info!("DrawStrokeTool deactivated and reset to Idle state");
    }

    fn on_pointer_down(
        &mut self, 
        pos: Pos2,
        button: egui::PointerButton,
        modifiers: &egui::Modifiers,
        editor_model: &EditorModel,
        _renderer: &mut Renderer,  // Add renderer parameter but don't use it
    ) -> Option<Command> {
        info!(
            "DrawStrokeTool::on_pointer_down called at position: {:?} with button: {:?}",
            pos, button
        );

        // Only handle primary button for drawing
        if button != egui::PointerButton::Primary {
            return None;
        }

        // Determine stroke color and thickness based on tool settings and modifiers
        let mut color = self.default_color;
        let mut thickness = self.default_thickness;

        // Example modifier: Shift for alternate color (red)
        if modifiers.shift {
            color = Color32::RED;
        }

        // Example modifier: Ctrl for thicker stroke
        if modifiers.ctrl {
            thickness *= 2.0;
        }

        match self.state {
            DrawStrokeState::Idle => {
                // Start drawing with potentially modified color/thickness
                self.start_drawing(pos, color, thickness);
                None
            }
            DrawStrokeState::Drawing { .. } => {
                // Already drawing, add a point
                self.add_point(pos);
                None
            }
        }
    }

    fn on_pointer_move(
        &mut self, 
        pos: Pos2,
        held_buttons: &[egui::PointerButton],
        modifiers: &egui::Modifiers,
        editor_model: &mut EditorModel,
        ui: &egui::Ui,
        renderer: &mut Renderer
    ) -> Option<Command> {
        // Only continue if primary button is held
        if !held_buttons.contains(&egui::PointerButton::Primary) {
            return None;
        }

        match &mut self.state {
            DrawStrokeState::Drawing { stroke, .. } => {
                // Add the point to the stroke
                stroke.add_point(pos);
                
                // No need to call update_preview here as it will be called by the app
                // after handling input events
                None
            }
            _ => None,
        }
    }

    fn on_pointer_up(
        &mut self, 
        pos: Pos2,
        button: egui::PointerButton,
        modifiers: &egui::Modifiers,
        editor_model: &EditorModel
    ) -> Option<Command> {
        info!(
            "DrawStrokeTool::on_pointer_up called at position: {:?} with button: {:?}",
            pos, button
        );

        // Only handle primary button for drawing
        if button != egui::PointerButton::Primary {
            return None;
        }

        match self.state {
            DrawStrokeState::Idle => None,
            DrawStrokeState::Drawing { .. } => {
                // Add the final point and finish the stroke
                self.add_point(pos);
                self.finish_drawing()
            }
        }
    }

    fn reset_interaction_state(&mut self) {
        self.state = DrawStrokeState::Idle;
        info!("Reset interaction state to Idle");
    }

    fn update_preview(&mut self, renderer: &mut Renderer) {
        match &self.state {
            DrawStrokeState::Idle => {
                // No preview in Idle state
                renderer.clear_stroke_preview();
            }
            DrawStrokeState::Drawing { stroke, .. } => {
                // Use the new renderer methods directly instead of creating a StrokePreview
                renderer.set_stroke_preview(
                    stroke.points().to_vec(),
                    stroke.thickness(),
                    stroke.color()
                );
                info!("Updated stroke preview with {} points", stroke.points().len());
            }
        }
    }

    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.clear_stroke_preview();
        info!("Cleared stroke preview");
    }

    fn on_key_event(
        &mut self,
        key: egui::Key,
        pressed: bool,
        modifiers: &egui::Modifiers,
        editor_model: &EditorModel
    ) -> Option<Command> {
        // Only handle key press events (not releases)
        if !pressed {
            return None;
        }

        // Add keyboard shortcuts for adjusting stroke properties
        match key {
            egui::Key::ArrowUp if modifiers.ctrl => {
                // Increase stroke thickness
                self.default_thickness = (self.default_thickness + 1.0).min(20.0);
                info!("Increased stroke thickness to {}", self.default_thickness);
                None
            }
            egui::Key::ArrowDown if modifiers.ctrl => {
                // Decrease stroke thickness
                self.default_thickness = (self.default_thickness - 1.0).max(1.0);
                info!("Decreased stroke thickness to {}", self.default_thickness);
                None
            }
            // Add more shortcuts as needed
            _ => None,
        }
    }

    fn ui(&mut self, ui: &mut Ui, _editor_model: &EditorModel) -> Option<Command> {
        match &self.state {
            DrawStrokeState::Idle => {
                ui.label("Drawing Tool Settings:");

                // Color picker
                ui.horizontal(|ui| {
                    ui.label("Stroke color:");
                    ui.color_edit_button_srgba(&mut self.default_color);
                });

                // Thickness slider
                ui.horizontal(|ui| {
                    ui.label("Thickness:");
                    ui.add(egui::Slider::new(&mut self.default_thickness, 1.0..=20.0).text("px"));
                });

                ui.separator();
                ui.label("Use the mouse to draw on the canvas.");
                
                // Display keyboard shortcuts
                ui.separator();
                ui.label("Keyboard Shortcuts:");
                ui.label("• Shift + Click: Draw with red color");
                ui.label("• Ctrl + Click: Double stroke thickness");
                ui.label("• Ctrl + ↑: Increase thickness");
                ui.label("• Ctrl + ↓: Decrease thickness");
            }
            DrawStrokeState::Drawing { stroke, start_time } => {
                ui.label("Currently drawing...");
                
                // Show duration
                let duration = start_time.elapsed();
                ui.label(format!("Drawing for: {:.1}s", duration.as_secs_f32()));
                
                // Show point count
                ui.label(format!("Points: {}", stroke.points().len()));
                
                // Show current stroke properties
                ui.label(format!("Color: {:?}", stroke.color()));
                ui.label(format!("Thickness: {:.1}px", stroke.thickness()));
            }
        }

        None // No immediate command from UI
    }

    fn get_config(&self) -> Box<dyn ToolConfig> {
        Box::new(DrawStrokeConfig {
            color: self.default_color,
            thickness: self.default_thickness,
        })
    }

    fn apply_config(&mut self, config: &dyn ToolConfig) {
        if let Some(config) = config.as_any().downcast_ref::<DrawStrokeConfig>() {
            self.default_color = config.color;
            self.default_thickness = config.thickness;
        }
    }
}

impl Default for UnifiedDrawStrokeTool {
    fn default() -> Self {
        Self::new()
    }
}

// Factory function to create a new DrawStrokeTool
pub fn new_draw_stroke_tool() -> UnifiedDrawStrokeTool {
    UnifiedDrawStrokeTool::new()
}
