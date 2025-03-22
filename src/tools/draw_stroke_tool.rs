use crate::command::Command;
use crate::renderer::Renderer;
use crate::state::EditorModel;
use crate::tools::{Tool, ToolConfig};
use crate::tools::draw_stroke_helper::DrawStrokeHelper;
use egui::{Color32, Pos2, Ui};
use log::info;
use std::any::Any;
use std::fmt;

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
    Ready,
    Drawing { stroke: DrawStrokeHelper },
}

// Manual Debug implementation for DrawStrokeState
impl fmt::Debug for DrawStrokeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ready => write!(f, "Ready"),
            Self::Drawing { stroke } => f
                .debug_struct("Drawing")
                .field("stroke_points", &stroke.points().len())
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
            state: DrawStrokeState::Ready,
            default_color: Color32::BLACK,
            default_thickness: 2.0,
        }
    }

    pub fn start_drawing(&mut self, pos: Pos2) {
        info!("start_drawing called at position: {:?}", pos);

        let mut stroke = DrawStrokeHelper::new(self.default_color, self.default_thickness);
        stroke.add_point(pos);

        self.state = DrawStrokeState::Drawing { stroke };

        info!(
            "Successfully created Drawing state with initial point at {:?}",
            pos
        );
    }

    pub fn add_point(&mut self, pos: Pos2) {
        if let DrawStrokeState::Drawing { stroke } = &mut self.state {
            info!("add_point called with position: {:?}", pos);
            stroke.add_point(pos);
        }
    }

    pub fn finish_drawing(&mut self) -> Option<Command> {
        info!("finish_drawing called");

        if let DrawStrokeState::Drawing { stroke } = &self.state {
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

                // Reset to Ready state
                self.state = DrawStrokeState::Ready;

                info!(
                    "Successfully finished stroke with ID {} and {} points, generated command",
                    id,
                    points.len()
                );
                return Some(command);
            }
        }

        // If we can't finish (not in Drawing state or not enough points), just reset
        self.state = DrawStrokeState::Ready;
        info!("Reset to Ready state without generating command");
        None
    }

    // Get the current state name
    pub fn current_state_name(&self) -> &'static str {
        match self.state {
            DrawStrokeState::Ready => "Ready",
            DrawStrokeState::Drawing { .. } => "Drawing",
        }
    }
}

impl Tool for UnifiedDrawStrokeTool {
    fn name(&self) -> &'static str {
        "Draw Stroke"
    }

    fn activate(&mut self, _editor_model: &EditorModel) {
        // Reset to Ready state when activated
        self.state = DrawStrokeState::Ready;
        info!("DrawStrokeTool activated and reset to Ready state");
    }

    fn deactivate(&mut self, _editor_model: &EditorModel) {
        // Reset to Ready state when deactivated
        self.state = DrawStrokeState::Ready;
        info!("DrawStrokeTool deactivated and reset to Ready state");
    }

    fn on_pointer_down(&mut self, pos: Pos2, _editor_model: &EditorModel) -> Option<Command> {
        info!(
            "DrawStrokeTool::on_pointer_down called at position: {:?}",
            pos
        );

        match self.state {
            DrawStrokeState::Ready => {
                // Start drawing
                self.start_drawing(pos);
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
        _editor_model: &mut EditorModel,
        _ui: &egui::Ui,
        _renderer: &mut Renderer,
    ) -> Option<Command> {
        match &mut self.state {
            DrawStrokeState::Drawing { stroke } => {
                // Add the point to the stroke
                stroke.add_point(pos);
                self.update_preview(_renderer);
                None
            }
            _ => None,
        }
    }

    fn on_pointer_up(&mut self, pos: Pos2, _editor_model: &EditorModel) -> Option<Command> {
        info!(
            "DrawStrokeTool::on_pointer_up called at position: {:?}",
            pos
        );

        match self.state {
            DrawStrokeState::Ready => None,
            DrawStrokeState::Drawing { .. } => {
                // Add the final point and finish the stroke
                self.add_point(pos);
                self.finish_drawing()
            }
        }
    }

    fn update_preview(&mut self, renderer: &mut Renderer) {
        match &self.state {
            DrawStrokeState::Ready => {
                // No preview in Ready state
                renderer.set_preview_stroke(None);
            }
            DrawStrokeState::Drawing { stroke } => {
                // Create a preview stroke from the current points
                let preview = stroke.to_stroke_preview();
                renderer.set_preview_stroke(Some(preview));
                info!("Preview stroke has points");
            }
        }
    }

    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_preview_stroke(None);
    }

    fn ui(&mut self, ui: &mut Ui, _editor_model: &EditorModel) -> Option<Command> {
        match self.state {
            DrawStrokeState::Ready => {
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
            }
            DrawStrokeState::Drawing { .. } => {
                ui.label("Currently drawing...");
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
