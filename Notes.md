# Rotation Transform Fix: A Deep Dive

## The Problem

The application had a critical issue with rotation transformations where:

1. The rotation gizmo worked correctly (visual feedback showed proper angle calculation)
2. The actual image/content only scaled instead of rotating
3. Strokes were transforming correctly while images weren't

## Initial Assumptions (All Incorrect)

1. Assumed the issue was primarily with `atan2` argument order and coordinate system
2. Believed the transform matrix calculation in `layer.rs` was correct without deeper verification
3. Thought negating the Y coordinate would resolve screen-space vs world-space mismatch
4. Failed to trace how the rotation value actually flows through the rendering pipeline

## Failed Attempts

### First Attempt: Coordinate System Fixes

```rust
// Original
let initial_angle = (initial_pos - center).y.atan2((initial_pos - center).x);

// "Fix"
let initial_vec = initial_pos - center;
let initial_angle = (-initial_vec.y).atan2(initial_vec.x);
```

- Simply negating Y was naive
- Didn't verify transform space
- Failed to understand egui's coordinate system fully

### Second Attempt: Matrix Order

```rust
// Changed from:
// translate → rotate → scale → translate back
// To:
// translate → scale → rotate → translate back
```

- Changed matrix multiplication order
- Still didn't address the core issue
- Focused on transform calculation when the problem was in rendering

### Third Attempt: Transform Application

```rust
// Problematic approach: Creating axis-aligned rectangle
let transformed_rect = egui::Rect::from_points(&transformed_corners);
painter.image(texture.id(), final_rect, uv_rect, Color32::WHITE);

// This lost rotation information by creating an axis-aligned bounding box
```

## The Real Problem

The core issue wasn't in the transformation calculation but in how we were rendering the transformed image:

1. We were using `painter.image()` which expects an axis-aligned rectangle
2. By creating a rectangle from transformed points, we were losing rotation information
3. The transformation matrix and angle calculations were correct, but the rendering approach couldn't represent rotated images

## The Solution

### 1. Use egui's Mesh System

```rust
// Create proper textured mesh
let vertices: Vec<egui::epaint::Vertex> = transformed_corners
    .iter()
    .zip(uvs.iter())
    .map(|(&pos, &uv)| egui::epaint::Vertex {
        pos: pos,
        uv: uv,
        color,
    })
    .collect();

// Create mesh with proper indices for two triangles
let indices = vec![0, 1, 2, 0, 2, 3];
```

### 2. Preserve Transform Information

- Transform vertices directly instead of creating a bounding rectangle
- Maintain UV coordinates for proper texture mapping
- Use egui's primitive system for rendering

### 3. Proper Texture Mapping

```rust
// Create UV coordinates for texture mapping
let uv_rect = egui::Rect::from_min_max(
    egui::pos2(0.0, 0.0),
    egui::pos2(1.0, 1.0)
);
```

## Why This Works

1. **Primitive-Based Rendering**:

   - egui uses a tessellation system that converts shapes into triangles
   - By creating our own mesh, we maintain full transform information
   - No information is lost in intermediate representations

2. **Direct Transform Application**:

   - Transforms are applied to vertices directly
   - No intermediate axis-aligned bounding boxes
   - Preserves all transformation information

3. **Proper Texture Mapping**:
   - UV coordinates remain unchanged
   - Texture is properly mapped to transformed triangles
   - Maintains visual quality during rotation

## Lessons Learned

1. **Understand Your Rendering Pipeline**:

   - Know how your UI framework handles rendering
   - Understand the limitations of high-level APIs
   - Be aware of information loss in intermediate steps

2. **Debug Visualization is Critical**:

   - Visual feedback helped identify where the problem wasn't
   - Showed that angle calculation was correct
   - Helped narrow down the issue to rendering

3. **Clean Code Principles**:

   - Separate transform calculation from application
   - Centralize transform application code
   - Create proper abstractions for transformed shapes

4. **Testing Considerations**:
   - Need visual tests for transformations
   - Test each step of the transform pipeline
   - Verify no information loss between steps

## Future Recommendations

1. **Code Organization**:

   ```rust
   impl Transform {
       fn apply_to_point(&self, point: Pos2, pivot: Vec2) -> Pos2;
       fn apply_to_mesh(&self, mesh: &mut Mesh, pivot: Vec2);
       fn create_transformed_mesh(&self, vertices: &[Pos2]) -> Mesh;
   }
   ```

2. **Debug Tools**:

   - Add transform visualization toggles
   - Show intermediate steps in transform pipeline
   - Visualize coordinate systems and pivot points

3. **Testing Strategy**:
   - Add visual regression tests
   - Test transform composition
   - Verify rendering output

## Conclusion

The rotation transform issue was ultimately a rendering problem, not a mathematical one. By understanding egui's rendering pipeline and using its primitive system directly, we maintained transform information throughout the pipeline and achieved correct rotation behavior.

This experience highlights the importance of:

1. Understanding your rendering pipeline
2. Not making assumptions about high-level APIs
3. Using proper debug visualization
4. Testing at the right level of abstraction

# The Scaling Gizmo Saga: A Tale of Fixed Points and Transformations

## The Problem

The scaling gizmo implementation had several issues:

1. Different corners behaved inconsistently
2. Top-left corner scaled in reverse (dragging out made things smaller)
3. Top-right and bottom-left corners didn't maintain aspect ratio with shift key
4. The scaling behavior felt unintuitive and unpredictable

## Failed Approaches

### First Attempt: Sign-Based Scaling

```rust
// Calculate scale delta based on distance from center
let scale_delta = Vec2::new(
    scale_factor - 1.0,
    scale_factor - 1.0
);

// Apply different signs for different corners
match handle {
    GizmoHandle::ScaleTopLeft => transform.scale * Vec2::new(1.0 - scale_delta.x, 1.0 - scale_delta.y),
    GizmoHandle::ScaleTopRight => transform.scale * Vec2::new(1.0 + scale_delta.x, 1.0 - scale_delta.y),
    // ...
}
```

Problems:

1. Center-based scaling made corner behavior counterintuitive
2. Sign flipping led to reversed scaling for some corners
3. Mixed addition/subtraction broke aspect ratio preservation

### Second Attempt: Unified Delta Application

```rust
// Determine scale direction for each axis
let (scale_x_sign, scale_y_sign) = match handle {
    GizmoHandle::ScaleTopLeft => (-1.0, -1.0),
    GizmoHandle::ScaleTopRight => (1.0, -1.0),
    // ...
};

// Apply the scale with direction
transform.scale = initial_scale * (Vec2::splat(1.0) + scale_delta * signs);
```

Problems:

1. Mathematically equivalent to first approach
2. Still used center-based scaling
3. Didn't address the fundamental issues

## The Solution: Fixed-Point Scaling

The key insight was to treat each scale operation relative to the opposite corner instead of the center:

```rust
// Get the fixed point (opposite corner)
let fixed_point = match handle {
    GizmoHandle::ScaleTopLeft => bounds.right_bottom(),
    GizmoHandle::ScaleTopRight => bounds.left_bottom(),
    // ...
};

// Calculate vectors from fixed point
let initial_vec = initial_pos - fixed_point;
let current_vec = current_pos - fixed_point;

// Calculate scale factors independently
let scale_x = current_vec.x / initial_vec.x;
let scale_y = current_vec.y / initial_vec.y;
```

### Why This Works

1. **Natural Reference Point**:

   - Using the opposite corner as fixed point matches user's mental model
   - Dragging away from fixed point naturally increases size
   - Dragging toward fixed point naturally decreases size

2. **Independent Axis Scaling**:

   - Each axis calculated separately from vector components
   - Signs emerge naturally from vector math
   - No need for manual sign flipping

3. **Clean Aspect Ratio Preservation**:
   - Take max scale factor when shift is held
   - Preserve signs for direction
   - Works consistently for all corners

## Lessons Learned

1. **Choose the Right Reference Point**:

   - Center isn't always the best scaling reference
   - Consider user's mental model of the operation
   - Fixed points can simplify complex transformations

2. **Vector Math > Manual Signs**:

   - Let vector math handle directions naturally
   - Avoid manual sign flipping when possible
   - Use geometric relationships to simplify logic

3. **Independent vs. Unified Scaling**:

   - Calculate axes independently for precision
   - Unify only when aspect ratio matters
   - Preserve directional information

4. **Testing Considerations**:
   - Test all corners with and without shift key
   - Verify behavior at extreme scales
   - Check interaction with rotation

## Future Improvements

1. **Enhanced Visual Feedback**:

   - Show scale factors while dragging
   - Visualize fixed point and scaling axes
   - Indicate when aspect ratio is locked

2. **Smart Constraints**:

   - Add snapping to common scale factors
   - Optional grid alignment
   - Minimum/maximum scale limits

3. **Performance Optimization**:
   - Cache vector calculations
   - Optimize frequent operations
   - Reduce allocations in hot paths

## Conclusion

The scaling gizmo journey demonstrates how choosing the right mathematical model and reference point can dramatically simplify complex UI interactions. By switching from center-based scaling with manual sign adjustment to fixed-point scaling with natural vector math, we created a more intuitive and maintainable solution.

# The Coordinate Space Conundrum: A Tale of Two Spaces

## The Recurring Problem

The application has faced recurring issues with coordinate space handling, particularly:

1. Stroke preview appearing offset from actual stroke position
2. Inconsistent handling of canvas offsets
3. Mixing of screen space and document space coordinates

## The Core Issue

The fundamental challenge stems from managing two distinct coordinate spaces:

1. **Screen Space**:

   - Coordinates relative to the canvas position on screen
   - Includes canvas offset (`canvas_rect.min`)
   - Used by egui for input and rendering

2. **Document Space**:
   - Coordinates relative to the document origin (0,0)
   - Independent of canvas position
   - Used for storing and transforming content

## The Solution Pattern

The solution involves maintaining strict boundaries between coordinate spaces:

```rust
// Converting from screen to document space (input)
let doc_pos = screen_pos - canvas_rect.min.to_vec2();

// Converting from document to screen space (rendering)
let screen_pos = doc_pos + canvas_rect.min.to_vec2();
```

### Key Principles

1. **Single Source of Truth**:

   - Store all content in document space
   - Convert to screen space only for rendering
   - Never store screen space coordinates

2. **Explicit Conversions**:

   - Clearly mark all coordinate space conversions
   - Use consistent naming conventions
   - Document coordinate space in comments

3. **Consistent Preview Handling**:
   - Preview must use same coordinate space logic as final content
   - Convert preview coordinates same way as committed content
   - Test preview alignment with final result

## Lessons Learned

1. **Coordinate Space Hygiene**:

   - Never mix coordinate spaces without explicit conversion
   - Document coordinate space assumptions
   - Add debug assertions for coordinate ranges

2. **Testing Strategy**:

   - Test at canvas boundaries
   - Verify preview matches committed content
   - Add visual regression tests

3. **Code Organization**:
   - Create helper methods for coordinate conversion
   - Use strong typing to prevent mixing
   - Centralize coordinate space logic

## Future Recommendations

1. **Type System Enforcement**:

   ```rust
   struct DocumentPoint(Vec2);
   struct ScreenPoint(Vec2);

   impl DocumentPoint {
       fn to_screen(self, canvas_offset: Vec2) -> ScreenPoint;
   }

   impl ScreenPoint {
       fn to_document(self, canvas_offset: Vec2) -> DocumentPoint;
   }
   ```

2. **Debug Tools**:

   - Add coordinate space visualization
   - Log coordinate conversions
   - Show coordinate grids

3. **Testing Infrastructure**:
   - Automated visual regression tests
   - Coordinate conversion unit tests
   - Canvas resize tests

## Conclusion

Coordinate space issues are a common source of bugs in canvas-based applications. By maintaining strict coordinate space boundaries, using explicit conversions, and implementing proper testing, we can prevent these issues from recurring.

The key is to:

1. Always be explicit about coordinate spaces
2. Convert at well-defined boundaries
3. Test both preview and final rendering
4. Use strong typing when possible
