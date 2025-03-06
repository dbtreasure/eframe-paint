use egui::{Pos2, Ui, Color32};
use crate::stroke::MutableStroke;
use crate::command::Command;
use crate::document::Document;
use crate::tools::{Tool, ToolConfig};
use crate::renderer::Renderer;
use crate::state::EditorState;
use std::fmt;
use std::any::Any;

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

// State type definitions
#[derive(Clone, Debug)]
pub struct Ready;

// Custom implementation of Debug for Drawing since MutableStroke doesn't implement Debug
#[derive(Clone)]
pub struct Drawing {
    stroke: MutableStroke,
}

// Implement Debug manually for Drawing
impl fmt::Debug for Drawing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Drawing")
            .field("stroke_points", &self.stroke.points().len())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct DrawStrokeTool<State = Ready> {
    state: State,
    default_color: Color32,
    default_thickness: f32,
}

impl DrawStrokeTool<Ready> {
    pub fn new() -> Self {
        Self {
            state: Ready,
            default_color: Color32::BLACK,
            default_thickness: 2.0,
        }
    }

    pub fn start_drawing(self, pos: Pos2) -> Result<DrawStrokeTool<Drawing>, Self> {
        if self.can_transition() {
            let mut stroke = MutableStroke::new(self.default_color, self.default_thickness);
            stroke.add_point(pos);
            Ok(DrawStrokeTool {
                state: Drawing {
                    stroke,
                },
                default_color: self.default_color,
                default_thickness: self.default_thickness,
            })
        } else {
            Err(self)
        }
    }
    
    fn can_transition(&self) -> bool {
        // Ready state can always transition to Drawing
        true
    }
    
    // Get the current color
    pub fn color(&self) -> Color32 {
        self.default_color
    }
    
    // Get the current thickness
    pub fn thickness(&self) -> f32 {
        self.default_thickness
    }
    
    // Set the color
    pub fn set_color(&mut self, color: Color32) {
        self.default_color = color;
    }
    
    // Set the thickness
    pub fn set_thickness(&mut self, thickness: f32) {
        self.default_thickness = thickness;
    }

    pub fn restore_state(&mut self, other: &DrawStrokeToolType) {
        if let DrawStrokeToolType::Ready(other_tool) = other {
            self.default_color = other_tool.color();
            self.default_thickness = other_tool.thickness();
        }
    }
}

impl DrawStrokeTool<Drawing> {
    pub fn add_point(&mut self, pos: Pos2) {
        self.state.stroke.add_point(pos);
    }

    pub fn finish(self) -> Result<(Command, DrawStrokeTool<Ready>), Self> {
        if self.can_transition() {
            // Extract the stroke from the state to consume it
            let stroke = self.state.stroke;
            
            // Use into_stroke_ref instead of to_stroke_ref to avoid cloning
            let command = Command::AddStroke(stroke.into_stroke_ref());
            
            let new_tool = DrawStrokeTool {
                state: Ready,
                default_color: self.default_color,
                default_thickness: self.default_thickness,
            };
            Ok((command, new_tool))
        } else {
            Err(self)
        }
    }
    
    pub fn finish_with_point(mut self, pos: Pos2) -> Result<(Command, DrawStrokeTool<Ready>), Self> {
        // Add the final point
        self.state.stroke.add_point(pos);
        
        // Then finish normally
        self.finish()
    }
    
    fn can_transition(&self) -> bool {
        // For now, all transitions from Drawing state are valid
        // In a real implementation, we might check if the stroke has enough points
        self.state.stroke.points().len() >= 2
    }
    
    // Get the current color
    pub fn color(&self) -> Color32 {
        self.default_color
    }
    
    // Get the current thickness
    pub fn thickness(&self) -> f32 {
        self.default_thickness
    }
    
    // Get a reference to the current stroke
    pub fn stroke(&self) -> &MutableStroke {
        &self.state.stroke
    }
}

impl Default for DrawStrokeTool<Drawing> {
    fn default() -> Self {
        Self {
            state: Drawing {
                stroke: MutableStroke::new(Color32::BLACK, 1.0)
            },
            default_color: Color32::BLACK,
            default_thickness: 1.0,
        }
    }
}

// Implement Tool for Ready state
impl Tool for DrawStrokeTool<Ready> {
    fn name(&self) -> &'static str { 
        "Draw Stroke" 
    }

    fn activate(&mut self, _doc: &Document) {
        // No-op, already in ready state
    }
    
    fn deactivate(&mut self, _doc: &Document) {
        // No-op for Ready state
    }
    
    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // We can't directly change self's type, so we'll use the wrapper enum
        // This will be handled by the DrawStrokeToolType wrapper
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // No-op in Ready state
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // No-op in Ready state
        None
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        // No preview in Ready state
        renderer.set_preview_stroke(None);
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_preview_stroke(None);
    }
    
    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
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
        
        None  // No immediate command from UI
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

impl Default for DrawStrokeTool<Ready> {
    fn default() -> Self {
        Self::new() // Reuse existing constructor
    }
}

// Implement Tool for Drawing state
impl Tool for DrawStrokeTool<Drawing> {
    fn name(&self) -> &'static str { 
        "Draw Stroke" 
    }
    
    fn activate(&mut self, _doc: &Document) {
        // No-op, already active
    }
    
    fn deactivate(&mut self, _doc: &Document) {
        // No-op, handled by the wrapper
    }
    
    fn on_pointer_down(&mut self, pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        self.add_point(pos);
        None
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        self.add_point(pos);
        None
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        self.add_point(pos);
        // We can't change self's type here, so this is handled by the wrapper
        None
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        let preview = self.stroke().to_stroke_ref();
        renderer.set_preview_stroke(Some(preview));
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_preview_stroke(None);
    }
    
    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Currently drawing...");
        None
    }
    
    fn get_config(&self) -> Box<dyn ToolConfig> {
        Box::new(DrawStrokeConfig {
            color: self.stroke().color(),
            thickness: self.stroke().thickness(),
        })
    }
    
    fn apply_config(&mut self, _config: &dyn ToolConfig) {
        // Cannot change config while drawing
    }
}

// Wrapper enum to handle state transitions
#[derive(Clone)]
pub enum DrawStrokeToolType {
    Ready(DrawStrokeTool<Ready>),
    Drawing(DrawStrokeTool<Drawing>),
}

impl Tool for DrawStrokeToolType {
    fn name(&self) -> &'static str {
        match self {
            Self::Ready(tool) => tool.name(),
            Self::Drawing(tool) => tool.name(),
        }
    }
    
    fn activate(&mut self, doc: &Document) {
        match self {
            Self::Ready(tool) => tool.activate(doc),
            Self::Drawing(tool) => tool.activate(doc),
        }
    }
    
    fn deactivate(&mut self, doc: &Document) {
        // If we're in the drawing state, we need to ensure we're back in ready state
        self.ensure_ready_state();
        
        match self {
            Self::Ready(tool) => tool.deactivate(doc),
            Self::Drawing(_) => unreachable!("Should be in Ready state after ensure_ready_state"),
        }
    }
    
    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::Ready(tool) => {
                // Try to transition to Drawing state
                if let Ok(drawing_tool) = tool.clone().start_drawing(pos) {
                    *self = Self::Drawing(drawing_tool);
                    None
                } else {
                    // Couldn't transition, stay in Ready state
                    None
                }
            }
            Self::Drawing(tool) => tool.on_pointer_down(pos, doc, state),
        }
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::Ready(_) => None,
            Self::Drawing(tool) => tool.on_pointer_move(pos, doc, state),
        }
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::Ready(_) => None,
            Self::Drawing(tool) => {
                // Try to finish drawing and transition back to Ready state
                let mut drawing_tool = tool.clone();
                
                // Add the final point
                drawing_tool.add_point(pos);
                
                // Try to finish the stroke
                match drawing_tool.finish() {
                    Ok((command, ready_tool)) => {
                        *self = Self::Ready(ready_tool);
                        Some(command)
                    }
                    Err(drawing_tool) => {
                        // Couldn't finish, stay in Drawing state
                        *self = Self::Drawing(drawing_tool);
                        None
                    }
                }
            }
        }
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::Ready(tool) => tool.update_preview(renderer),
            Self::Drawing(tool) => tool.update_preview(renderer),
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::Ready(tool) => tool.clear_preview(renderer),
            Self::Drawing(tool) => tool.clear_preview(renderer),
        }
    }
    
    fn ui(&mut self, ui: &mut Ui, doc: &Document) -> Option<Command> {
        match self {
            Self::Ready(tool) => tool.ui(ui, doc),
            Self::Drawing(tool) => tool.ui(ui, doc),
        }
    }
    
    fn get_config(&self) -> Box<dyn ToolConfig> {
        match self {
            Self::Ready(tool) => tool.get_config(),
            Self::Drawing(tool) => tool.get_config(),
        }
    }
    
    fn apply_config(&mut self, config: &dyn ToolConfig) {
        match self {
            Self::Ready(tool) => tool.apply_config(config),
            Self::Drawing(tool) => tool.apply_config(config),
        }
    }
}

impl DrawStrokeToolType {
    // Add these helper methods
    
    /// Returns the current state name as a string
    pub fn current_state_name(&self) -> &'static str {
        match self {
            Self::Ready(_) => "Ready",
            Self::Drawing(_) => "Drawing",
        }
    }
    
    /// Ensures the tool is in the Ready state, transitioning if necessary
    pub fn ensure_ready_state(&mut self) {
        if let Self::Drawing(_drawing_tool) = self {
            // Force transition to Ready state, discarding any in-progress drawing
            let default_tool = DrawStrokeTool::<Ready>::default();
            *self = Self::Ready(default_tool);
        }
    }
    
    /// Check if this tool can transition to another state
    pub fn can_transition(&self) -> bool {
        match self {
            Self::Ready(_) => true, // Ready can always transition to Drawing
            Self::Drawing(drawing_tool) => {
                // A drawing tool can only transition to Ready if it has at least 2 points
                // We'll use the can_transition method on the Drawing tool
                drawing_tool.can_transition()
            }
        }
    }
    
    /// Restore state from another tool instance
    pub fn restore_state(&mut self, other: &Self) {
        match (self, other) {
            // Only restore if both are in Ready state
            (Self::Ready(self_tool), Self::Ready(other_tool)) => {
                // Copy color and thickness settings
                self_tool.set_color(other_tool.color());
                self_tool.set_thickness(other_tool.thickness());
            },
            // We don't restore Drawing state as it's an active operation
            _ => {},
        }
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> Box<dyn ToolConfig> {
        match self {
            Self::Ready(tool) => {
                Box::new(DrawStrokeConfig {
                    color: tool.color(),
                    thickness: tool.thickness(),
                })
            },
            Self::Drawing(tool) => {
                Box::new(DrawStrokeConfig {
                    color: tool.color(),
                    thickness: tool.thickness(),
                })
            },
        }
    }
    
    /// Apply a configuration
    pub fn apply_config(&mut self, config: &dyn ToolConfig) {
        if let Some(cfg) = config.as_any().downcast_ref::<DrawStrokeConfig>() {
            match self {
                Self::Ready(tool) => {
                    tool.set_color(cfg.color);
                    tool.set_thickness(cfg.thickness);
                },
                Self::Drawing(_) => {
                    // Cannot apply config while drawing
                    // Will be applied when returning to Ready state
                },
            }
        }
    }
}

// Factory function to create a new DrawStrokeToolType
pub fn new_draw_stroke_tool() -> DrawStrokeToolType {
    DrawStrokeToolType::Ready(DrawStrokeTool::new())
} 