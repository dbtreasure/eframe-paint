# Phase 1: Core Structure Changes

This document outlines the detailed implementation steps for the first phase of our element unification refactoring.

## Overview

Phase 1 focuses on establishing the foundational structure for our unified element system:

- Creating the element module structure
- Implementing the Element trait
- Removing Arc wrappers
- Creating concrete element implementations

## Directory Structure

Create the following directory structure:

```
src/
├── element/
│   ├── mod.rs       # Public Element interface
│   ├── common.rs    # Shared utilities and constants
│   ├── stroke.rs    # Stroke implementation
│   └── image.rs     # Image implementation
```

## Step-by-Step Implementation

### Step 1: Create Element Trait (element/mod.rs)

```rust
// src/element/mod.rs
use egui;

mod common;
mod stroke;
mod image;

// Re-export the concrete types for internal use only
pub(crate) use stroke::Stroke;
pub(crate) use image::Image;

// Constants moved from the old element.rs
pub use common::*;

/// Common trait for all element types in the document
pub trait Element {
    /// Get the unique identifier for this element
    fn id(&self) -> usize;

    /// Get the element type as a string
    fn element_type(&self) -> &'static str;

    /// Get the bounding rectangle for this element
    fn rect(&self) -> egui::Rect;

    /// Test if the given point hits this element
    fn hit_test(&self, pos: egui::Pos2) -> bool;

    /// Move the element by the specified delta
    fn translate(&mut self, delta: egui::Vec2) -> Result<(), String>;

    /// Resize the element to fit the new rectangle
    fn resize(&mut self, new_rect: egui::Rect) -> Result<(), String>;

    /// Get the texture handle for this element, if available
    fn texture(&self) -> Option<&egui::TextureHandle>;

    /// Generate or regenerate texture for this element
    fn regenerate_texture(&mut self, ctx: &egui::Context) -> bool;
}

/// Storage type for all elements
#[derive(Clone)]
pub enum ElementType {
    Stroke(Stroke),
    Image(Image),
}

impl ElementType {
    pub fn get_id(&self) -> usize {
        match self {
            ElementType::Stroke(stroke) => stroke.id(),
            ElementType::Image(image) => image.id(),
        }
    }

    pub fn element_type_str(&self) -> &'static str {
        match self {
            ElementType::Stroke(_) => "stroke",
            ElementType::Image(_) => "image",
        }
    }
}

// Implement Element trait for ElementType (delegation pattern)
impl Element for ElementType {
    fn id(&self) -> usize {
        self.get_id()
    }

    fn element_type(&self) -> &'static str {
        self.element_type_str()
    }

    fn rect(&self) -> egui::Rect {
        match self {
            ElementType::Stroke(s) => s.rect(),
            ElementType::Image(i) => i.rect(),
        }
    }

    fn hit_test(&self, pos: egui::Pos2) -> bool {
        match self {
            ElementType::Stroke(s) => s.hit_test(pos),
            ElementType::Image(i) => i.hit_test(pos),
        }
    }

    fn translate(&mut self, delta: egui::Vec2) -> Result<(), String> {
        match self {
            ElementType::Stroke(s) => s.translate(delta),
            ElementType::Image(i) => i.translate(delta),
        }
    }

    fn resize(&mut self, new_rect: egui::Rect) -> Result<(), String> {
        match self {
            ElementType::Stroke(s) => s.resize(new_rect),
            ElementType::Image(i) => i.resize(new_rect),
        }
    }

    fn texture(&self) -> Option<&egui::TextureHandle> {
        match self {
            ElementType::Stroke(s) => s.texture(),
            ElementType::Image(i) => i.texture(),
        }
    }

    fn regenerate_texture(&mut self, ctx: &egui::Context) -> bool {
        match self {
            ElementType::Stroke(s) => s.regenerate_texture(ctx),
            ElementType::Image(i) => i.regenerate_texture(ctx),
        }
    }
}

// Utility functions
pub fn compute_element_rect(element: &dyn Element) -> egui::Rect {
    let base_rect = element.rect();
    let padding = match element.element_type() {
        "stroke" => STROKE_BASE_PADDING,
        "image" => IMAGE_PADDING,
        _ => STROKE_BASE_PADDING, // Default
    };

    egui::Rect::from_min_max(
        egui::pos2(base_rect.min.x - padding, base_rect.min.y - padding),
        egui::pos2(base_rect.max.x + padding, base_rect.max.y + padding),
    )
}
```

### Step 2: Create Common Constants (element/common.rs)

```rust
// src/element/common.rs
pub const RESIZE_HANDLE_RADIUS: f32 = 15.0;
pub const STROKE_BASE_PADDING: f32 = 10.0;
pub const IMAGE_PADDING: f32 = 10.0;
```

### Step 3: Stroke Implementation (element/stroke.rs)

```rust
// src/element/stroke.rs
use egui;
use crate::element::Element;
use log::info;

/// Represents a stroke element (drawn line)
#[derive(Clone, Debug)]
pub struct Stroke {
    id: usize,
    points: Vec<egui::Pos2>,
    thickness: f32,
    color: egui::Color32,
    texture_handle: Option<egui::TextureHandle>,
}

impl Stroke {
    pub fn new(id: usize, points: Vec<egui::Pos2>, thickness: f32, color: egui::Color32) -> Self {
        Self {
            id,
            points,
            thickness,
            color,
            texture_handle: None,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn points(&self) -> &[egui::Pos2] {
        &self.points
    }

    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    pub fn color(&self) -> egui::Color32 {
        self.color
    }

    /// Rasterize the stroke to a texture
    fn rasterize(&self, ctx: &egui::Context) -> egui::TextureHandle {
        // Simplified implementation - will need expansion in actual code
        // Calculate bounds
        let rect = self.calculate_bounds();
        let size = rect.size();

        // Minimum size check
        let size = egui::Vec2::new(
            size.x.max(1.0),
            size.y.max(1.0),
        );

        // Create a temporary painter to render stroke to texture
        let color_image = egui::ColorImage::new([size.x as usize, size.y as usize], egui::Color32::TRANSPARENT);

        // This is simplified and will need actual rasterization logic
        // In practice, we would:
        // 1. Create an offscreen framebuffer
        // 2. Render the stroke to it
        // 3. Capture as a texture

        let texture_id = format!("stroke_{}", self.id);
        ctx.load_texture(texture_id, color_image, egui::TextureOptions::LINEAR)
    }

    /// Calculate the bounding rectangle for this stroke
    fn calculate_bounds(&self) -> egui::Rect {
        if self.points.is_empty() {
            return egui::Rect::NOTHING;
        }

        // Calculate bounds from points
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for point in &self.points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        // Expand by stroke thickness/2
        let padding = self.thickness / 2.0;

        egui::Rect::from_min_max(
            egui::pos2(min_x - padding, min_y - padding),
            egui::pos2(max_x + padding, max_y + padding),
        )
    }
}

impl Element for Stroke {
    fn id(&self) -> usize {
        self.id
    }

    fn element_type(&self) -> &'static str {
        "stroke"
    }

    fn rect(&self) -> egui::Rect {
        self.calculate_bounds()
    }

    fn hit_test(&self, pos: egui::Pos2) -> bool {
        // Simple rectangle hit test - can be enhanced with point-to-line test
        self.rect().contains(pos)
    }

    fn translate(&mut self, delta: egui::Vec2) -> Result<(), String> {
        // Apply translation to all points
        for point in &mut self.points {
            *point += delta;
        }

        // Invalidate texture since element has changed
        self.texture_handle = None;

        Ok(())
    }

    fn resize(&mut self, new_rect: egui::Rect) -> Result<(), String> {
        let old_rect = self.rect();

        // Skip if rectangles are identical
        if (old_rect.min - new_rect.min).abs() < 0.001 &&
           (old_rect.max - new_rect.max).abs() < 0.001 {
            return Ok(());
        }

        // Calculate scale factors
        let scale_x = new_rect.width() / old_rect.width();
        let scale_y = new_rect.height() / old_rect.height();

        // Apply scaling to all points
        for point in &mut self.points {
            // Transform relative to the original rect's min
            let relative_x = point.x - old_rect.min.x;
            let relative_y = point.y - old_rect.min.y;

            // Scale
            let scaled_x = relative_x * scale_x;
            let scaled_y = relative_y * scale_y;

            // Transform back to absolute coordinates with new rect's min
            point.x = new_rect.min.x + scaled_x;
            point.y = new_rect.min.y + scaled_y;
        }

        // Invalidate texture
        self.texture_handle = None;

        Ok(())
    }

    fn texture(&self) -> Option<&egui::TextureHandle> {
        self.texture_handle.as_ref()
    }

    fn regenerate_texture(&mut self, ctx: &egui::Context) -> bool {
        if self.texture_handle.is_none() {
            self.texture_handle = Some(self.rasterize(ctx));
            true
        } else {
            false
        }
    }
}
```

### Step 4: Image Implementation (element/image.rs)

```rust
// src/element/image.rs
use egui;
use crate::element::Element;

/// Represents an image element
#[derive(Clone, Debug)]
pub struct Image {
    id: usize,
    position: egui::Pos2,
    size: egui::Vec2,
    data: Vec<u8>,
    texture_handle: Option<egui::TextureHandle>,
}

impl Image {
    pub fn new(id: usize, data: Vec<u8>, size: egui::Vec2, position: egui::Pos2) -> Self {
        Self {
            id,
            position,
            size,
            data,
            texture_handle: None,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn position(&self) -> egui::Pos2 {
        self.position
    }

    pub fn size(&self) -> egui::Vec2 {
        self.size
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Create a texture from the image data
    fn create_texture(&self, ctx: &egui::Context) -> egui::TextureHandle {
        let width = self.size.x as usize;
        let height = self.size.y as usize;

        // Make sure data size matches expected dimensions
        if self.data.len() == width * height * 4 {
            // Create color image from RGBA data
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [width, height],
                &self.data
            );

            // Load as texture
            let texture_id = format!("image_{}", self.id);
            ctx.load_texture(texture_id, color_image, egui::TextureOptions::LINEAR)
        } else {
            // Create a placeholder for invalid data
            let size = 100;
            let mut placeholder = vec![0; size * size * 4];

            // Fill with magenta to indicate error
            for pixel in placeholder.chunks_mut(4) {
                pixel[0] = 255; // R
                pixel[1] = 0;   // G
                pixel[2] = 255; // B
                pixel[3] = 255; // A
            }

            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [size, size],
                &placeholder
            );

            let texture_id = format!("image_error_{}", self.id);
            ctx.load_texture(texture_id, color_image, egui::TextureOptions::LINEAR)
        }
    }
}

impl Element for Image {
    fn id(&self) -> usize {
        self.id
    }

    fn element_type(&self) -> &'static str {
        "image"
    }

    fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(self.position, self.size)
    }

    fn hit_test(&self, pos: egui::Pos2) -> bool {
        self.rect().contains(pos)
    }

    fn translate(&mut self, delta: egui::Vec2) -> Result<(), String> {
        self.position += delta;

        // No need to invalidate texture for position change since
        // the texture content doesn't change, only its position

        Ok(())
    }

    fn resize(&mut self, new_rect: egui::Rect) -> Result<(), String> {
        // Update position and size
        self.position = new_rect.min;
        self.size = new_rect.size();

        // For images, we need to regenerate the texture with the new size
        self.texture_handle = None;

        Ok(())
    }

    fn texture(&self) -> Option<&egui::TextureHandle> {
        self.texture_handle.as_ref()
    }

    fn regenerate_texture(&mut self, ctx: &egui::Context) -> bool {
        if self.texture_handle.is_none() {
            self.texture_handle = Some(self.create_texture(ctx));
            true
        } else {
            false
        }
    }
}
```

### Step 5: Migration from Old Element.rs

1. Create the new files and directories as outlined above
2. Comment out the existing `src/element.rs` file, but don't delete it yet
3. Update imports in other files to use the new module structure
4. Implement converter functions to migrate from old types to new types

```rust
// in src/element/mod.rs, add:

/// Convert from old StrokeRef to new Stroke
pub fn convert_stroke_ref(stroke_ref: &crate::stroke::StrokeRef) -> Stroke {
    Stroke::new(
        stroke_ref.id(),
        stroke_ref.points().to_vec(),
        stroke_ref.thickness(),
        stroke_ref.color(),
    )
}

/// Convert from old ImageRef to new Image
pub fn convert_image_ref(image_ref: &crate::image::ImageRef) -> Image {
    Image::new(
        image_ref.id(),
        image_ref.data().to_vec(),
        image_ref.size(),
        image_ref.position(),
    )
}
```

## Testing Strategy

For Phase 1, we'll focus on unit tests to ensure the new element implementations behave the same as the old ones:

1. Create test suite in `tests/element_tests.rs`
2. Test each method of the Element trait for each concrete type
3. Compare results between old and new implementations

Example test:

```rust
#[test]
fn test_stroke_rect_calculation() {
    // Create sample stroke with the old API
    let old_stroke = create_test_stroke_ref();
    let old_rect = old_stroke.rect();

    // Create equivalent stroke with the new API
    let new_stroke = element::convert_stroke_ref(&old_stroke);
    let new_rect = new_stroke.rect();

    // Compare results
    assert_eq!(old_rect.min, new_rect.min);
    assert_eq!(old_rect.max, new_rect.max);
}
```

## Validation Checklist

- [ ] All elements implement the Element trait
- [ ] Concrete implementations (Stroke, Image) maintain feature parity with existing code
- [ ] Arc wrappers are removed in favor of direct ownership
- [ ] Texture generation works for both element types
- [ ] Unit tests pass for all element types

## Next Steps

After completing Phase 1, we'll proceed to Phase 2 (Storage Refactoring) where we'll update the EditorModel to use the new unified element types.
