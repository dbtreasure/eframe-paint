# Chapter 5: Tools of the Trade – Enhancing Drawing Interaction and Layer Management

In this chapter, we take a major step forward in our paint application. Building on the foundation of rendering and stroke capture, this commit introduces a suite of tools and UI enhancements that allow users to choose between different drawing modes, adjust brush properties, and manage layers. Think of it as opening your toolbox: you now have a brush, an eraser, a selection tool, and even the ability to organize your work in layers.

For those new to Rust or graphics programming, we'll break this down into simple concepts and walk through the key changes, explaining the purpose behind each feature. We'll cover:

1. **Tools Panel:** How selecting a tool (Brush, Eraser, or Selection) affects drawing.
2. **Brush Customization:** Using a color picker and a thickness slider for refining strokes.
3. **Layer Management:** How layers are handled and modified via a right-side panel.

Let's dive into each of these aspects.

---

## 1. The Tools Panel: Choosing Your Instrument

Earlier, our painting was quite basic. Now, imagine having a toolbox with multiple instruments. The tools panel is implemented as a left-side panel in our UI. It lets you choose your current drawing tool. The new `Tool` enum in our code defines the available tools:

```rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tool {
    Brush,
    Eraser,
    Selection,
}
```

Within the `Renderer` struct, we now maintain the current tool along with brush settings. This addition allows the application to adjust its behavior based on which tool is active. For example, when drawing with the eraser, the stroke color is set to the background color (commonly white).

The tools panel is rendered by the `render_tools_panel` method in the Renderer. Here's an excerpt:

```rust
pub fn render_tools_panel(&mut self, ui: &mut egui::Ui) {
    ui.heading("Tools");
    ui.separator();

    // Tool selection buttons
    ui.horizontal(|ui| {
        if ui.selectable_label(self.current_tool == Tool::Brush, "🖌 Brush").clicked() {
            self.current_tool = Tool::Brush;
        }
        if ui.selectable_label(self.current_tool == Tool::Eraser, "⌫ Eraser").clicked() {
            self.current_tool = Tool::Eraser;
        }
        if ui.selectable_label(self.current_tool == Tool::Selection, "◻ Selection").clicked() {
            self.current_tool = Tool::Selection;
        }
    });

    ui.separator();

    // Color picker
    ui.horizontal(|ui| {
        ui.label("Color:");
        egui::color_picker::color_edit_button_srgba(
            ui,
            &mut self.brush_color,
            egui::color_picker::Alpha::Opaque
        );
    });

    // Brush thickness slider
    ui.horizontal(|ui| {
        ui.label("Thickness:");
        ui.add(egui::Slider::new(&mut self.brush_thickness, 1.0..=50.0));
    });
}
```

This panel not only lets you change tools but also customize brush properties. Notice the use of the color picker and slider controls; these are built into @egui and make it straightforward to expose configuration options to the user.

---

## 2. Enhanced Drawing Interaction

Now that we have a tools panel, the drawing interaction becomes more dynamic. In the update method of `PaintApp`, when a drag event starts, the application determines the appropriate stroke properties based on the current tool.

Here's a simplified snippet showing how this decision logic integrates into the update loop:

```rust
if response.drag_started() {
    self.current_stroke.points.clear();
    if let Some(pos) = response.interact_pointer_pos() {
        if let Some(renderer) = &self.renderer {
            // Determine stroke properties based on the selected tool
            match renderer.current_tool() {
                Tool::Brush => {
                    self.current_stroke.color = renderer.brush_color();
                    self.current_stroke.thickness = renderer.brush_thickness();
                }
                Tool::Eraser => {
                    // Use background color for eraser effect
                    self.current_stroke.color = egui::Color32::WHITE;
                    self.current_stroke.thickness = renderer.brush_thickness();
                }
                Tool::Selection => {
                    // For selection, use a default thin stroke
                    self.current_stroke.color = egui::Color32::TRANSPARENT;
                    self.current_stroke.thickness = 1.0;
                }
            }
        }
        self.current_stroke.points.push((pos.x, pos.y));
    }
}
```

This example shows how different tools modify the stroke's properties. Even if you come from other programming languages, you can appreciate how the match statement in Rust cleanly handles multiple cases, ensuring that the correct behavior is applied based on the active tool.

---

## 3. Layer Management: Organizing Your Work

Beyond tools, managing layers is a crucial feature in any paint application. Layers allow you to segregate different elements of your art, making it easier to edit and organize your work.

In our commit, a right-side panel is introduced to display and manage layers. Users can view the list of layers, toggle their visibility, and add new layers with a simple button click. Here's the core idea:

```rust
// In the right-side panel for layers
egui::SidePanel::right("layers_panel").show(ctx, |ui| {
    ui.heading("Layers");
    ui.separator();

    // Iterate over layers (displayed in reverse order for top-first effect)
    for (idx, layer) in self.document.layers.iter().enumerate().rev() {
        ui.horizontal(|ui| {
            let is_active = Some(idx) == self.document.active_layer;
            if ui.selectable_label(is_active, &layer.name).clicked() {
                self.document.active_layer = Some(idx);
            }
            if ui.button(if layer.visible { "👁" } else { "👁‍🗨" }).clicked() {
                self.document.layers[idx].visible = !self.document.layers[idx].visible;
            }
        });
    }

    ui.separator();
    if ui.button("+ Add Layer").clicked() {
        self.document.add_layer(&format!("Layer {}", self.document.layers.len()));
    }
});
```

This panel gives users direct control over layer selection and visibility. Each layer is listed with a label and an icon button to toggle its visibility. This abstraction helps separate concerns in the application, making it easier to handle complex compositions.

---

## 4. Wrapping Up the Pipeline

In this commit, we have significantly enhanced the interactive aspects of the paint application:

- **Tools Panel:** Empowers users to select different drawing tools and adjust properties such as color and thickness.
- **Enhanced Drawing:** The update loop now responds dynamically to drag events, applying tool-specific behavior to stroke properties.
- **Layer Management:** Provides a clear UI for managing the different layers in your artwork, helping keep your projects organized.

These features are built on top of the previous rendering pipeline, demonstrating how Rust's safety, modularity, and pattern matching can be used to create rich and interactive applications.

---

## Conclusion

In Chapter 5, we introduced the advanced tools that elevate our paint application from a simple canvas to a fully featured art creation tool:

- We added a **tools panel** for selecting different drawing instruments, complete with a color picker and brush thickness slider.
- We enhanced the **drawing interaction** to respect the current tool, ensuring that the stroke color and thickness adjust accordingly.
- We integrated a **layers panel** that allows users to view and manage different layers of their artwork.

By combining these UI panels with Rust's powerful features, we not only improve the application's functionality but also keep the code clean and maintainable. This chapter highlights how each part of our system interlocks to provide a seamless user experience, bridging the gap between high-level UI abstraction and low-level graphics control.

Welcome to a new level of creative coding in Rust – where every tool, every layer, and every stroke is an opportunity to learn and create.
