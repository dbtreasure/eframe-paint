# Document Audit

This section details all identified usages of the `Document` struct and its methods within the codebase. This is a crucial step in preparing for the state consolidation refactor.

## Usages of `Document`

The `Document` struct is used in the following locations:

- **`src/app.rs`:**

  - **`PaintApp` struct:** A `Document` instance is a member of the `PaintApp` struct (`15:110:src/app.rs`). This is the primary usage, where the document is managed.
    ```rust
    15|pub struct PaintApp {
    16|    renderer: Renderer,
    17|    document: Document,
    18|    state: EditorState,
    ```
  - **`PaintApp::new()`:** A new `Document` is created when `PaintApp` is initialized (`44:44:src/app.rs`).
    ```rust
    44|            document: Document::new(),
    ```
  - **`PaintApp::update()`:** The `Document` is accessed within the main update loop for various purposes:
    - Passed to `central_panel` to render the document. (`117:117:src/app.rs`)
      ```rust
      117|            .show(ctx, |ui| central_panel(ui, &mut self.document, &self.state, &mut self.renderer))
      ```
    - Passed to `route_event` to handle input events that might modify the document. (`136:136:src/app.rs`)
      ```rust
      136|                route_event(event.clone(), &mut self.document, &mut self.state, &mut self.renderer);
      ```
    - Passed to the active tool's `on_event` method. (`146:146:src/app.rs`)
      ```rust
      146|                        let cmd = active_tool.on_event(event.clone(), &mut self.document, &self.state);
      ```
    - Passed to `tool_ref.on_pointer_down` within the `with_tool_mut` closure. (`218:218:src/app.rs`)
      ```rust
      218|                        cmd_result = tool_ref.on_pointer_down(position, document, &state_copy);
      ```
    - The document is marked as modified after command execution. (`183:183:src/app.rs`)
      ```rust
      183|        self.document.mark_modified();
      ```
  - **`PaintApp::execute_command()`:** The `Document` is mutated by the `CommandHistory::execute` method. (`178:178:src/app.rs`)
    ```rust
    178|        self.command_history.execute(command.clone(), &mut self.document);
    ```
  - **`PaintApp::process_dropped_file()`** The document is modified by adding a new image.

- **`src/panels/central_panel.rs`:**

  - **`central_panel()` function:** The `Document` is accessed for:
    - Iterating through strokes and images for rendering.
    - Calling `element_at_position()` to determine if an element is under the cursor.
    - Calling `get_element_mut()` to modify a selected element.

- **`src/command.rs`:**

  - **`Command` enum variants:** Several command variants operate directly on the `Document`:
    - `AddStroke`: Adds a stroke to the document.
    - `AddImage`: Adds an image to the document.
    - `MoveElement`: Modifies an element within the document (either stroke or image).
    - `ResizeElement`: Modifies an element within the document.
    - `ReplaceStroke`: Replaces a stroke in the document.
    - `ReplaceImage`: Replaces an image in the document.
  - **`Command::apply()`:** This method mutates the `Document` based on the specific command variant.
  - **`Command::unapply()`:** This method reverts changes to the `Document` based on the command.

- **`src/tools/*.rs` (Various Tool Implementations):**

  - Tools like `DrawStrokeTool`, `SelectionTool`, etc., receive a reference to the `Document` in their `on_pointer_down`, `on_pointer_up`, `on_pointer_move`, and `draw` methods. They use this to:
    - Access existing elements (for selection, hit-testing).
    - Potentially create new elements (e.g., `DrawStrokeTool` creates strokes).
    - Generate commands that will modify the document.

- **`src/file_handler.rs`:**
  - **`FileHandler::process_dropped_file()`:** Creates an `AddImage` command to add a dropped image to the document.

## `Document` Methods Used

The following methods of the `Document` struct are used throughout the codebase:

- `new()`: Creates a new document.
- `add_stroke()`: Adds a stroke to the document.
- `strokes()`: Returns a slice of strokes.
- `strokes_mut()`: Returns a mutable slice of strokes.
- `remove_last_stroke()`: Removes the last stroke.
- `add_image()`: Adds an image to the document.
- `images()`: Returns a slice of images.
- `images_mut()`: Returns a mutable slice of images.
- `remove_last_image()`: Removes the last image.
- `find_image_by_id()`: Finds an image by its ID.
- `find_stroke_by_id()`: Finds a stroke by its ID.
- `find_element_by_id()`: Finds an element (stroke or image) by its ID.
- `contains_element()`: Checks if an element with a given ID exists.
- `get_element_mut()`: Returns a mutable reference to an element.
- `element_at_position()`: Finds the element at a given position.
- `get_element_by_id()`: Retrieves an element by ID.
- `replace_stroke_by_id()`: Replaces a stroke with a new one.
- `replace_image_by_id()`: Replaces an image with a new one.
- `mark_modified()`: Increments the document version.
- `version()`: Returns the current document version.
- `increment_version()`: Increments the document version (less common than `mark_modified()`).
- `element_draw_index()`: Gets the draw order index of an element.

## Summary and Implications for Consolidation

The audit reveals that `Document` is central to the application's data management, as expected. It's primarily accessed and modified within `PaintApp`, but also significantly within the command system and individual tools. The widespread use and mutation of `Document` highlight the importance of consolidating it with `EditorState`. The numerous methods for accessing and modifying both strokes and images separately point to the need for the element unification refactor as a subsequent step. The versioning methods (`mark_modified`, `version`, `increment_version`) are also key indicators of duplicated state with `EditorState`.

# EditorState Audit

This section details all identified usages of the `EditorState` struct and its methods within the codebase. This complements the `Document` audit and is crucial for the state consolidation refactor.

## Usages of `EditorState`

The `EditorState` struct is used in the following locations:

- **`src/app.rs`:**

  - **`PaintApp` struct:** An `EditorState` instance is a member of the `PaintApp` struct (`18:110:src/app.rs`).
    ```rust
    15|pub struct PaintApp {
    16|    renderer: Renderer,
    17|    document: Document,
    18|    state: EditorState,
    ```
  - **`PaintApp::new()`:** A new `EditorState` is created when `PaintApp` is initialized (`48:48:src/app.rs`).
    ```rust
    48|            state: EditorState::new(),
    ```
  - **`PaintApp::update()`:** The `EditorState` is accessed and modified within the main update loop:

    - Passed to `central_panel` for rendering and input handling. (`117:117:src/app.rs`)
      ```rust
      117|            .show(ctx, |ui| central_panel(ui, &mut self.document, &self.state, &mut self.renderer))
      ```
    - Passed to `route_event` to handle input events. (`136:136:src/app.rs`)
      ```rust
      136|                route_event(event.clone(), &mut self.document, &mut self.state, &mut self.renderer);
      ```
    - Passed to the active tool's `on_event` method. (`146:146:src/app.rs`)
      ```rust
      146|                        let cmd = active_tool.on_event(event.clone(), &mut self.document, &self.state);
      ```
    - The active tool is retrieved using `state.active_tool()`. (`153:153:src/app.rs`)
      ```rust
      153|        if let Some(active_tool) = &mut self.state.active_tool() {
      ```
    - `state.with_tool_mut` is used to access and modify the active tool. (`215:215:src/app.rs`)

      ```rust
      215|                state.with_tool_mut(|active_tool| {
      ```

    - The `EditorState` is updated after executing a command to potentially update the selection. (`187:187:src/app.rs`)
      ```rust
      187|            self.state = self.state.with_selected_elements(vec![element_type]);
      ```
    - The `EditorState` is updated when setting the active tool. (`88:90:src/app.rs`)
      ```rust
      88|        self.state = self.state
      89|            .update_tool(|_| Some(tool_clone))
      90|            .update_selection(|_| vec![]);
      ```

  - **`PaintApp::set_active_tool()`:** The `EditorState`'s active tool is updated. (`88:90:src/app.rs`)
    ```rust
    88|        self.state = self.state
    89|            .update_tool(|_| Some(tool_clone))
    90|            .update_selection(|_| vec![]);
    ```
  - **`PaintApp::active_tool()`:** Retrieves the currently active tool from the `EditorState`.
  - **`PaintApp::execute_command()`:** The editor state is updated with the newly created/selected element.

- **`src/panels/central_panel.rs`:**

  - **`central_panel()` function:** The `EditorState` is accessed for:

    - Getting the active tool.
    - Updating the selection.
    - Passed down to `handle_input_event`.

  - **`CentralPanel::handle_input_event()`:**
    - The `EditorState` is mutated within this function, primarily to update the selection and active tool.

- **`src/tools/*.rs` (Various Tool Implementations):**

  - Tools receive a reference to the `EditorState` in methods like `on_pointer_down`, `on_pointer_up`, and `on_pointer_move`. They use this to:
    - Access the currently selected elements.
    - Access the active tool.
    - Modify the selection (especially `SelectionTool`).

- **`src/renderer.rs`:**
  - **`Renderer::render()`:** Receives the selected elements, derived from `EditorState`.

## `EditorState` Methods Used

The following methods of the `EditorState` struct are used:

- `new()`: Creates a new `EditorState`.
- `builder()`: Creates an `EditorStateBuilder` for modifying the state.
- `with_active_tool()`: Builder method to update the active tool (used for legacy compatibility).
- `with_selected_elements()`: Builder method to update selected elements (used for legacy compatibility).
- `active_tool()`: Returns the currently active tool.
- `selected_elements()`: **Currently unimplemented (returns an empty vector and logs a warning).** This is a significant point for the refactor.
- `update_tool()`: Updates the active tool using a closure.
- `update_selection()`: Updates the selection using a closure.
- `with_tool_mut()`: Provides mutable access to the active tool.

## `EditorStateBuilder` Methods Used

- `with_active_tool()`
- `with_selected_element_ids()`
- `build()`

## Summary and Implications for Consolidation

The audit of `EditorState` reveals its role in managing the active tool and selection, both of which are crucial for UI interaction. The use of the builder pattern (and its planned removal) is noted. The most important finding is the **unimplemented `selected_elements()` method**, which highlights a significant gap in the current state management. The `EditorState`'s version field is also a clear duplication with `Document`. The heavy interaction between `EditorState` and `PaintApp`, `central_panel`, and the tools reinforces the need for consolidation with `Document`. The fact that selection is handled in multiple places (within `EditorState`, within the `SelectionTool`'s internal state, and indirectly via commands) is a major source of complexity that the consolidation should address.
