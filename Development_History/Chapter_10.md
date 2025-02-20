# Chapter 10: Coordinated Stroke Rendering – Converting Between Screen and Document Spaces

In this chapter, we document a significant enhancement that refines the accuracy of stroke rendering. With commit [eb45de27c62ac3cbf54a1fd3dbb4b610ec2ff648](#), we implemented a robust coordinate conversion mechanism that transforms user input from screen space into document space and then back for precise rendering.

## Introduction

Digital drawing applications demand precision, especially when the drawing surface is subject to offsets and scaling. Previously, stroke input was recorded directly in screen coordinates, leading to inconsistencies when the canvas layout changed. This commit addresses that issue by establishing clear boundaries between screen space and document space. By centralizing coordinate conversion, we ensure that strokes are recorded consistently and rendered accurately under all conditions.

## Implementation Details

### Screen to Document Conversion

Input coordinates captured from user interactions are first adjusted by subtracting the canvas offset (`canvas_rect.min`). This converts the coordinates from screen space to document space, which is used for storing and processing strokes.

### Document to Screen Conversion

When rendering stroke previews, the stored document space coordinates are converted back to screen space by adding the canvas offset. This guarantees that the visual representation matches the intended input, even if the canvas is repositioned.

### Integration in the Update Loop

Both conversion processes are tightly integrated into the update cycle:

- During input handling, all captured points undergo conversion before being appended to the current stroke.
- When drawing the stroke preview, the reverse conversion is applied to align the preview with the actual viewport.

## Code Samples

### Capturing Stroke Points with Conversion

```rust
impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(pos) = input_event.pos() {
            // Convert input coordinate from screen space to document space
            let doc_pos = pos - canvas_rect.min.to_vec2();
            self.current_stroke.points.push((doc_pos.x, doc_pos.y));
        }
    }
}
```

### Rendering Stroke Previews with Conversion

```rust
impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.current_stroke.points.is_empty() {
            painter.add(egui::Shape::line(
                self.current_stroke.points.iter()
                    .map(|&(x, y)| egui::pos2(x, y) + canvas_rect.min.to_vec2())
                    .collect(),
                egui::Stroke::new(self.current_stroke.thickness, egui::Color32::BLACK),
            ));
        }
    }
}
```

## Conclusion

This update establishes a clear and consistent conversion process between screen and document coordinate spaces. By converting coordinates when recording user input and reversing the conversion for rendering, we ensure that stroke previews accurately reflect the actual stored data. This enhancement is crucial for preventing offset errors and lays the groundwork for further improvements in our input handling and rendering pipeline.
