# Chapter 6: Undoing Mistakes – Command System and Document Refactoring

Every creative process involves mistakes, and a robust paint application must give users the power to undo and redo their actions. In this chapter, we introduce a command system that encapsulates drawing actions so that they can be reversed and replayed. This is a critical feature for any creative tool, and here we'll explain how we implemented it using Rust.

Our goals in this chapter are to:

- **Explain the Command Pattern:** How wrapping an action in a command lets us easily undo and redo changes.
- **Refactor the Document Structure:** How separating Stroke, Layer, and Command into their own modules improves code organization.
- **Integrate Undo/Redo in the UI:** How new buttons trigger undo and redo operations.
- **Show the Full Picture:** With code examples and simplified explanations, we'll illustrate the entire pipeline from committing a stroke to undoing a mistake.

---

## 1. The Command Pattern: A Primer

The Command Pattern is a design pattern where actions—like adding a stroke to the canvas—are encapsulated into command objects. These objects include all the information necessary to perform the action. In our case, we defined a `Command` enum to represent undoable actions. For now, it includes a single variant:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Adds a stroke to a specific layer
    AddStroke {
        /// The index of the layer to add the stroke to
        layer_index: usize,
        /// The stroke to add
        stroke: Stroke,
    },
    // Future commands can be added here as needed.
}
```

This enum allows us to capture the action of adding a stroke, including which layer should receive the stroke and the stroke data itself.

---

## 2. Committing a Stroke with Commands

Previously, when a user finished drawing a stroke, the stroke was simply appended to the current layer. Now, we wrap that process in a command. In the `PaintApp` structure, a new method—`commit_current_stroke`—uses Rust's `std::mem::take` to remove the current stroke, then wraps it into a command and tells the document to execute it:

```rust
fn commit_current_stroke(&mut self) {
    let stroke = std::mem::take(&mut self.current_stroke);
    if let Some(active_layer) = self.document.active_layer {
        let command = Command::AddStroke {
            layer_index: active_layer,
            stroke,
        };
        self.document.execute_command(command);
    }
}
```

This approach decouples stroke creation from stroke application. The document, responsible for overall state, now manages a stack of commands, making it possible to undo or redo each action.

---

## 3. The Document's New Structure: Undo and Redo

The `Document` structure has been refactored to include two new stacks:

- **undo_stack:** Stores the history of commands that have been executed.
- **redo_stack:** Temporarily holds commands that were undone, allowing them to be replayed if needed.

### Executing a Command

When a new command is executed, the document applies the command and then pushes it onto the undo stack. It also clears the redo stack, ensuring that new actions invalidate any redo history:

```rust
pub fn execute_command(&mut self, command: Command) {
    match &command {
        Command::AddStroke { layer_index, stroke } => {
            if let Some(layer) = self.layers.get_mut(*layer_index) {
                layer.add_stroke(stroke.clone());
            }
        }
    }
    self.undo_stack.push(command);
    self.redo_stack.clear(); // Clear redo history on new command
}
```
