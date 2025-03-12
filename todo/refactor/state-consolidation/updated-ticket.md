# State Consolidation Sub-Tasks

This document outlines the refined sub-tasks for consolidating `Document` and `EditorState` into a unified `EditorModel`.

## Phase 1: Core Structure and Migration

These tasks focus on creating the `EditorModel` and migrating the essential data. This is the foundation for all subsequent changes.

1.  **Create `EditorModel` Struct:**

    - Create the `src/state.rs` file (or modify the existing one) and define the `EditorModel` struct:

      ```rust
      pub type ElementId = usize;

      pub struct EditorModel {
          content: Vec<ElementType>,
          version: usize,
          selected_element_ids: HashSet<ElementId>,
          active_tool: ToolType,
      }
      ```

    - Define the `ElementId` type alias.
    - Ensure the `ToolType` enum is defined (likely in `src/tools/mod.rs` or similar).

2.  **Implement `EditorModel` Basic Methods:**

    - Implement the following methods on `EditorModel`:
      - `new() -> Self`: Constructor to create a new `EditorModel` with initial values (empty `content`, `version = 0`, empty `selected_element_ids`, and a default `active_tool`).
      - `mark_modified(&mut self)`: Increments the `version` counter.
      - `get_element_by_id(&self, id: ElementId) -> Option<&ElementType>`: Retrieves an element by its ID (should eventually replace `Document`'s `find_element_by_id`).
      - `is_element_selected(&self, id: ElementId) -> bool`: Checks if an element is selected.

3.  **Migrate Fields to `EditorModel`:**

    - Move fields from `Document` and `EditorState` to `EditorModel` as per the final structure. This is a purely mechanical move, with no logic changes yet.
    - Remove the original fields from `Document` and `EditorState`.

4.  **Update `PaintApp`:**
    - Replace the `document: Document` and `state: EditorState` fields in `PaintApp` with `editor_model: EditorModel`.
    - Update `PaintApp::new()` to initialize the `EditorModel` using `EditorModel::new()`.

## Phase 2: API Updates and Integration

These tasks involve updating the rest of the codebase to use the new `EditorModel`. This is the most time-consuming phase.

5.  **Refactor `PaintApp` Methods:**

    - Systematically update _all_ methods in `PaintApp` that previously accessed `self.document` or `self.state` to now access `self.editor_model`. This includes, but is not limited to:
      - `update()`
      - `execute_command()`
      - `set_active_tool()` (and related tool-switching logic)
      - `process_dropped_file()` (if kept in `PaintApp`)
      - Any rendering-related logic that accessed the document or state.

6.  **Refactor `central_panel`:**

    - Update `central_panel()` and `CentralPanel::handle_input_event()` to accept `&mut EditorModel` instead of separate `&mut Document` and `&mut EditorState` arguments.
    - Update all code within these functions to use the `EditorModel`.

7.  **Refactor Tools:**

    - Update all tool methods (e.g., `on_pointer_down`, `on_pointer_move`, `draw`, `activate`, `deactivate`) to accept `&mut EditorModel` instead of separate `&Document` and `&EditorState` arguments.
    - Update tool code to use `EditorModel` methods (especially for selection handling in `SelectionTool`).

8.  **Refactor Command System:**

    - Update the `Command` enum and the `Command::apply()` and `Command::unapply()` methods to operate on `&mut EditorModel` instead of `&mut Document`.
    - Ensure all commands that modify the document or selection call `editor_model.mark_modified()` after making changes.

9.  **Refactor `Renderer`:**

    - Update `Renderer::render()` to accept `&EditorModel` (or relevant parts of it) instead of separate `Document` and selection information.

10. **Refactor `FileHandler` (or Merge):**
    - If keeping `FileHandler`, update it to generate commands that operate on the `EditorModel`.
    - Alternatively, consider merging its functionality into `PaintApp` or `EditorModel` (as per previous discussions).

## Phase 3: Selection Handling and Commands

These tasks focus on implementing the unified selection handling strategy.

11. **Implement Selection Commands:**

    - Create the following command variants (or similar, based on your existing command structure):
      - `SelectElement(ElementId)`
      - `DeselectElement(ElementId)`
      - `ClearSelection`
      - `ToggleSelection(ElementId)`
    - Implement the `apply()` and `unapply()` methods for these commands to modify the `selected_element_ids` field of the `EditorModel`.

12. **Update `SelectionTool` for Command Usage:**

    - Modify the `SelectionTool` to generate the selection commands (from the previous step) in response to user interactions (clicks, drags).
    - Ensure `SelectionTool` _does not_ directly modify the `selected_element_ids` field.

13. **Implement Remaining `EditorModel` Methods:**
    - Implement the following methods on `EditorModel`:
      - `select_element(&mut self, id: ElementId)`
      - `deselect_element(&mut self, id: ElementId)`
      - `clear_selection(&mut self)`
      - `toggle_selection(&mut self, id: ElementId)`
    - These methods should be called by the selection commands.

## Phase 4: Cleanup and Testing

14. **Remove `Document` and `EditorState`:**

    - Delete the `src/document.rs` and `src/state.rs` files (or remove the `Document` and `EditorState` structs if the files contain other code).
    - Remove any remaining code related to these structs.

15. **Thorough Testing:**

    - Write unit tests for all `EditorModel` methods.
    - Update _all_ existing unit and UI tests to work with the new `EditorModel`-based API.
    - Pay special attention to:
      - Undo/redo functionality.
      - Tool switching.
      - Element manipulation (adding, removing, modifying).
      - Selection handling.

16. **Documentation:**
    - Update all relevant documentation (including inline comments) to reflect the new state management system.

This detailed breakdown provides a clear roadmap for the state consolidation refactor. By dividing the work into these phases and sub-tasks, you can approach the refactoring in a manageable and organized way. Remember to commit frequently and test thoroughly after each significant change. This structured approach will minimize the risk of introducing bugs and ensure a successful transition to the new, unified state management system.
