use egui::{Pos2, Ui};
use crate::command::Command;
use crate::document::Document;
use crate::tools::Tool;
use crate::renderer::Renderer;
use crate::state::ElementType;

// State type definitions
#[derive(Clone)]
pub struct Active;

#[derive(Clone)]
pub struct TextureSelected {
    selected_elements: Vec<ElementType>,
}

#[derive(Clone)]
pub struct ScalingEnabled {
    selected_elements: Vec<ElementType>,
}

#[derive(Clone)]
pub struct Scaling {
    selected_elements: Vec<ElementType>,
}

#[derive(Clone)]
pub struct SelectionTool<State = Active> {
    #[allow(dead_code)]
    state: State,
}

impl SelectionTool<Active> {
    pub fn new() -> Self {
        Self { state: Active }
    }
    
    // Transition to TextureSelected state
    pub fn select_texture(self, elements: Vec<ElementType>) -> SelectionTool<TextureSelected> {
        SelectionTool { state: TextureSelected { selected_elements: elements } }
    }
}

impl SelectionTool<TextureSelected> {
    pub fn new() -> Self {
        Self { state: TextureSelected { selected_elements: Vec::new() } }
    }
    
    // Transition to Active state
    pub fn deselect_texture(self) -> SelectionTool<Active> {
        SelectionTool { state: Active }
    }
    
    // Transition to ScalingEnabled state
    pub fn enable_scaling(self) -> SelectionTool<ScalingEnabled> {
        SelectionTool { state: ScalingEnabled { selected_elements: self.state.selected_elements } }
    }
    
    // Update selected elements
    pub fn update_selected_elements(&mut self, elements: Vec<ElementType>) {
        self.state.selected_elements = elements;
    }
    
    // Get selected elements
    pub fn selected_elements(&self) -> &[ElementType] {
        &self.state.selected_elements
    }
}

impl SelectionTool<ScalingEnabled> {
    pub fn new() -> Self {
        Self { state: ScalingEnabled { selected_elements: Vec::new() } }
    }
    
    // Transition to TextureSelected state
    pub fn cancel_scaling(self) -> SelectionTool<TextureSelected> {
        SelectionTool { state: TextureSelected { selected_elements: self.state.selected_elements } }
    }
    
    // Transition to Scaling state
    pub fn start_scaling(self) -> SelectionTool<Scaling> {
        SelectionTool { state: Scaling { selected_elements: self.state.selected_elements } }
    }
    
    // Update selected elements
    pub fn update_selected_elements(&mut self, elements: Vec<ElementType>) {
        self.state.selected_elements = elements;
    }
    
    // Get selected elements
    pub fn selected_elements(&self) -> &[ElementType] {
        &self.state.selected_elements
    }
}

impl SelectionTool<Scaling> {
    pub fn new() -> Self {
        Self { state: Scaling { selected_elements: Vec::new() } }
    }
    
    // Transition to TextureSelected state
    pub fn finish_scaling(self) -> SelectionTool<TextureSelected> {
        SelectionTool { state: TextureSelected { selected_elements: self.state.selected_elements } }
    }
    
    // Update selected elements
    pub fn update_selected_elements(&mut self, elements: Vec<ElementType>) {
        self.state.selected_elements = elements;
    }
    
    // Get selected elements
    pub fn selected_elements(&self) -> &[ElementType] {
        &self.state.selected_elements
    }
}

impl Tool for SelectionTool<Active> {
    fn name(&self) -> &'static str {
        "Selection"
    }

    fn deactivate(&mut self, _doc: &Document) {
        // When the selection tool is deactivated, we want to clear any selected elements
        // This will be handled in the app.rs file by modifying the state
    }

    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        // Check if we're selecting an element
        if let Some(_element) = doc.element_at_position(pos) {
            // We'll transition to TextureSelected in the wrapper enum
            // The actual selection is handled in the central panel
        }
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // No state transition in Active state on pointer move
        None
    }

    fn update_preview(&mut self, _renderer: &mut Renderer) {
        // No preview needed for selection tool
    }

    fn clear_preview(&mut self, _renderer: &mut Renderer) {
        // No preview to clear
    }

    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool (Active)");
        ui.separator();
        ui.label("Click on elements to select them.");
        ui.label("Selected elements will be highlighted with a red box.");
        
        None  // No immediate command from UI
    }
}

// Implement Tool for TextureSelected state
impl Tool for SelectionTool<TextureSelected> {
    fn name(&self) -> &'static str {
        "Selection"
    }

    fn deactivate(&mut self, _doc: &Document) {
        // When the selection tool is deactivated, we want to clear any selected elements
        // This will be handled in the app.rs file by modifying the state
    }

    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // We don't return a command, but the selection will be handled in the central panel
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // We can't directly access the state from the document
        // The state will be passed to is_over_resize_handle by the wrapper
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // No state transition in TextureSelected state on pointer up
        None
    }

    fn update_preview(&mut self, _renderer: &mut Renderer) {
        // No preview needed for selection tool
    }

    fn clear_preview(&mut self, _renderer: &mut Renderer) {
        // No preview to clear
    }

    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool (Texture Selected)");
        ui.separator();
        ui.label("Element selected.");
        ui.label("Hover over resize handles to enable scaling.");
        
        None  // No immediate command from UI
    }
}

// Implement Tool for ScalingEnabled state
impl Tool for SelectionTool<ScalingEnabled> {
    fn name(&self) -> &'static str {
        "Selection"
    }

    fn deactivate(&mut self, _doc: &Document) {
        // When the selection tool is deactivated, we want to clear any selected elements
        // This will be handled in the app.rs file by modifying the state
    }

    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // State transitions are handled by the wrapper enum
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // Stay in ScalingEnabled state
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // State transitions are handled by the wrapper enum
        None
    }

    fn update_preview(&mut self, _renderer: &mut Renderer) {
        // No preview needed for selection tool
    }

    fn clear_preview(&mut self, _renderer: &mut Renderer) {
        // No preview to clear
    }

    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool (Scaling Enabled)");
        ui.separator();
        ui.label("Click and drag to resize the selected element.");
        ui.label("Release to cancel scaling.");
        
        None  // No immediate command from UI
    }
}

// Implement Tool for Scaling state
impl Tool for SelectionTool<Scaling> {
    fn name(&self) -> &'static str {
        "Selection"
    }

    fn deactivate(&mut self, _doc: &Document) {
        // When the selection tool is deactivated, we want to clear any selected elements
        // This will be handled in the app.rs file by modifying the state
    }

    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // Already scaling, ignore additional pointer down events
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // In a real implementation, this would update the scaling preview
        // For now, we'll just stay in the Scaling state
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document) -> Option<Command> {
        // State transitions are handled by the wrapper enum
        None
    }

    fn update_preview(&mut self, _renderer: &mut Renderer) {
        // In a real implementation, this would update the scaling preview
    }

    fn clear_preview(&mut self, _renderer: &mut Renderer) {
        // In a real implementation, this would clear the scaling preview
    }

    fn ui(&mut self, ui: &mut Ui, _doc: &Document) -> Option<Command> {
        ui.label("Selection Tool (Scaling Active)");
        ui.separator();
        ui.label("Dragging to resize...");
        ui.label("Release to apply scaling.");
        
        None  // No immediate command from UI
    }
}

// Wrapper enum to handle state transitions
#[derive(Clone)]
pub enum SelectionToolType {
    Active(SelectionTool<Active>),
    TextureSelected(SelectionTool<TextureSelected>),
    ScalingEnabled(SelectionTool<ScalingEnabled>),
    Scaling(SelectionTool<Scaling>),
}

impl Tool for SelectionToolType {
    fn name(&self) -> &'static str {
        match self {
            Self::Active(tool) => tool.name(),
            Self::TextureSelected(tool) => tool.name(),
            Self::ScalingEnabled(tool) => tool.name(),
            Self::Scaling(tool) => tool.name(),
        }
    }

    fn activate(&mut self, doc: &Document) {
        match self {
            Self::Active(tool) => tool.activate(doc),
            Self::TextureSelected(tool) => tool.activate(doc),
            Self::ScalingEnabled(tool) => tool.activate(doc),
            Self::Scaling(tool) => tool.activate(doc),
        }
    }
    
    fn deactivate(&mut self, doc: &Document) {
        match self {
            Self::Active(tool) => tool.deactivate(doc),
            Self::TextureSelected(tool) => tool.deactivate(doc),
            Self::ScalingEnabled(tool) => tool.deactivate(doc),
            Self::Scaling(tool) => tool.deactivate(doc),
        }
    }
    
    fn requires_selection(&self) -> bool {
        match self {
            Self::Active(tool) => tool.requires_selection(),
            Self::TextureSelected(tool) => tool.requires_selection(),
            Self::ScalingEnabled(tool) => tool.requires_selection(),
            Self::Scaling(tool) => tool.requires_selection(),
        }
    }

    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        match self {
            Self::Active(tool) => {
                let result = tool.on_pointer_down(pos, doc);
                
                // Check if we're selecting an element
                if let Some(element) = doc.element_at_position(pos) {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let active_tool = std::mem::take(tool);
                    
                    // Transition to TextureSelected state with the selected element
                    let texture_selected_tool = active_tool.select_texture(vec![element]);
                    
                    // Replace self with the TextureSelected variant
                    *self = SelectionToolType::TextureSelected(texture_selected_tool);
                }
                
                result
            },
            Self::TextureSelected(tool) => tool.on_pointer_down(pos, doc),
            Self::ScalingEnabled(tool) => {
                // Check if we're clicking on a resize handle
                let selected_elements = tool.selected_elements();
                if is_over_resize_handle(pos, doc, Some(selected_elements)) {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let scaling_enabled_tool = std::mem::take(tool);
                    
                    // Start scaling
                    let scaling_tool = scaling_enabled_tool.start_scaling();
                    
                    // Replace self with the Scaling variant
                    *self = SelectionToolType::Scaling(scaling_tool);
                    
                    // Forward the call to the new state
                    if let Self::Scaling(tool) = self {
                        tool.on_pointer_down(pos, doc)
                    } else {
                        None
                    }
                } else {
                    // Just forward the call
                    tool.on_pointer_down(pos, doc)
                }
            },
            Self::Scaling(tool) => tool.on_pointer_down(pos, doc),
        }
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        // We don't have direct access to the state here
        // We'll check if we're over a resize handle using the document's element_at_position
        
        match self {
            Self::Active(tool) => tool.on_pointer_move(pos, doc),
            Self::TextureSelected(tool) => {
                println!("TextureSelected: Checking if position {:?} is over resize handle", pos);
                
                // Get the selected elements from the tool state
                let selected_elements = tool.selected_elements();
                
                // Check if we're over a resize handle
                if is_over_resize_handle(pos, doc, Some(selected_elements)) {
                    println!("TextureSelected: Position is over resize handle, transitioning to ScalingEnabled");
                    
                    // Use std::mem::take to get ownership while leaving a default in place
                    let texture_selected_tool = std::mem::take(tool);
                    
                    // Enable scaling
                    let scaling_enabled_tool = texture_selected_tool.enable_scaling();
                    
                    // Replace self with the ScalingEnabled variant
                    *self = SelectionToolType::ScalingEnabled(scaling_enabled_tool);
                    
                    // Forward the call to the new state
                    if let Self::ScalingEnabled(tool) = self {
                        tool.on_pointer_move(pos, doc)
                    } else {
                        None
                    }
                } else {
                    // Just forward the call
                    tool.on_pointer_move(pos, doc)
                }
            },
            Self::ScalingEnabled(tool) => {
                // Check if we're still over a resize handle
                println!("ScalingEnabled: Checking if position {:?} is over resize handle", pos);
                
                // Get the selected elements from the tool state
                let selected_elements = tool.selected_elements();
                
                if !is_over_resize_handle(pos, doc, Some(selected_elements)) {
                    println!("ScalingEnabled: Position is not over resize handle, transitioning to TextureSelected");
                    
                    // Use std::mem::take to get ownership while leaving a default in place
                    let scaling_enabled_tool = std::mem::take(tool);
                    
                    // Cancel scaling
                    let texture_selected_tool = scaling_enabled_tool.cancel_scaling();
                    
                    // Replace self with the TextureSelected variant
                    *self = SelectionToolType::TextureSelected(texture_selected_tool);
                    
                    // Forward the call to the new state
                    if let Self::TextureSelected(tool) = self {
                        tool.on_pointer_move(pos, doc)
                    } else {
                        None
                    }
                } else {
                    // Just forward the call
                    tool.on_pointer_move(pos, doc)
                }
            },
            Self::Scaling(tool) => tool.on_pointer_move(pos, doc),
        }
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.on_pointer_up(pos, doc),
            Self::TextureSelected(tool) => tool.on_pointer_up(pos, doc),
            Self::ScalingEnabled(tool) => {
                // Use std::mem::take to get ownership while leaving a default in place
                let scaling_enabled_tool = std::mem::take(tool);
                
                // Cancel scaling
                let texture_selected_tool = scaling_enabled_tool.cancel_scaling();
                
                // Replace self with the TextureSelected variant
                *self = SelectionToolType::TextureSelected(texture_selected_tool);
                
                // Forward the call to the new state
                if let Self::TextureSelected(tool) = self {
                    tool.on_pointer_up(pos, doc)
                } else {
                    None
                }
            },
            Self::Scaling(tool) => {
                // Use std::mem::take to get ownership while leaving a default in place
                let scaling_tool = std::mem::take(tool);
                
                // Finish scaling
                let texture_selected_tool = scaling_tool.finish_scaling();
                
                // Replace self with the TextureSelected variant
                *self = SelectionToolType::TextureSelected(texture_selected_tool);
                
                // Forward the call to the new state
                if let Self::TextureSelected(tool) = self {
                    tool.on_pointer_up(pos, doc)
                } else {
                    None
                }
            },
        }
    }
    
    fn update_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::Active(tool) => tool.update_preview(renderer),
            Self::TextureSelected(tool) => tool.update_preview(renderer),
            Self::ScalingEnabled(tool) => tool.update_preview(renderer),
            Self::Scaling(tool) => tool.update_preview(renderer),
        }
    }
    
    fn clear_preview(&mut self, renderer: &mut Renderer) {
        match self {
            Self::Active(tool) => tool.clear_preview(renderer),
            Self::TextureSelected(tool) => tool.clear_preview(renderer),
            Self::ScalingEnabled(tool) => tool.clear_preview(renderer),
            Self::Scaling(tool) => tool.clear_preview(renderer),
        }
    }
    
    fn ui(&mut self, ui: &mut Ui, doc: &Document) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.ui(ui, doc),
            Self::TextureSelected(tool) => tool.ui(ui, doc),
            Self::ScalingEnabled(tool) => tool.ui(ui, doc),
            Self::Scaling(tool) => tool.ui(ui, doc),
        }
    }
}

impl SelectionToolType {
    pub fn new() -> Self {
        SelectionToolType::Active(SelectionTool::<Active>::new())
    }
    
    pub fn current_state_name(&self) -> &'static str {
        match self {
            Self::Active(_) => "Active",
            Self::TextureSelected(_) => "TextureSelected",
            Self::ScalingEnabled(_) => "ScalingEnabled",
            Self::Scaling(_) => "Scaling",
        }
    }

    // Update state based on selected elements
    pub fn update_for_selected_elements(&mut self, selected_elements: &[ElementType]) {
        let elements = selected_elements.to_vec();
        
        match self {
            Self::Active(tool) => {
                if !elements.is_empty() {
                    // Transition to TextureSelected if we have selected elements
                    let active_tool = std::mem::take(tool);
                    let texture_selected_tool = active_tool.select_texture(elements);
                    *self = SelectionToolType::TextureSelected(texture_selected_tool);
                }
            },
            Self::TextureSelected(tool) => {
                if elements.is_empty() {
                    // Transition to Active if we have no selected elements
                    let texture_selected_tool = std::mem::take(tool);
                    let active_tool = texture_selected_tool.deselect_texture();
                    *self = SelectionToolType::Active(active_tool);
                } else {
                    // Update the selected elements
                    tool.update_selected_elements(elements);
                }
            },
            Self::ScalingEnabled(tool) => {
                if elements.is_empty() {
                    // Transition to Active if we have no selected elements
                    let scaling_enabled_tool = std::mem::take(tool);
                    let texture_selected_tool = scaling_enabled_tool.cancel_scaling();
                    let active_tool = texture_selected_tool.deselect_texture();
                    *self = SelectionToolType::Active(active_tool);
                } else {
                    // Update the selected elements
                    tool.update_selected_elements(elements);
                }
            },
            Self::Scaling(tool) => {
                if elements.is_empty() {
                    // Transition to Active if we have no selected elements
                    let scaling_tool = std::mem::take(tool);
                    let texture_selected_tool = scaling_tool.finish_scaling();
                    let active_tool = texture_selected_tool.deselect_texture();
                    *self = SelectionToolType::Active(active_tool);
                } else {
                    // Update the selected elements
                    tool.update_selected_elements(elements);
                }
            },
        }
    }
}

pub fn new_selection_tool() -> SelectionToolType {
    SelectionToolType::new()
}

// Helper function to check if a position is over a resize handle
fn is_over_resize_handle(pos: Pos2, doc: &Document, selected_elements: Option<&[ElementType]>) -> bool {
    // First, check if we're over a resize handle of any selected elements
    if let Some(elements) = selected_elements {
        if !elements.is_empty() {
            println!("Checking {} selected elements for resize handles", elements.len());
            
            for element in elements {
                let rect = get_element_rect(element);
                
                // Check all four corners with a generous radius for easier detection
                let handle_radius = 15.0;
                
                let corners = [
                    (rect.left_top(), "left_top"),
                    (rect.right_top(), "right_top"),
                    (rect.left_bottom(), "left_bottom"),
                    (rect.right_bottom(), "right_bottom"),
                ];
                
                for (corner, name) in corners.iter() {
                    let distance = pos.distance(*corner);
                    
                    if distance <= handle_radius {
                        println!("Found resize handle at corner: {}, distance: {}", name, distance);
                        return true;
                    }
                }
            }
            
            // We have selected elements but didn't find a resize handle
            // Continue checking other elements instead of returning early
            println!("No resize handles found in selected elements, checking other elements");
        }
    }
    
    // If we don't have selected elements or they're empty, check the element at the position
    if let Some(element) = doc.element_at_position(pos) {
        let rect = get_element_rect(&element);
        
        // Check if the position is near any of the corner handles
        let handle_radius = 15.0;
        
        let corners = [
            (rect.left_top(), "left_top"),
            (rect.right_top(), "right_top"),
            (rect.left_bottom(), "left_bottom"),
            (rect.right_bottom(), "right_bottom"),
        ];
        
        for (corner, name) in corners.iter() {
            let distance = pos.distance(*corner);
            
            if distance <= handle_radius {
                println!("Found resize handle at corner: {}, distance: {}", name, distance);
                return true;
            }
        }
    }
    
    // If we didn't find a handle at the position, check all nearby positions
    let nearby_offsets = [
        (-5.0, -5.0), (0.0, -5.0), (5.0, -5.0),
        (-5.0, 0.0),               (5.0, 0.0),
        (-5.0, 5.0),  (0.0, 5.0),  (5.0, 5.0),
    ];
    
    for (dx, dy) in nearby_offsets.iter() {
        let nearby_pos = egui::pos2(pos.x + dx, pos.y + dy);
        if let Some(element) = doc.element_at_position(nearby_pos) {
            let rect = get_element_rect(&element);
            
            // Check if the original position is near any of the corner handles
            let handle_radius = 15.0;
            
            let corners = [
                (rect.left_top(), "left_top"),
                (rect.right_top(), "right_top"),
                (rect.left_bottom(), "left_bottom"),
                (rect.right_bottom(), "right_bottom"),
            ];
            
            for (corner, name) in corners.iter() {
                let distance = pos.distance(*corner);
                
                if distance <= handle_radius {
                    println!("Found resize handle at corner: {} via nearby position, distance: {}", name, distance);
                    return true;
                }
            }
        }
    }
    
    false
}

// Helper function to get the bounding rectangle of an element
fn get_element_rect(element: &ElementType) -> egui::Rect {
    match element {
        ElementType::Stroke(stroke_ref) => {
            // For strokes, calculate bounding box from points
            let points = stroke_ref.points();
            if points.is_empty() {
                return egui::Rect::NOTHING;
            }
            
            // Find min/max coordinates
            let mut min_x = points[0].x;
            let mut min_y = points[0].y;
            let mut max_x = points[0].x;
            let mut max_y = points[0].y;
            
            for point in points {
                min_x = min_x.min(point.x);
                min_y = min_y.min(point.y);
                max_x = max_x.max(point.x);
                max_y = max_y.max(point.y);
            }
            
            // Add padding - use a larger padding for strokes to make resize handles easier to grab
            // Also consider the stroke thickness
            let base_padding = 10.0;
            let thickness_padding = stroke_ref.thickness();
            let padding = base_padding + thickness_padding;
            
            min_x -= padding;
            min_y -= padding;
            max_x += padding;
            max_y += padding;
            
            let rect = egui::Rect::from_min_max(
                egui::pos2(min_x, min_y),
                egui::pos2(max_x, max_y),
            );
            
            println!("Stroke bounding box: {:?}", rect);
            rect
        },
        ElementType::Image(image_ref) => {
            // For images, use the image's rect with some padding
            let rect = image_ref.rect();
            let padding = 5.0;
            let padded_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x - padding, rect.min.y - padding),
                egui::pos2(rect.max.x + padding, rect.max.y + padding),
            );
            
            println!("Image bounding box: {:?}", padded_rect);
            padded_rect
        }
    }
}

impl Default for SelectionTool<Active> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SelectionTool<TextureSelected> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SelectionTool<ScalingEnabled> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SelectionTool<Scaling> {
    fn default() -> Self {
        Self::new()
    }
}