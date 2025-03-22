use crate::element::{Element, ElementType};
use crate::renderer::Renderer;
use crate::state::EditorModel;
use crate::widgets::resize_handle::Corner;
use egui;
use log;

// Image resizing functionality has been moved to the element implementation

#[derive(Clone, Debug)]
pub enum Command {
    AddElement {
        element: ElementType,
    },
    RemoveElement {
        element_id: usize,
        old_element: ElementType, // Store removed element for undo
    },
    MoveElement {
        element_id: usize,
        delta: egui::Vec2,
        old_position: egui::Pos2, // Store original position for undo
    },
    ResizeElement {
        element_id: usize,
        corner: Corner,
        new_position: egui::Pos2,
        old_rect: egui::Rect, // Store original rect for undo
    },
    // Selection commands remain mostly unchanged
    SelectElement(usize),
    DeselectElement(usize),
    ClearSelection,
    ToggleSelection(usize),
}

impl Command {
    /// Handle texture invalidation after command execution
    ///
    /// This method leverages the unified Element trait approach for consistent
    /// texture invalidation across all element types.
    pub fn invalidate_textures(&self, renderer: &mut Renderer) {
        match self {
            Command::AddElement { element } => {
                log::info!("🧹 Invalidating texture for new element {}", element.id());
                // Clear any existing texture for this element ID
                renderer.clear_element_state(element.id());

                // Create a mutable clone to invalidate the texture
                let mut element_clone = element.clone();
                element_clone.invalidate_texture();
            }
            Command::RemoveElement { element_id, .. } => {
                log::info!("🧹 Invalidating texture for removed element {}", element_id);
                // Clean up all texture state for this element
                renderer.clear_element_state(*element_id);
            }
            Command::ResizeElement { element_id, .. } => {
                log::info!("🧹 Invalidating texture for resized element {}", element_id);

                // First clear by ID to remove any stale textures
                renderer.clear_element_state(*element_id);

                // For resize operations, always reset all element state to be safe
                // This is because resize can affect the texture generation parameters
                renderer.clear_all_element_state();
            }
            Command::MoveElement { element_id, .. } => {
                log::info!("🧹 Invalidating texture for moved element {}", element_id);

                // Clear element state for this specific element
                renderer.clear_element_state(*element_id);

                // For elements that may have complex rendering (like strokes),
                // we perform a more thorough invalidation
                if let Some(element) = renderer.find_element(*element_id) {
                    // Check element type and apply specific invalidation if needed
                    if element.element_type() == "stroke" {
                        log::info!("🧹 Extra invalidation for stroke element {}", element_id);
                        renderer.invalidate_texture(*element_id);
                    }
                } else {
                    // If element not found, clear all state to be safe
                    renderer.clear_all_element_state();
                }
            }
            // Selection commands don't need texture invalidation
            Command::SelectElement(_)
            | Command::DeselectElement(_)
            | Command::ClearSelection
            | Command::ToggleSelection(_) => {
                // Just request a repaint to ensure the UI updates for selection changes
                renderer.get_ctx().request_repaint();
            }
        }

        // Always request a repaint to ensure changes are visible
        renderer.get_ctx().request_repaint();
    }

    /// Execute a command on the editor model
    ///
    /// This method applies the command to the editor model and returns a Result
    /// to indicate success or failure. The result contains an error message if
    /// the command execution failed.
    pub fn execute(&self, editor_model: &mut EditorModel) -> Result<(), String> {
        match self {
            Command::AddElement { element } => {
                log::info!(
                    "💻 Executing AddElement command for element {} (type: {})",
                    element.id(),
                    element.element_type()
                );

                // Clone the element since we need to add it to the editor model
                let new_element = element.clone();

                // Add the element to the editor model
                editor_model.add_element(new_element);
                editor_model.mark_modified();

                Ok(())
            }
            Command::RemoveElement {
                element_id,
                old_element: _,
            } => {
                log::info!(
                    "💻 Executing RemoveElement command for element {}",
                    element_id
                );

                // Remove the element from the editor model
                if editor_model.remove_element_by_id(*element_id).is_none() {
                    return Err(format!("Element with id {} not found", element_id));
                }

                editor_model.mark_modified();
                Ok(())
            }
            Command::MoveElement {
                element_id,
                delta,
                old_position: _,
            } => {
                log::info!(
                    "💻 Executing MoveElement command: element={}, delta={:?}",
                    element_id,
                    delta
                );

                // Take ownership of the element
                let mut element = editor_model
                    .take_element_by_id(*element_id)
                    .ok_or_else(|| format!("Element with id {} not found", element_id))?;

                // Translate the element using the Element trait method
                element.translate(*delta)?;

                // Invalidate the texture
                element.invalidate_texture();

                // Return ownership to the model
                editor_model.add_element(element);
                editor_model.mark_modified();

                Ok(())
            }
            Command::ResizeElement {
                element_id,
                corner,
                new_position,
                old_rect: _,
            } => {
                log::info!(
                    "💻 Executing ResizeElement command for element {}",
                    element_id
                );

                // Find the element and get its current rect
                let current_rect = editor_model
                    .find_element_by_id(*element_id)
                    .ok_or_else(|| format!("Element with id {} not found", element_id))?
                    .rect();

                // Compute the new rectangle based on the corner and new position
                let new_rect = Renderer::compute_resized_rect(current_rect, *corner, *new_position);

                log::info!(
                    "📐 Resizing element {} from {:?} to {:?}",
                    element_id,
                    current_rect,
                    new_rect
                );

                // Take ownership of the element
                let mut element = editor_model
                    .take_element_by_id(*element_id)
                    .ok_or_else(|| format!("Element with id {} not found", element_id))?;

                // Resize the element using the Element trait method
                element.resize(new_rect)?;

                // Invalidate the texture
                element.invalidate_texture();

                // Return ownership to the model
                editor_model.add_element(element);
                editor_model.mark_modified();

                Ok(())
            }
            Command::SelectElement(element_id) => {
                log::info!(
                    "💻 Executing SelectElement command for element {}",
                    element_id
                );
                editor_model.select_element(*element_id);
                Ok(())
            }
            Command::DeselectElement(element_id) => {
                log::info!(
                    "💻 Executing DeselectElement command for element {}",
                    element_id
                );
                editor_model.deselect_element(*element_id);
                Ok(())
            }
            Command::ClearSelection => {
                log::info!("💻 Executing ClearSelection command");
                editor_model.clear_selection();
                Ok(())
            }
            Command::ToggleSelection(element_id) => {
                log::info!(
                    "💻 Executing ToggleSelection command for element {}",
                    element_id
                );
                editor_model.toggle_selection(*element_id);
                Ok(())
            }
        }
    }

    /// Undo a command that was previously executed
    ///
    /// This method reverts the changes made by the command and returns a Result
    /// to indicate success or failure. The result contains an error message if
    /// the undo operation failed.
    pub fn undo(&self, editor_model: &mut EditorModel) -> Result<(), String> {
        match self {
            Command::AddElement { element } => {
                log::info!("↩️ Undoing AddElement command for element {}", element.id());

                // Remove the added element
                if editor_model.remove_element_by_id(element.id()).is_none() {
                    return Err(format!(
                        "Failed to remove element {} during undo",
                        element.id()
                    ));
                }

                editor_model.mark_modified();
                Ok(())
            }
            Command::RemoveElement {
                element_id: _,
                old_element,
            } => {
                log::info!(
                    "↩️ Undoing RemoveElement command for element {}",
                    old_element.id()
                );

                // Re-add the removed element
                editor_model.add_element(old_element.clone());
                editor_model.mark_modified();
                Ok(())
            }
            Command::MoveElement {
                element_id,
                delta: _,
                old_position,
            } => {
                log::info!("↩️ Undoing MoveElement command for element {}", element_id);

                // Take ownership of the element
                let mut element = editor_model
                    .take_element_by_id(*element_id)
                    .ok_or_else(|| format!("Element with id {} not found", element_id))?;

                // Get the current position
                let current_pos = element.rect().min;

                // Calculate the delta to move back to the original position
                let reverse_delta = *old_position - current_pos;

                log::info!("🔙 Moving element back with delta {:?}", reverse_delta);

                // Translate the element back to its original position
                element.translate(reverse_delta)?;

                // Invalidate the texture
                element.invalidate_texture();

                // Return ownership to the model
                editor_model.add_element(element);
                editor_model.mark_modified();

                Ok(())
            }
            Command::ResizeElement {
                element_id,
                corner: _,
                new_position: _,
                old_rect,
            } => {
                log::info!(
                    "↩️ Undoing ResizeElement command for element {}",
                    element_id
                );

                // Take ownership of the element
                let mut element = editor_model
                    .take_element_by_id(*element_id)
                    .ok_or_else(|| format!("Element with id {} not found", element_id))?;

                log::info!("🔙 Resizing element back to original rect {:?}", old_rect);

                // Resize the element back to its original rectangle
                element.resize(*old_rect)?;

                // Invalidate the texture
                element.invalidate_texture();

                // Return ownership to the model
                editor_model.add_element(element);
                editor_model.mark_modified();

                Ok(())
            }
            Command::SelectElement(element_id) => {
                log::info!(
                    "↩️ Undoing SelectElement command for element {}",
                    element_id
                );
                // Undo a selection by deselecting the element
                editor_model.deselect_element(*element_id);
                Ok(())
            }
            Command::DeselectElement(element_id) => {
                log::info!(
                    "↩️ Undoing DeselectElement command for element {}",
                    element_id
                );
                // Undo a deselection by selecting the element
                editor_model.select_element(*element_id);
                Ok(())
            }
            Command::ClearSelection => {
                // This is harder to undo properly without storing the previous selection
                log::warn!(
                    "⚠️ Cannot properly undo ClearSelection without storing previous selection"
                );
                Err("Cannot undo clear selection - previous selection not stored".to_string())
            }
            Command::ToggleSelection(element_id) => {
                log::info!(
                    "↩️ Undoing ToggleSelection command for element {}",
                    element_id
                );
                // Undo a toggle by toggling again
                editor_model.toggle_selection(*element_id);
                Ok(())
            }
        }
    }

    // Keep these methods for backward compatibility during transition

    /// DEPRECATED: Use execute() instead
    pub fn apply_to_editor_model(&self, editor_model: &mut EditorModel) {
        log::warn!("⚠️ apply_to_editor_model is deprecated, use execute() instead");

        // Call the new execute method and ignore any errors
        let _ = self.execute(editor_model);
    }

    /// DEPRECATED: Use undo() instead
    pub fn unapply_from_editor_model(&self, editor_model: &mut EditorModel) {
        log::warn!("⚠️ unapply_from_editor_model is deprecated, use undo() instead");

        // Call the new undo method and ignore any errors
        let _ = self.undo(editor_model);
    }

    // Stub helper methods have been removed as they are no longer needed
}

pub struct CommandHistory {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Execute a command on an EditorModel
    ///
    /// Returns a Result indicating success or failure. If successful, the command
    /// is added to the undo stack and the redo stack is cleared.
    pub fn execute(
        &mut self,
        command: Command,
        editor_model: &mut EditorModel,
    ) -> Result<(), String> {
        // Execute the command and handle any errors
        match command.execute(editor_model) {
            Ok(()) => {
                // Clear the redo stack when a new command is executed
                self.redo_stack.clear();

                // Add the command to the undo stack
                self.undo_stack.push(command);

                Ok(())
            }
            Err(e) => {
                log::error!("⚠️ Command execution failed: {}", e);
                Err(e)
            }
        }
    }

    /// Undo a command on an EditorModel
    ///
    /// Returns a Result indicating success or failure. If successful, the command
    /// is moved from the undo stack to the redo stack.
    pub fn undo(&mut self, editor_model: &mut EditorModel) -> Result<(), String> {
        if let Some(command) = self.undo_stack.pop() {
            // Try to undo the command
            match command.undo(editor_model) {
                Ok(()) => {
                    // Add the command to the redo stack
                    self.redo_stack.push(command);
                    Ok(())
                }
                Err(e) => {
                    log::error!("⚠️ Command undo failed: {}", e);
                    // Put the command back on the undo stack if it fails
                    self.undo_stack.push(command);
                    Err(e)
                }
            }
        } else {
            let msg = "Nothing to undo".to_string();
            log::info!("{}", msg);
            Err(msg)
        }
    }

    /// Redo a command on an EditorModel
    ///
    /// Returns a Result indicating success or failure. If successful, the command
    /// is moved from the redo stack to the undo stack.
    pub fn redo(&mut self, editor_model: &mut EditorModel) -> Result<(), String> {
        if let Some(command) = self.redo_stack.pop() {
            // Try to execute the command
            match command.execute(editor_model) {
                Ok(()) => {
                    // Add the command to the undo stack
                    self.undo_stack.push(command);
                    Ok(())
                }
                Err(e) => {
                    log::error!("⚠️ Command redo failed: {}", e);
                    // Put the command back on the redo stack if it fails
                    self.redo_stack.push(command);
                    Err(e)
                }
            }
        } else {
            let msg = "Nothing to redo".to_string();
            log::info!("{}", msg);
            Err(msg)
        }
    }

    /// DEPRECATED: Execute a command without error handling (for backward compatibility)
    pub fn execute_ignore_errors(&mut self, command: Command, editor_model: &mut EditorModel) {
        log::warn!("⚠️ execute_ignore_errors is deprecated, use execute() instead");

        // Call the new execute method and ignore any errors
        let _ = self.execute(command, editor_model);
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_stack(&self) -> &[Command] {
        &self.undo_stack
    }

    pub fn redo_stack(&self) -> &[Command] {
        &self.redo_stack
    }
}
