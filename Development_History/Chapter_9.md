# Chapter 9: Revamping Layer Management – Reordering and Renaming Enhancements

In this chapter, we document a significant enhancement that makes our layer management more intuitive and flexible. With commit [80989fbcd9adebfd1293a0cebcd3c94e6cf3afa2](#), we introduced dynamic drag-and-drop reordering and in-place renaming of layers. These changes empower users to control the organization and presentation of their artwork with ease.

## Introduction

Before this update, layers were managed in a static order with fixed names. Recognizing the need for a more interactive approach, we overhauled the layer management system so that users can reorder layers through drag-and-drop and rename them directly via a double-click interface. This commit not only refines the user experience but also simplifies the underlying logic by encapsulating these actions into defined commands.

## Implementation Details

### Layer Reordering

The commit adds a new command variant, `ReorderLayer`, which encapsulates the reordering operation. In the updated UI, users can simply drag a layer to a new position. The system calculates the target position based on the pointer's location and then triggers the command to update the layers' order accordingly. In the Document module, this command adjusts the underlying layers array and updates the active layer index when necessary, ensuring that the drawing order remains consistent.

### Layer Renaming

To improve expressiveness, we added in-place renaming functionality through the new `RenameLayer` command. By double-clicking a layer name in the side panel, users can seamlessly edit it. Once the field loses focus, if a change is detected, a command is issued to update the layer's name. In this way, the change becomes part of the document's command history, allowing for potential undo/redo support.

### Code Integration

The updates span multiple components:

- **App Module:** Enhancements include handling drag events and double-click actions in the layers panel. The UI now displays drag indicators and provides an editable text field when renaming a layer.
- **Command Module:** Two new command types—`ReorderLayer` and `RenameLayer`—were introduced. These commands encapsulate the necessary data (such as from/to indices for reordering and old/new names for renaming) and implement both execution and reversal logic.
- **Document Module:** The Document processes these commands by updating the layers array accordingly. It carefully adjusts indices to maintain consistency, ensuring that the active layer remains correctly set after reordering or renaming.

## Code Samples

Below are concise code snippets illustrating the new functionality:

### Reorder Command Definition

```rust
pub enum Command {
    // ... existing command variants ...
    /// Reorders a layer from one position to another
    ReorderLayer {
        from_index: usize,
        to_index: usize,
    },
    /// Renames a layer
    RenameLayer {
        layer_index: usize,
        old_name: String,
        new_name: String,
    },
}
```

### Handling Layer Reordering in the Document

```rust
impl Document {
    pub fn execute_command(&mut self, command: Command) {
        match &command {
            Command::ReorderLayer { from_index, to_index } => {
                if *from_index < self.layers.len() && *to_index < self.layers.len() {
                    let layer = self.layers.remove(*from_index);
                    self.layers.insert(*to_index, layer);
                    // Adjust active layer index if necessary
                    if let Some(active_idx) = self.active_layer {
                        self.active_layer = Some(if active_idx == *from_index {
                            *to_index
                        } else {
                            active_idx
                        });
                    }
                }
            },
            Command::RenameLayer { layer_index, new_name, .. } => {
                if let Some(layer) = self.layers.get_mut(*layer_index) {
                    layer.name = new_name.clone();
                }
            },
            _ => {}
        }
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }
}
```

### UI Integration for Drag-and-Drop and Renaming

Within the app's layer panel, event handling is updated to capture drag gestures and double-click events for renaming:

```rust
// In the App's update loop for the layers panel...
for (idx, layer) in self.document.layers.iter().enumerate() {
    let layer_response = ui.allocate_rect(layer_rect, egui::Sense::click_and_drag());
    if layer_response.dragged() {
        self.dragged_layer = Some(idx);
    }

    // Editable label on double-click
    if layer_response.double_clicked() {
        self.editing_layer_name = Some(idx);
    }
    // ... additional UI logic for visibility toggles and active layer selection
}
```

## Conclusion

The enhancements documented in this chapter significantly elevate the usability of our paint application. With dynamic layer reordering and seamless in-place renaming, users gain finer control over their creative process. This update not only simplifies layer management but also lays the groundwork for future improvements, such as animated transitions and expanded command history features.

Building on these changes, future work may explore more sophisticated UI feedback during reordering and deeper integration with undo/redo mechanisms, ensuring that our application continues to evolve along with user needs.
