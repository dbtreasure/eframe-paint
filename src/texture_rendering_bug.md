# Texture Rendering Bug Report

## Bug Description

The application suffers from persistent issues with element visibility and texture management:

1. After transforming elements (moving or resizing), the original element remains visible alongside the transformed element
2. Selection boxes don't update properly to follow transformed elements
3. Clicking on the transformed element eventually makes the original disappear, suggesting element ID or selection state issues

## Root Cause Analysis

After extensive investigation, the root cause appears to be a complex interplay between several systems:

1. **egui Texture Management**: 
   - egui handles textures in a specific way that doesn't align with our caching approach
   - The framework expects ephemeral textures or careful manual invalidation

2. **Element Reference Management**:
   - When elements are transformed, both the original and transformed versions can exist simultaneously in different states
   - Document and renderer states become desynchronized

3. **Z-Ordering and Visibility Issues**:
   - The rendering pipeline doesn't correctly handle element z-order, causing both versions to be visible

## Failed Approaches

### Attempt 1: Texture Cache Invalidation

```rust
pub fn invalidate_texture(&mut self, element_id: usize) {
    self.texture_cache.remove(&element_id);
    self.texture_id_counter += 1;
    self.texture_keys.insert(element_id, self.texture_id_counter);
}
```

**Why it failed**: While this approach properly invalidated specific textures, it didn't prevent the original element from being rendered. The issue was that the element was still present in the document or state, leading to multiple versions being rendered.

### Attempt 2: Document Rebuild and State Reset

```rust
// In execute_command
self.document.rebuild();
self.renderer.reset_texture_state();
```

**Why it failed**: Rebuilding the document didn't address the fundamental issue of multiple elements being rendered. The renderer continued to draw both versions because both were tracked in some form.

### Attempt 3: Ephemeral Textures

```rust
// Completely rewritten draw_image method
fn draw_image(&mut self, ctx: &egui::Context, painter: &egui::Painter, image: &Image) {
    // ...
    let unique_texture_name = format!("ephemeral_img_{}_{}", image_id, self.texture_id_counter);
    let texture = ctx.load_texture(unique_texture_name, color_image, egui::TextureOptions::default());
}
```

**Why it failed**: This approach ensured fresh textures each frame, but didn't fix the core issue that multiple versions of the same element were being drawn. Even with fresh textures, if the document or state contained multiple references to the same conceptual element, all would be rendered.

### Attempt 4: In-place Element Replacement

```rust
pub fn replace_stroke_by_id(&mut self, id: usize, new_stroke: StrokeRef) -> bool {
    // ...
    if let Some(index) = index_to_remove {
        self.strokes[index] = new_stroke;
        self.mark_modified();
        return true;
    }
    false
}
```

**Why it failed**: While this approach maintained proper z-ordering, it didn't fully resolve the issue. This suggests that the problem may extend beyond just element replacement and is more deeply rooted in how elements are tracked across the application.

## Current Status

The bug persists despite multiple approaches, suggesting a more fundamental architectural issue. The main challenge appears to be ensuring that:

1. Only one version of an element exists in the system at any time
2. All components (document, renderer, selection state) are synchronized
3. All old references are properly cleaned up after transformations

## Recommended Next Steps

1. **Complete Architectural Review**:
   - Trace the full lifecycle of elements from creation through transformation
   - Identify all places where element references are stored or tracked

2. **Minimal Reproduction**:
   - Create a simplified test case that isolates just the element transformation flow
   - Use this to pinpoint exactly where duplicates occur

3. **Transaction-Based Approach**:
   - Consider implementing a transaction-based system for element operations
   - Ensure that all state updates happen atomically

4. **Event-Based Synchronization**:
   - Implement an observer pattern where all components subscribe to element lifecycle events
   - Ensure all components are notified when elements are created, modified, or deleted

5. **egui Expert Consultation**:
   - Consult with egui experts about the proper approach for dynamic element rendering
   - Review recommended patterns for texture management in this framework

The persistent nature of this bug suggests it may require a more foundational refactoring of how elements are managed throughout the application rather than isolated fixes to the texture rendering system.