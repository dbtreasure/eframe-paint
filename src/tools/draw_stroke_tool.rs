use egui::{Pos2, Ui, Color32};
use crate::stroke::MutableStroke;
use crate::command::Command;
use crate::document::Document;
use crate::tools::Tool;
use crate::renderer::Renderer;

// State type definitions
#[derive(Clone)]
pub struct Ready;

#[derive(Clone)]
pub struct Drawing {
    stroke: MutableStroke,
}

#[derive(Clone)]
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

    pub fn start_drawing(self, pos: Pos2) -> DrawStrokeTool<Drawing> {
        let mut stroke = MutableStroke::new(self.default_color, self.default_thickness);
        stroke.add_point(pos);
        DrawStrokeTool {
            state: Drawing {
                stroke,
            },
            default_color: self.default_color,
            default_thickness: self.default_thickness,
        }
    }
}

impl DrawStrokeTool<Drawing> {
    pub fn add_point(&mut self, pos: Pos2) {
        self.state.stroke.add_point(pos);
    }

    pub fn finish(self) -> (Command, DrawStrokeTool<Ready>) {
        // Extract the stroke from the state to consume it
        let stroke = self.state.stroke;
        
        // Use into_stroke_ref instead of to_stroke_ref to avoid cloning
        let command = Command::AddStroke(stroke.into_stroke_ref());
        
        let new_tool = DrawStrokeTool {
            state: Ready,
            default_color: self.default_color,
            default_thickness: self.default_thickness,
        };
        (command, new_tool)
    }
    
    pub fn finish_with_point(mut self, pos: Pos2) -> (Command, DrawStrokeTool<Ready>) {
        // Add the final point
        self.state.stroke.add_point(pos);
        
        // Then finish normally
        self.finish()
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
    
    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // We can't directly change self's type, so we'll use the wrapper enum
        // This will be handled by the DrawStrokeToolType wrapper
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // No-op in Ready state
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
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
}

impl Default for DrawStrokeTool<Ready> {
    fn default() -> Self {
        Self::new() // Reuse existing constructor
    }
}

// Implement Tool for Drawing state
impl Tool for DrawStrokeTool<Drawing> {
    fn name(&self) -> &'static str { 
        "Drawing Stroke" 
    }
    
    fn activate(&mut self, _doc: &Document) {
        // This shouldn't happen, but it's handled by the wrapper
    }
    
    fn deactivate(&mut self, _doc: &Document) {
        // This is handled by the wrapper
    }

    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // Already drawing, ignore additional pointer down events
        None
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, _doc: &Document) -> Option<Command> {
        self.add_point(pos);
        None
    }

    fn on_pointer_up(&mut self, pos: Pos2, _doc: &Document) -> Option<Command> {
        self.add_point(pos);
        // State transition is handled by the wrapper
        None
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        let preview = self.state.stroke.to_stroke_ref();
        renderer.set_preview_stroke(Some(preview));
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        renderer.set_preview_stroke(None);
    }
    
    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Currently drawing a stroke...");
        ui.label("Release the mouse button to finish.");
        None
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
            DrawStrokeToolType::Ready(tool) => tool.name(),
            DrawStrokeToolType::Drawing(tool) => tool.name(),
        }
    }

    fn activate(&mut self, doc: &Document) {
        // Always ensure we're in Ready state when activated
        self.ensure_ready_state();
        
        // Then call the Ready state's activate method
        if let Self::Ready(tool) = self {
            tool.activate(doc);
        }
    }
    
    fn deactivate(&mut self, doc: &Document) {
        // If we're in Drawing state, finalize the stroke but discard the command
        if let Self::Drawing(_) = self {
            // Create a new Ready tool instead of cloning and finishing
            *self = Self::Ready(DrawStrokeTool::<Ready>::default());
        }
        
        // Then call the Ready state's deactivate method
        if let Self::Ready(tool) = self {
            tool.deactivate(doc);
        }
    }
    
    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        match self {
            DrawStrokeToolType::Ready(tool) => {
                // Use std::mem::take to get ownership while leaving a default in place
                let ready_tool = std::mem::take(tool);
                let drawing_tool = ready_tool.start_drawing(pos);
                
                // Replace self with the Drawing variant
                *self = DrawStrokeToolType::Drawing(drawing_tool);
                
                None
            },
            DrawStrokeToolType::Drawing(tool) => tool.on_pointer_down(pos, doc),
        }
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        match self {
            DrawStrokeToolType::Ready(tool) => tool.on_pointer_move(pos, doc),
            DrawStrokeToolType::Drawing(tool) => tool.on_pointer_move(pos, doc),
        }
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        match self {
            DrawStrokeToolType::Ready(tool) => tool.on_pointer_up(pos, doc),
            DrawStrokeToolType::Drawing(tool) => {
                // Use std::mem::take to get ownership while leaving a default in place
                let drawing_tool = std::mem::take(tool);
                
                // Add the final point and finish
                let (command, ready_tool) = drawing_tool.finish_with_point(pos);
                
                // Replace self with the Ready variant
                *self = DrawStrokeToolType::Ready(ready_tool);
                
                Some(command)
            }
        }
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        match self {
            DrawStrokeToolType::Ready(tool) => tool.update_preview(renderer),
            DrawStrokeToolType::Drawing(tool) => tool.update_preview(renderer),
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        match self {
            DrawStrokeToolType::Ready(tool) => tool.clear_preview(renderer),
            DrawStrokeToolType::Drawing(tool) => tool.clear_preview(renderer),
        }
    }
    
    fn ui(&mut self, ui: &mut Ui, doc: &Document) -> Option<Command> {
        match self {
            DrawStrokeToolType::Ready(tool) => tool.ui(ui, doc),
            DrawStrokeToolType::Drawing(tool) => tool.ui(ui, doc),
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
        if let Self::Drawing(_) = self {
            *self = Self::Ready(DrawStrokeTool::<Ready>::default());
        }
    }
}

// Factory function to create a new DrawStrokeToolType
pub fn new_draw_stroke_tool() -> DrawStrokeToolType {
    DrawStrokeToolType::Ready(DrawStrokeTool::new())
} 