# Chapter 7: Polishing the Command System and Advanced Document Management

As our paint application matures, we need to refine not only how we draw, but also how we manage and organize user actions. In this chapter, we focus on:

- **Command Grouping:** Merging multiple strokes into a single, undoable operation.
- **Enhanced Document Management:** Improving our undo/redo mechanism and providing utilities to reset and refine command history.
- **UI Enhancements:** Polishing the undo/redo controls by adding extra buttons for merging commands and clearing history.

For engineers new to Rust, these enhancements may seem daunting, but we'll break down each concept, provide simple code examples, and explain the underlying Rust principles, such as pattern matching, ownership, and modular design.

---

## 1. Enhanced Command Grouping

Earlier, every stroke was a separate command. Now, we introduce command grouping, which allows consecutive strokes to be merged into one action. This makes undo operations more natural — if you draw a continuous line, you want to undo it in one go rather than stroke by stroke.

Here's a simplified helper function that demonstrates how two AddStroke commands might be merged:

```rust
/// Merges the last two AddStroke commands if they belong to the same layer.
pub fn merge_last_commands(&mut self) {
    if self.undo_stack.len() >= 2 {
        let last = &self.undo_stack[self.undo_stack.len() - 1];
        let second_last = &self.undo_stack[self.undo_stack.len() - 2];
        if let (Command::AddStroke { layer_index: idx1, stroke: stroke1 },
                Command::AddStroke { layer_index: idx2, stroke: stroke2 }) = (last, second_last) {
            if idx1 == idx2 {
                // Merge by concatenating the stroke points
                let mut merged_stroke = stroke2.clone();
                merged_stroke.points.extend_from_slice(&stroke1.points);
                // Remove the last two commands and push the merged command
                self.undo_stack.pop();
                self.undo_stack.pop();
                self.undo_stack.push(Command::AddStroke {
                    layer_index: *idx1,
                    stroke: merged_stroke,
                });
            }
        }
    }
}
```

This function uses Rust's pattern matching to safely extract and merge the strokes if they originate from the same layer. Notice how the match ensures type safety and clarity.

---

## 2. Improved Document Management

Our document now not only holds layers and strokes, but also two stacks: an `undo_stack` and a `redo_stack`. To enable users to clear any lingering state, we add a utility method to reset both stacks:

```rust
pub fn clear_history(&mut self) {
    self.undo_stack.clear();
    self.redo_stack.clear();
}
```

By refactoring our document into separate modules (for strokes, layers, and commands), we maintain a clean architecture that is easier to maintain and extend. This modularity is a recurring pattern in Rust development, emphasizing separation of concerns and encapsulation.

---

## 3. UI Enhancements for Command Management

A great feature is only as good as its accessibility. In our updated UI, we've added new buttons so users can directly perform command-related actions. Below is an excerpt from our update loop in `PaintApp` that integrates these controls:

```rust
ui.separator();
ui.horizontal(|ui| {
    if ui.button("⟲ Undo").clicked() {
        self.document.undo();
    }
    if ui.button("⟳ Redo").clicked() {
        self.document.redo();
    }
    // Button to merge the last two commands for batch undo
    if ui.button("Merge Last").clicked() {
        self.document.merge_last_commands();
    }
    // Button to clear all history
    if ui.button("Clear History").clicked() {
        self.document.clear_history();
    }
});
```

These controls enhance the user experience by giving direct access to advanced command operations. They also serve to illustrate how UI elements in an immediate mode GUI can directly invoke complex logic without the overhead of traditional event handling.

---

## Wrapping Up

Let's recap what we've achieved in Chapter 7:

- **Command Grouping:** We introduced a method to merge consecutive AddStroke commands into a single operation, making undo actions more intuitive.
- **Enhanced Document Management:** By adding methods like `clear_history`, we provide robust control over our command stacks, ensuring a fresh start when needed.
- **UI Enhancements:** Extra buttons in the UI now allow users to undo, redo, merge commands, and clear history—all of which improve the overall usability of the application.

These enhancements not only refine the architecture of our application but also demonstrate Rust's strengths in pattern matching, modular design, and safe state management. Through these changes, our application becomes increasingly powerful and user-friendly, paving the way for even more creative features down the road.

Welcome to the next level of creative control—where each command is polished, every history can be managed, and your art is always at your command!
