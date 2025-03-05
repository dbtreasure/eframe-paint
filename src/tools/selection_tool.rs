use egui::{Pos2, Ui, Rect};
use crate::command::Command;
use crate::document::Document;
use crate::tools::{Tool, ToolConfig};
use crate::renderer::Renderer;
use crate::state::ElementType;
use crate::geometry::hit_testing::{compute_element_rect, is_point_near_handle, RESIZE_HANDLE_RADIUS};
use crate::state::EditorState;
use crate::widgets::Corner;
use std::any::Any;
use std::marker::PhantomData;
use log::{debug, info};
use std::sync::Arc;

// Config for SelectionTool
#[derive(Clone, Debug)]
pub struct SelectionToolConfig {
    // Add any configurable properties here
    // For now, it's just a placeholder
}

impl ToolConfig for SelectionToolConfig {
    fn tool_name(&self) -> &'static str {
        "Selection"
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
pub struct Active;

#[derive(Clone, Debug)]
pub struct TextureSelected;

#[derive(Clone, Debug)]
pub struct ScalingEnabled;

#[derive(Clone)]
pub struct Scaling {
    element: ElementType,
    corner: Corner,
    original_rect: egui::Rect,
    start_pos: egui::Pos2,
    handle_size: f32,
    _state: std::marker::PhantomData<Scaling>,
    current_preview: Option<egui::Rect>,
}

// New consolidated state enum for the refactored SelectionTool
#[derive(Clone)]
pub enum SelectionState {
    Idle,
    Selecting {
        element: ElementType,
        start_pos: Pos2
    },
    Resizing {
        element: ElementType,
        corner: Corner,
        original_rect: Rect,
        start_pos: Pos2,
        handle_size: f32
    },
    Dragging {
        element: ElementType,
        offset: egui::Vec2
    }
}

// Manual Debug implementation for SelectionState
impl std::fmt::Debug for SelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Selecting { start_pos, .. } => f.debug_struct("Selecting")
                .field("start_pos", start_pos)
                .finish_non_exhaustive(),
            Self::Resizing { corner, original_rect, start_pos, handle_size, .. } => f.debug_struct("Resizing")
                .field("corner", corner)
                .field("original_rect", original_rect)
                .field("start_pos", start_pos)
                .field("handle_size", handle_size)
                .finish_non_exhaustive(),
            Self::Dragging { offset, .. } => f.debug_struct("Dragging")
                .field("offset", offset)
                .finish_non_exhaustive(),
        }
    }
}

// New consolidated SelectionTool struct
#[derive(Debug, Clone)]
pub struct UnifiedSelectionTool {
    pub state: SelectionState,
    pub handle_size: f32,
    pub current_preview: Option<Rect>
}

impl UnifiedSelectionTool {
    pub fn new() -> Self {
        Self {
            state: SelectionState::Idle,
            handle_size: RESIZE_HANDLE_RADIUS,
            current_preview: None
        }
    }
}

// Manual Debug implementation for Scaling
impl std::fmt::Debug for Scaling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scaling")
            .field("corner", &self.corner)
            .field("original_rect", &self.original_rect)
            .field("start_pos", &self.start_pos)
            .field("handle_size", &self.handle_size)
            .finish_non_exhaustive() // Skip the element field which doesn't implement Debug
    }
}

#[derive(Clone)]
pub struct SelectionTool<State = Active> {
    handle_size: f32,
    _state: std::marker::PhantomData<State>,
    // Fields needed for scaling
    element: Option<ElementType>,
    corner: Option<Corner>,
    original_rect: Option<egui::Rect>,
    start_pos: Option<egui::Pos2>,
    // Current preview rect during scaling
    current_preview: Option<egui::Rect>,
}

impl<State> std::fmt::Debug for SelectionTool<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionTool")
            .field("handle_size", &self.handle_size)
            .field("corner", &self.corner)
            .field("original_rect", &self.original_rect)
            .field("start_pos", &self.start_pos)
            .field("current_preview", &self.current_preview)
            .finish_non_exhaustive() // Skip element field which doesn't implement Debug
    }
}

impl SelectionTool<Active> {
    pub fn new() -> Self {
        Self {
            handle_size: RESIZE_HANDLE_RADIUS,
            _state: std::marker::PhantomData,
            element: None,
            corner: None,
            original_rect: None,
            start_pos: None,
            current_preview: None,
        }
    }

    pub fn handle_size(&self) -> f32 {
        self.handle_size
    }

    pub fn set_handle_size(&mut self, size: f32) {
        self.handle_size = size;
    }

    pub fn restore_state(&mut self, other: &SelectionToolType) {
        if let SelectionToolType::Active(other_tool) = other {
            self.handle_size = other_tool.handle_size();
        }
    }
    
    pub fn select_texture(&self, element: ElementType) -> Result<SelectionTool<TextureSelected>, Self> {
        let mut texture_selected = SelectionTool::<TextureSelected>::new();
        texture_selected.element = Some(element);
        Ok(texture_selected)
    }
}

impl SelectionTool<TextureSelected> {
    pub fn new() -> Self {
        Self {
            handle_size: RESIZE_HANDLE_RADIUS,
            _state: std::marker::PhantomData,
            element: None,
            corner: None,
            original_rect: None,
            start_pos: None,
            current_preview: None,
        }
    }
    
    pub fn enable_scaling(self) -> Result<SelectionTool<ScalingEnabled>, Self> {
        if let Some(element) = &self.element {
            let mut scaling_enabled = SelectionTool::<ScalingEnabled>::new();
            scaling_enabled.element = Some(element.clone());
            Ok(scaling_enabled)
        } else {
            Err(self)
        }
    }
}

impl SelectionTool<ScalingEnabled> {
    pub fn new() -> Self {
        Self {
            handle_size: RESIZE_HANDLE_RADIUS,
            _state: std::marker::PhantomData,
            element: None,
            corner: None,
            original_rect: None,
            start_pos: None,
            current_preview: None,
        }
    }
    
    pub fn start_scaling(self, corner: Corner, pos: Pos2) -> Result<SelectionTool<Scaling>, Self> {
        if let Some(element) = &self.element {
            let original_rect = element.rect();
            let scaling = SelectionTool::<Scaling>::new(
                element.clone(),
                corner,
                pos
            );
            Ok(scaling)
        } else {
            Err(self)
        }
    }
    
    pub fn cancel_scaling(self) -> Result<SelectionTool<TextureSelected>, Self> {
        if let Some(element) = &self.element {
            let mut texture_selected = SelectionTool::<TextureSelected>::new();
            texture_selected.element = Some(element.clone());
            Ok(texture_selected)
        } else {
            Err(self)
        }
    }
}

impl SelectionTool<Scaling> {
    pub fn new(element: ElementType, corner: Corner, start_pos: egui::Pos2) -> Self {
        let original_rect = element.rect();
        Self {
            handle_size: RESIZE_HANDLE_RADIUS,
            _state: std::marker::PhantomData,
            element: Some(element),
            corner: Some(corner),
            original_rect: Some(original_rect),
            start_pos: Some(start_pos),
            current_preview: None,
        }
    }
    
    pub fn calculate_preview_rect(&self, current_pos: egui::Pos2) -> egui::Rect {
        let start_pos = self.start_pos.expect("start_pos should be set during scaling");
        let original_rect = self.original_rect.expect("original_rect should be set during scaling");
        let corner = self.corner.expect("corner should be set during scaling");
        
        let delta = current_pos - start_pos;
        let mut rect = original_rect;
        
        // Calculate new rect based on which corner is being dragged
        match corner {
            Corner::TopLeft => {
                // When dragging top-left, min.x and min.y change while max remains fixed
                rect.min.x = (original_rect.min.x + delta.x).min(rect.max.x - 10.0);
                rect.min.y = (original_rect.min.y + delta.y).min(rect.max.y - 10.0);
            }
            Corner::TopRight => {
                // When dragging top-right, max.x and min.y change while min.x and max.y remain fixed
                rect.max.x = (original_rect.max.x + delta.x).max(rect.min.x + 10.0);
                rect.min.y = (original_rect.min.y + delta.y).min(rect.max.y - 10.0);
            }
            Corner::BottomLeft => {
                // When dragging bottom-left, min.x and max.y change while max.x and min.y remain fixed
                rect.min.x = (original_rect.min.x + delta.x).min(rect.max.x - 10.0);
                rect.max.y = (original_rect.max.y + delta.y).max(rect.min.y + 10.0);
            }
            Corner::BottomRight => {
                // When dragging bottom-right, max.x and max.y change while min remains fixed
                rect.max.x = (original_rect.max.x + delta.x).max(rect.min.x + 10.0);
                rect.max.y = (original_rect.max.y + delta.y).max(rect.min.y + 10.0);
            }
        }
        
        // Ensure minimum size of 10x10 pixels
        if rect.width() < 10.0 {
            match corner {
                Corner::TopLeft | Corner::BottomLeft => rect.min.x = rect.max.x - 10.0,
                Corner::TopRight | Corner::BottomRight => rect.max.x = rect.min.x + 10.0,
            }
        }
        if rect.height() < 10.0 {
            match corner {
                Corner::TopLeft | Corner::TopRight => rect.min.y = rect.max.y - 10.0,
                Corner::BottomLeft | Corner::BottomRight => rect.max.y = rect.min.y + 10.0,
            }
        }
        
        rect
    }
    
    fn finish_scaling(self) -> Result<(Command, SelectionTool<TextureSelected>), Self> {
        if let (Some(element), Some(corner), Some(preview_rect)) = (self.element.as_ref(), self.corner, self.current_preview) {
            let element_id = element.get_id();
            let new_position = match corner {
                Corner::TopLeft => preview_rect.left_top(),
                Corner::TopRight => preview_rect.right_top(),
                Corner::BottomLeft => preview_rect.left_bottom(),
                Corner::BottomRight => preview_rect.right_bottom(),
            };
            
            let command = Command::ResizeElement {
                element_id,
                corner,
                new_position,
            };
            
            let mut texture_selected = SelectionTool::<TextureSelected>::new();
            texture_selected.element = self.element;
            Ok((command, texture_selected))
        } else {
            Err(self)
        }
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
    
    fn on_pointer_move(&mut self, pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // Calculate and update the preview rect based on the current mouse position
        let preview_rect = self.calculate_preview_rect(pos);
        self.current_preview = Some(preview_rect);
        None
    }
    
    fn on_pointer_up(&mut self, _pos: Pos2, _doc: &Document, _state: &EditorState) -> Option<Command> {
        // State transitions are handled by the wrapper enum
        None
    }

    fn update_preview(&mut self, renderer: &mut Renderer) {
        // Update the renderer with our current preview rect
        if let Some(preview_rect) = self.current_preview {
            renderer.set_resize_preview(Some(preview_rect));
        }
    }

    fn clear_preview(&mut self, renderer: &mut Renderer) {
        // Clear the resize preview
        self.current_preview = None;
        renderer.set_resize_preview(None);
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
                if let Some(element) = doc.element_at_position(pos) {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let active_tool = std::mem::take(tool);
                    
                    // Transition to TextureSelected state with the selected element
                    match active_tool.select_texture(element.clone()) {
                        Ok(texture_selected_tool) => {
                            // Replace self with the TextureSelected variant
                            *self = SelectionToolType::TextureSelected(texture_selected_tool);
                        }
                        Err(original_tool) => {
                            // If transition failed, restore the original tool
                            *tool = original_tool;
                        }
                    }
                }
                
                result
            },
            Self::TextureSelected(tool) => {
                // Check if we clicked outside the selected texture
                if !tool.contains_point(pos) {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let _texture_selected_tool = std::mem::take(tool);
                    
                    // Replace self with a new Active variant
                    *self = SelectionToolType::Active(SelectionTool::<Active>::new());
                }
                None
            },
            Self::ScalingEnabled(tool) => {
                // Check if we're clicking on a resize handle
                if is_over_resize_handle(pos, doc, state) {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let scaling_enabled_tool = std::mem::take(tool);
                    
                    // Start scaling with the current corner and position
                    match scaling_enabled_tool.start_scaling(Corner::TopLeft, pos) {
                        Ok(scaling_tool) => {
                            // Replace self with the Scaling variant
                            *self = SelectionToolType::Scaling(scaling_tool);
                        }
                        Err(original_tool) => {
                            // If transition failed, restore the original tool
                            *tool = original_tool;
                        }
                    }
                    
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
                // Check if we're hovering over a resize handle
                if is_over_resize_handle(pos, doc, state) {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let texture_selected_tool = std::mem::take(tool);
                    
                    // Enable scaling
                    match texture_selected_tool.enable_scaling() {
                        Ok(scaling_enabled_tool) => {
                            // Replace self with the ScalingEnabled variant
                            *self = SelectionToolType::ScalingEnabled(scaling_enabled_tool);
                        }
                        Err(original_tool) => {
                            // If transition failed, restore the original tool
                            *tool = original_tool;
                        }
                    }
                }
                
                None
            },
            Self::ScalingEnabled(tool) => tool.on_pointer_move(pos, doc, state),
            Self::Scaling(tool) => tool.on_pointer_move(pos, doc, state),
        }
    }
    
    fn on_pointer_up(&mut self, pos: Pos2, doc: &Document, state: &EditorState) -> Option<Command> {
        match self {
            Self::Active(tool) => tool.on_pointer_up(pos, doc, state),
            Self::TextureSelected(tool) => {
                // Check if we clicked outside the selected texture
                if !tool.contains_point(pos) {
                    // Use std::mem::take to get ownership while leaving a default in place
                    let _texture_selected_tool = std::mem::take(tool);
                    
                    // Replace self with a new Active variant
                    *self = SelectionToolType::Active(SelectionTool::<Active>::new());
                }
                None
            },
            Self::ScalingEnabled(tool) => {
                // Use std::mem::take to get ownership while leaving a default in place
                let scaling_enabled_tool = std::mem::take(tool);
                
                // Cancel scaling
                match scaling_enabled_tool.cancel_scaling() {
                    Ok(texture_selected_tool) => {
                        // Replace self with the TextureSelected variant
                        *self = SelectionToolType::TextureSelected(texture_selected_tool);
                        None
                    }
                    Err(original_tool) => {
                        // If transition failed, restore the original tool
                        *tool = original_tool;
                        None
                    }
                }
            },
            Self::Scaling(tool) => {
                // Use std::mem::take to get ownership while leaving a default in place
                let scaling_tool = std::mem::take(tool);
                
                // Finish scaling and get the command
                match scaling_tool.finish_scaling() {
                    Ok((command, texture_selected_tool)) => {
                        // Replace self with the TextureSelected variant
                        *self = SelectionToolType::TextureSelected(texture_selected_tool);
                        Some(command)
                    }
                    Err(original_tool) => {
                        // If transition failed, restore the original tool
                        *tool = original_tool;
                        None
                    }
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

    // Update state based on editor state
    pub fn update_from_editor_state(&mut self, state: &crate::state::EditorState) {
        let has_elements = !state.selected_elements().is_empty();
        
        match self {
            Self::Active(tool) => {
                if has_elements {
                    // Transition to TextureSelected if we have selected elements
                    let active_tool = std::mem::take(tool);
                    if let Some(first_element) = state.selected_elements().first() {
                        if let Ok(texture_selected_tool) = active_tool.select_texture(first_element.clone()) {
                            *self = SelectionToolType::TextureSelected(texture_selected_tool);
                        }
                    }
                }
            },
            Self::TextureSelected(tool) => {
                if !has_elements {
                    // Transition to Active if we have no selected elements
                    let _texture_selected_tool = std::mem::take(tool);
                    *self = SelectionToolType::Active(SelectionTool::<Active>::new());
                }
            },
            Self::ScalingEnabled(tool) => {
                if !has_elements {
                    // Transition to Active if we have no selected elements
                    let _scaling_enabled_tool = std::mem::take(tool);
                    *self = SelectionToolType::Active(SelectionTool::<Active>::new());
                }
            },
            Self::Scaling(tool) => {
                if !has_elements {
                    // Transition to Active if we have no selected elements
                    let scaling_tool = std::mem::take(tool);
                    if let Ok((_, texture_selected_tool)) = scaling_tool.finish_scaling() {
                        *self = SelectionToolType::Active(SelectionTool::<Active>::new());
                    }
                }
            },
        }
    }

    // Check if the tool has an active transform operation
    pub fn has_active_transform(&self) -> bool {
        match self {
            Self::Active(_) => false,
            Self::TextureSelected(_) => false,
            Self::ScalingEnabled(_) => true,
            Self::Scaling(_) => true,
        }
    }
    
    // Check if the tool has pending texture operations
    pub fn has_pending_texture_ops(&self) -> bool {
        match self {
            Self::TextureSelected(_) => false, // For now, always return false
            _ => false, // Only TextureSelected state can have pending texture ops
        }
    }
    
    // Check if this tool can transition to another state
    pub fn can_transition(&self) -> bool {
        match self {
            Self::Active(_) => true,
            Self::TextureSelected(_) => true,
            Self::ScalingEnabled(_) => true,
            Self::Scaling(_) => true,
        }
    }
    
    // Restore state from another tool instance
    pub fn restore_state(&mut self, other: &Self) {
        // For now, we only restore the state type, not the internal state
        // In a real implementation, we would copy the internal state as well
        match (self, other) {
            // Only restore if the state types match
            (Self::Active(_), Self::Active(_)) => {},
            (Self::TextureSelected(_), Self::TextureSelected(_)) => {},
            (Self::ScalingEnabled(_), Self::ScalingEnabled(_)) => {},
            (Self::Scaling(_), Self::Scaling(_)) => {},
            // If state types don't match, replace with the appropriate state
            (self_ref @ Self::Active(_), Self::TextureSelected(_)) => {
                *self_ref = Self::TextureSelected(SelectionTool::<TextureSelected>::new());
            },
            (self_ref @ Self::Active(_), Self::ScalingEnabled(_)) => {
                *self_ref = Self::ScalingEnabled(SelectionTool::<ScalingEnabled>::new());
            },
            (self_ref @ Self::Active(_), Self::Scaling(_)) => {
                // Create a dummy element for the default implementation
                // This is not ideal but necessary for state restoration
                let dummy_element = ElementType::Stroke(Arc::new(crate::stroke::Stroke::new(
                    egui::Color32::RED,
                    1.0,
                    vec![egui::Pos2::new(0.0, 0.0), egui::Pos2::new(10.0, 10.0)],
                )));
                
                *self_ref = Self::Scaling(SelectionTool::<Scaling>::new(
                    dummy_element,
                    Corner::TopLeft,
                    egui::Pos2::new(0.0, 0.0)
                ));
            },
            // Other combinations - do nothing for now
            _ => {},
        }
    }

    /// Get the current configuration
    pub fn get_config(&self) -> Box<dyn ToolConfig> {
        // For now, we just return a basic config
        // In the future, we could store more settings
        Box::new(SelectionToolConfig {})
    }
    
    /// Apply a configuration
    pub fn apply_config(&mut self, config: &dyn ToolConfig) {
        if let Some(_cfg) = config.as_any().downcast_ref::<SelectionToolConfig>() {
            // Apply any configuration settings here
            // For now, there's nothing to apply
        }
    }
}

// Factory function to create a new SelectionToolType
pub fn new_selection_tool() -> SelectionToolType {
    SelectionToolType::new()
}

// Helper function to check if a position is over a resize handle
// NOTE: Must match logic in HitTestCache::is_point_near_any_handle()
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
        // Create a dummy element for the default implementation
        // This is not ideal but necessary for the Default trait
        let dummy_element = ElementType::Stroke(Arc::new(crate::stroke::Stroke::new(
            egui::Color32::RED,
            1.0,
            vec![egui::Pos2::new(0.0, 0.0), egui::Pos2::new(10.0, 10.0)],
        )));
        
        Self::new(
            dummy_element,
            Corner::TopLeft,
            egui::Pos2::new(0.0, 0.0)
        )
    }
}

impl ElementType {
    fn get_id(&self) -> usize {
        match self {
            ElementType::Stroke(s) => std::sync::Arc::as_ptr(s) as usize,
            ElementType::Image(i) => i.id(),
        }
    }

    fn rect(&self) -> Rect {
        match self {
            ElementType::Image(image) => image.rect(),
            ElementType::Stroke(_) => Rect::NOTHING, // TODO: Implement for strokes
        }
    }
}

impl<T> SelectionTool<T> {
    fn contains_point(&self, pos: Pos2) -> bool {
        if let Some(element) = &self.element {
            match element {
                ElementType::Image(image) => image.rect().contains(pos),
                ElementType::Stroke(_) => false, // TODO: Implement for strokes
            }
        } else {
            false
        }
    }
}