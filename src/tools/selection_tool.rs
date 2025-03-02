use egui::{Pos2, Ui};
use crate::command::Command;
use crate::document::Document;
use crate::tools::Tool;
use crate::renderer::Renderer;
use crate::state::ElementType;
use crate::geometry::hit_testing::{compute_element_rect, is_point_near_handle, RESIZE_HANDLE_RADIUS};
use crate::state::EditorState;

// State type definitions
#[derive(Clone)]
pub struct Active;

#[derive(Clone)]
pub struct TextureSelected;

#[derive(Clone)]
pub struct ScalingEnabled {
    initial_bounds: egui::Rect,
}

#[derive(Clone)]
pub struct Scaling {
    scale_factor: egui::Vec2,
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
    pub fn select_texture(self) -> SelectionTool<TextureSelected> {
        SelectionTool { state: TextureSelected }
    }
}

impl SelectionTool<TextureSelected> {
    pub fn new() -> Self {
        Self { state: TextureSelected }
    }
    
    // Transition to Active state
    pub fn deselect_texture(self) -> SelectionTool<Active> {
        SelectionTool { state: Active }
    }
    
    // Transition to ScalingEnabled state
    pub fn enable_scaling(self) -> SelectionTool<ScalingEnabled> {
        SelectionTool { state: ScalingEnabled { initial_bounds: egui::Rect::from_min_max(Pos2::ZERO, Pos2::ZERO) } }
    }
}

impl SelectionTool<ScalingEnabled> {
    pub fn new() -> Self {
        Self { state: ScalingEnabled { initial_bounds: egui::Rect::from_min_max(Pos2::ZERO, Pos2::ZERO) } }
    }
    
    // Transition to TextureSelected state
    pub fn cancel_scaling(self) -> SelectionTool<TextureSelected> {
        SelectionTool { state: TextureSelected }
    }
    
    // Transition to Scaling state
    pub fn start_scaling(self) -> SelectionTool<Scaling> {
        SelectionTool { state: Scaling { scale_factor: egui::Vec2::ZERO } }
    }
    
    // Update selected elements
    pub fn update_selected_elements(&mut self, _elements: Vec<ElementType>) {
        // This method is no longer used in the new implementation
    }
    
    // Get selected elements
    pub fn selected_elements(&self) -> &[ElementType] {
        &[]
    }
}

impl SelectionTool<Scaling> {
    pub fn new() -> Self {
        Self { state: Scaling { scale_factor: egui::Vec2::ZERO } }
    }
    
    // Transition to TextureSelected state
    pub fn finish_scaling(self) -> SelectionTool<TextureSelected> {
        SelectionTool { state: TextureSelected }
    }
    
    // Update selected elements
    pub fn update_selected_elements(&mut self, _elements: Vec<ElementType>) {
        // This method is no longer used in the new implementation
    }
    
    // Get selected elements
    pub fn selected_elements(&self) -> &[ElementType] {
        &[]
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

    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document, _state: &EditorState) -> Option<Command> {
        // Check if we're selecting an element
        if let Some(_element) = doc.element_at_position(pos) {
            // We'll transition to TextureSelected in the wrapper enum
            // The actual selection is handled in the central panel
        }
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // No state transition in Active state on pointer move
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // No state transition in Active state on pointer up
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

    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // We don't return a command, but the selection will be handled in the central panel
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // We can't directly access the state from the document
        // The state will be passed to is_over_resize_handle by the wrapper
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
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

    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // State transitions are handled by the wrapper enum
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // Stay in ScalingEnabled state
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
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

    fn on_pointer_down(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // State transitions are handled by the wrapper enum
        None
    }
    
    fn on_pointer_move(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // Handle scaling in the wrapper enum
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
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
        ui.label("Selection Tool (Scaling)");
        ui.separator();
        ui.label("Drag to resize the selected element.");
        ui.label("Release to apply the scaling.");
        
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

    fn on_pointer_down(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::Active(tool) => {
                let result = tool.on_pointer_down(pos, doc, state);
                
                // Check if we're selecting an element
                if !state.selected_elements().is_empty() {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let active_tool = std::mem::take(tool);
                    
                    // Transition to TextureSelected state with the selected element
                    // Note: We no longer store elements in the state
                    let texture_selected_tool = active_tool.select_texture();
                    
                    // Replace self with the TextureSelected variant
                    *self = SelectionToolType::TextureSelected(texture_selected_tool);
                }
                
                result
            },
            Self::TextureSelected(tool) => tool.on_pointer_down(pos, doc, state),
            Self::ScalingEnabled(tool) => {
                // Check if we're clicking on a resize handle
                if is_over_resize_handle(pos, doc, state) {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let scaling_enabled_tool = std::mem::take(tool);
                    
                    // Start scaling
                    let scaling_tool = scaling_enabled_tool.start_scaling();
                    
                    // Replace self with the Scaling variant
                    *self = SelectionToolType::Scaling(scaling_tool);
                    
                    None
                } else {
                    tool.on_pointer_down(pos, doc, state)
                }
            },
            Self::Scaling(tool) => tool.on_pointer_down(pos, doc, state),
        }
    }
    
    fn on_pointer_move(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.on_pointer_move(pos, doc, state),
            Self::TextureSelected(tool) => {
                println!("TextureSelected: Checking if position {:?} is over resize handle", pos);
                
                // Check if we're over a resize handle
                if is_over_resize_handle(pos, doc, state) {
                    println!("TextureSelected: Position is over resize handle, transitioning to ScalingEnabled");
                    
                    // Use std::mem::take to get ownership while leaving a default in place
                    let texture_selected_tool = std::mem::take(tool);
                    
                    // Enable scaling
                    let scaling_enabled_tool = texture_selected_tool.enable_scaling();
                    
                    // Replace self with the ScalingEnabled variant
                    *self = SelectionToolType::ScalingEnabled(scaling_enabled_tool);
                    
                    None
                } else {
                    tool.on_pointer_move(pos, doc, state)
                }
            },
            Self::ScalingEnabled(tool) => {
                // Check if we're still over a resize handle
                println!("ScalingEnabled: Checking if position {:?} is over resize handle", pos);
                
                if !is_over_resize_handle(pos, doc, state) {
                    println!("ScalingEnabled: Position is not over resize handle, transitioning to TextureSelected");
                    
                    // Use std::mem::take to get ownership while leaving a default in place
                    let scaling_enabled_tool = std::mem::take(tool);
                    
                    // Cancel scaling
                    let texture_selected_tool = scaling_enabled_tool.cancel_scaling();
                    
                    // Replace self with the TextureSelected variant
                    *self = SelectionToolType::TextureSelected(texture_selected_tool);
                    
                    None
                } else {
                    tool.on_pointer_move(pos, doc, state)
                }
            },
            Self::Scaling(tool) => tool.on_pointer_move(pos, doc, state),
        }
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.on_pointer_up(pos, doc, state),
            Self::TextureSelected(tool) => tool.on_pointer_up(pos, doc, state),
            Self::ScalingEnabled(tool) => {
                // Use std::mem::take to get ownership while leaving a default in place
                let scaling_enabled_tool = std::mem::take(tool);
                
                // Cancel scaling
                let texture_selected_tool = scaling_enabled_tool.cancel_scaling();
                
                // Replace self with the TextureSelected variant
                *self = SelectionToolType::TextureSelected(texture_selected_tool);
                
                None
            },
            Self::Scaling(tool) => {
                // Use std::mem::take to get ownership while leaving a default in place
                let scaling_tool = std::mem::take(tool);
                
                // Finish scaling
                let texture_selected_tool = scaling_tool.finish_scaling();
                
                // Replace self with the TextureSelected variant
                *self = SelectionToolType::TextureSelected(texture_selected_tool);
                
                None
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

    // Update state based on editor state
    pub fn update_from_editor_state(&mut self, state: &crate::state::EditorState) {
        let has_elements = !state.selected_elements().is_empty();
        
        match self {
            Self::Active(tool) => {
                if has_elements {
                    // Transition to TextureSelected if we have selected elements
                    let active_tool = std::mem::take(tool);
                    let texture_selected_tool = active_tool.select_texture();
                    *self = SelectionToolType::TextureSelected(texture_selected_tool);
                }
            },
            Self::TextureSelected(tool) => {
                if !has_elements {
                    // Transition to Active if we have no selected elements
                    let texture_selected_tool = std::mem::take(tool);
                    let active_tool = texture_selected_tool.deselect_texture();
                    *self = SelectionToolType::Active(active_tool);
                }
            },
            Self::ScalingEnabled(tool) => {
                if !has_elements {
                    // Transition to Active if we have no selected elements
                    let scaling_enabled_tool = std::mem::take(tool);
                    let texture_selected_tool = scaling_enabled_tool.cancel_scaling();
                    let active_tool = texture_selected_tool.deselect_texture();
                    *self = SelectionToolType::Active(active_tool);
                }
            },
            Self::Scaling(tool) => {
                if !has_elements {
                    // Transition to Active if we have no selected elements
                    let scaling_tool = std::mem::take(tool);
                    let texture_selected_tool = scaling_tool.finish_scaling();
                    let active_tool = texture_selected_tool.deselect_texture();
                    *self = SelectionToolType::Active(active_tool);
                }
            },
        }
    }
}

pub fn new_selection_tool() -> SelectionToolType {
    SelectionToolType::new()
}

// Helper function to check if a position is over a resize handle
fn is_over_resize_handle(pos: Pos2, doc: &Document, state: &crate::state::EditorState) -> bool {
    // First check selected elements from EditorState
    for element in state.selected_elements() {
        if is_point_near_handle(pos, element) {
            return true;
        }
    }
    
    // If we don't have selected elements or they're empty, check the element at the position
    if let Some(element) = doc.element_at_position(pos) {
        if is_point_near_handle(pos, &element) {
            return true;
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
            let rect = compute_element_rect(&element);
            
            // Check if the original position is near any of the corner handles
            let handle_radius = RESIZE_HANDLE_RADIUS;
            
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