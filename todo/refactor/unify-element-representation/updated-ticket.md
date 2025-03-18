# Updated Ticket: Unify Element Representation for Strokes, Images, and Text

## Context

The application currently handles strokes and images as separate entities with duplicated logic and separate storage. This leads to code duplication and makes it difficult to add new element types like text. We need to unify our element representation to improve code maintainability and enable future enhancements.

## Goals

1. **Complete Encapsulation**: All element-specific implementation details should be hidden from the rest of the application.
2. **Texture-Based Rendering**: Adopt a unified texture-based rendering approach for all element types.
3. **Direct Ownership**: Remove Arc wrappers and use direct ownership in the EditorModel with explicit ownership transfer patterns.
4. **Common Interface**: Provide a consistent API for all element operations.
5. **Text Support**: Include text as a first-class element type in the unified system.

## Architecture

### Core Structure

```
src/
├── element/
│   ├── mod.rs       # Public Element interface and factory
│   ├── common.rs    # Shared utilities and constants
│   ├── stroke.rs    # Stroke implementation
│   ├── image.rs     # Image implementation
│   └── text.rs      # Text implementation
```

### Implementation Approach

1. **Element Trait**

   ```rust
   // Public interface - ONLY this is exposed to rest of app
   pub trait Element {
       fn id(&self) -> usize;
       fn element_type(&self) -> &'static str;
       fn rect(&self) -> egui::Rect;
       fn draw(&self, painter: &egui::Painter);
       fn hit_test(&self, pos: egui::Pos2) -> bool;
       fn translate(&mut self, delta: egui::Vec2) -> Result<(), String>;
       fn resize(&mut self, new_rect: egui::Rect) -> Result<(), String>;
       fn texture(&self) -> Option<&egui::TextureHandle>;
       fn regenerate_texture(&mut self, ctx: &egui::Context) -> bool;
       // Track if element needs texture regeneration
       fn needs_texture_update(&self) -> bool;
       // Get current texture version for cache invalidation
       fn texture_version(&self) -> u64;
       // Invalidate texture when element is modified
       fn invalidate_texture(&mut self);
   }
   ```

2. **Unified Storage Type**

   ```rust
   // Storage type - implements Clone for necessary cases
   #[derive(Clone)]
   pub enum ElementType {
       Stroke(Stroke),
       Image(Image),
       Text(Text),
   }

   // Implementation delegates to concrete types
   impl Element for ElementType {
       // Dispatch to concrete implementations
   }
   ```

3. **Factory Pattern for Element Creation**

   ```rust
   // In element/mod.rs - public API
   pub mod factory {
       use super::*;

       // Factory functions that create and return owned elements
       pub fn create_stroke(
           id: usize,
           points: Vec<egui::Pos2>,
           thickness: f32,
           color: egui::Color32
       ) -> ElementType {
           ElementType::Stroke(stroke::Stroke::new(id, points, thickness, color))
       }

       pub fn create_image(
           id: usize,
           data: Vec<u8>,
           size: egui::Vec2,
           position: egui::Pos2
       ) -> ElementType {
           ElementType::Image(image::Image::new(id, data, size, position))
       }

       pub fn create_text(
           id: usize,
           content: String,
           font: egui::FontId,
           position: egui::Pos2
       ) -> ElementType {
           ElementType::Text(text::Text::new(id, content, font, position))
       }
   }
   ```

4. **EditorModel Storage**

   ```rust
   pub struct EditorModel {
       elements: Vec<ElementType>, // Direct ownership
   }

   impl EditorModel {
       // Ownership transfer methods - preferred approach
       pub fn take_element_by_id(&mut self, id: usize) -> Option<ElementType> {
           let pos = self.elements.iter().position(|e| e.id() == id)?;
           Some(self.elements.swap_remove(pos))
       }

       pub fn add_element(&mut self, element: ElementType) {
           self.elements.push(element);
       }

       // Reference-based methods - use when ownership transfer isn't needed
       pub fn find_element_by_id(&self, id: usize) -> Option<&ElementType> {
           self.elements.iter().find(|e| e.id() == id)
       }

       pub fn get_element_mut(&mut self, id: usize) -> Option<&mut ElementType> {
           self.elements.iter_mut().find(|e| e.id() == id)
       }
   }
   ```

## Design Decisions

### Ownership Philosophy

- **Prefer Ownership Transfer**: When elements need to be modified, transfer ownership to the modifier, then back to EditorModel
- **Use References** when only reading or making temporary changes to data
- **Clone As Fallback**: Use cloning only when necessary (typically for undo/redo or when preserving original state)
- **No Arc Wrappers**: Replace all `Arc<T>` wrappers with direct ownership to simplify mental model

Example ownership pattern for commands:

```rust
// Command execution - ownership transfer approach
fn execute(&self, model: &mut EditorModel) {
    // Take ownership from model
    let mut element = model.take_element_by_id(self.element_id)
        .expect("Element not found");

    // Modify element
    element.translate(self.delta).unwrap();

    // Return ownership to model
    model.add_element(element);
}
```

### Element Creation Through Factories

- Use module-level factory functions to create elements
- Factory functions maintain encapsulation while transferring ownership
- Tools and commands use factories instead of direct constructors
- Implementation details stay hidden within element implementations

Example factory usage:

```rust
// In a tool implementation
fn create_element(&self, ctx: &egui::Context, editor: &mut EditorModel) {
    // Create an element with direct ownership transfer
    let element = element::factory::create_stroke(
        editor.next_id(),
        self.points.clone(),
        self.thickness,
        self.color
    );

    // Transfer ownership to editor model
    editor.add_element(element);
}
```

### Texture-Based Rendering

All elements will be rendered as textures:

- Images already work this way
- Strokes will be rasterized at their creation
- Text will use egui's text layout and font system

Benefits:

- Simplified rendering logic (uniform rendering path)
- Consistent element behavior
- Performance improvement by reducing draw calls

Tradeoffs:

- Vector strokes lose infinite resolution
- Transformations may require re-rasterization

### Element-Specific Knowledge Isolation

- Element-specific fields and methods are private to their module
- Only the Element trait and ElementType enum are exposed
- No code outside element/ should access type-specific data or methods
- All operations go through the unified Element interface
- Command system works exclusively with the Element trait
- Tools create concrete types but transfer ownership to EditorModel

### Performance Optimization

- Lazy texture generation on first draw
- Texture invalidation on element modification
- Clear separation between model and rendering state

### Consistent Texture Invalidation

- All element types use the same texture invalidation mechanism
- Changes to any element property automatically trigger invalidation
- Element implementations maintain internal version tracking
- Texture manager uses element id + version as cache key
- Modifications to elements always call invalidate_texture()

## Migration Strategy

### 1. Preparation Phase (1 week)

1. **Create Parallel Structure**
   - Create new element/ directory without modifying existing files
   - Implement core trait and enum structures
   - Set up factory module
   - Create stubs for all concrete implementations

### 2. Core Implementation (2 weeks)

1. **Element Trait Implementation** (3 days)

   - Files to modify:
     - Create src/element/mod.rs with the Element trait definition
     - Create src/element/common.rs with shared utilities
   - Implementation steps:
     - Define the Element trait with all required methods
     - Implement ElementType enum
     - Add factory module
   - Checkpoint: Verify trait compiles without errors

2. **TextureManager Implementation** (2 days)

   - Files to modify:
     - Create src/texture_manager.rs
   - Implementation steps:
     - Define TextureManager struct with LRU cache
     - Implement core texture caching methods
     - Add texture invalidation mechanisms
   - Checkpoint: Unit test texture caching and eviction

3. **Stroke Implementation** (2 days)

   - Files to modify:
     - Create src/element/stroke.rs
   - Implementation steps:
     - Implement Stroke struct
     - Add texture generation method
     - Integrate with TextureManager
   - Checkpoint: Verify stroke textures render correctly

4. **Image Implementation** (1 day)

   - Files to modify:
     - Create src/element/image.rs
   - Implementation steps:
     - Implement Image struct
     - Add texture generation method
     - Integrate with TextureManager
   - Checkpoint: Verify image textures render correctly

5. **Renderer Integration** (3 days)

   - Files to modify:
     - Update src/renderer.rs
   - Implementation steps:
     - Add fallback path for old implementation
     - Integrate TextureManager
     - Add unified draw_element method
     - Handle preview states
   - Checkpoint: Verify rendering works with both old and new paths

6. **EditorModel Integration** (2 days)

   - Files to modify:
     - Update src/state.rs
   - Implementation steps:
     - Replace ElementType/ElementTypeMut with new Element trait
     - Update element storage
     - Implement ownership transfer methods
   - Checkpoint: Verify model operations work correctly

7. **Preview State Handling** (1 day)
   - Files to modify:
     - Update src/renderer.rs
     - Update src/tools/draw_stroke_tool.rs
   - Implementation steps:
     - Add preview texture generation
     - Update drawing preview mechanism
   - Checkpoint: Verify drawing preview works smoothly

### 3. Validation Steps (1 week)

1. **Unit Testing** (2 days)

   - Create tests for each component:
     - Element trait implementation tests
     - TextureManager tests
     - Stroke/Image implementation tests
   - Test error handling and edge cases:
     - Empty strokes
     - Invalid images
     - Large textures

2. **Integration Testing** (2 days)

   - Test interaction between components:
     - Element ↔ TextureManager
     - EditorModel ↔ Element
     - Renderer ↔ TextureManager
   - Test command execution flow:
     - Add/remove elements
     - Modify elements
     - Undo/redo operations

3. **Performance Benchmarking** (2 days)

   - Run benchmarks defined in preparation phase:
     - Document load time
     - Frame render time
     - Memory usage
   - Compare old vs new implementation:
     - Generate performance reports
     - Identify bottlenecks
     - Optimize critical paths

4. **Visual Regression Testing** (1 day)
   - Run visual comparison tests:
     - Generate screenshots of identical documents with old/new implementations
     - Compare pixel-by-pixel for differences
     - Document and address any visual regressions

## Implementation Tasks

1. **Core Structure Changes** (3 days)

   - Create element module structure
   - Implement Element trait and ElementType enum
   - Remove Arc wrappers
   - Create concrete element implementations
   - Implement factory module for element creation
   - Write unit tests for element creation and basic operations

   **Subtasks:**

   - Create element/mod.rs with trait definition (0.5 day)
   - Create element/common.rs with shared utilities (0.5 day)
   - Create ElementType enum with variants (0.5 day)
   - Implement factory functions (0.5 day)
   - Create test fixtures and unit tests (1 day)

   **Dependencies:**

   - None

   **Testing Requirements:**

   - Unit tests for Element trait methods
   - Tests for type conversion
   - Factory function tests

2. **TextureManager Implementation** (2 days)

   - Create TextureManager struct
   - Implement texture caching with LRU eviction
   - Add texture version tracking
   - Implement adaptive resolution scaling
   - Add culling for off-screen elements

   **Subtasks:**

   - Create texture_manager.rs (0.5 day)
   - Implement core caching functionality (0.5 day)
   - Add LRU eviction strategy (0.5 day)
   - Implement adaptive resolution based on zoom (0.5 day)

   **Dependencies:**

   - Element trait implementation

   **Testing Requirements:**

   - Cache hit/miss tests
   - Eviction policy tests
   - Performance benchmarks

3. **Storage Refactoring** (2 days)

   - Update EditorModel to store Vec<ElementType>
   - Implement ownership transfer methods (take/add)
   - Replace type-specific methods with unified methods
   - Update find/filter functions to use Element trait
   - Integrate factory functions for element creation

   **Subtasks:**

   - Update EditorModel storage (0.5 day)
   - Implement ownership transfer methods (0.5 day)
   - Update element access methods (0.5 day)
   - Update selection handling (0.5 day)

   **Dependencies:**

   - Core Structure Changes

   **Testing Requirements:**

   - Tests for element addition/removal
   - Tests for element selection
   - Tests for element finding/filtering

4. **Rendering System** (3 days)

   - Implement texture-based rendering
   - Add regenerate_texture method to Element trait
   - Implement TextureManager with proper caching
   - Ensure all element modifications properly invalidate textures
   - Add automatic texture garbage collection

   **Subtasks:**

   - Update Renderer to use TextureManager (1 day)
   - Implement stroke rasterization (1 day)
   - Add texture invalidation in Element implementations (0.5 day)
   - Add preview state handling (0.5 day)

   **Dependencies:**

   - TextureManager Implementation
   - Element Trait Implementation

   **Testing Requirements:**

   - Texture generation benchmarks
   - Visual comparison tests
   - Memory usage monitoring

5. **Command System Update** (2 days)

   - Replace variant-specific commands with element commands
   - Convert to ownership transfer pattern
   - Update command execution to use Element trait methods
   - Update undo/redo to work with unified elements
   - Use factory functions for element recreation

   **Subtasks:**

   - Update Command enum (0.5 day)
   - Update command execution methods (0.5 day)
   - Update undo/redo functionality (0.5 day)
   - Add texture invalidation after command execution (0.5 day)

   **Dependencies:**

   - Storage Refactoring

   **Testing Requirements:**

   - Command execution tests
   - Undo/redo tests
   - Texture invalidation tests

6. **Text Element Support** (2 days)

   - Implement Text struct
   - Add text rendering via TextureHandle
   - Create TextTool for adding/editing text
   - Add factory functions for text elements

   **Subtasks:**

   - Create element/text.rs (0.5 day)
   - Implement Text struct with the Element trait (0.5 day)
   - Add text texture generation (0.5 day)
   - Create TextTool (0.5 day)

   **Dependencies:**

   - Core Structure Changes
   - Rendering System

   **Testing Requirements:**

   - Text rendering tests
   - Text editing tests
   - Text tool interaction tests

7. **Testing & Cleanup** (2 days)

   - Update tests for unified elements
   - Remove obsolete code
   - Document new architecture
   - Performance optimization

   **Subtasks:**

   - Update existing tests (0.5 day)
   - Create new test cases (0.5 day)
   - Remove deprecated code (0.5 day)
   - Document architecture (0.5 day)

   **Dependencies:**

   - All previous tasks

   **Testing Requirements:**

   - Full test suite
   - Performance benchmarks
   - Visual regression tests

## Verification Checkpoints

### 1. Element Trait Verification

**What to verify:**

- Element trait methods are implemented correctly for all element types
- Type-specific data is properly encapsulated
- Factory functions create elements with correct properties

**Verification method:**

- Unit tests for each element type
- Manual verification of encapsulation
- Code review for factory functions

### 2. Texture Generation Verification

**What to verify:**

- Strokes render properly as textures
- Images maintain visual quality
- Text renders with proper font rendering

**Verification method:**

- Side-by-side comparison with old implementation
- Pixel-by-pixel screenshot comparison
- Visual inspection at different zoom levels

**Performance metrics:**

- Texture generation time
- Memory usage
- Frame render time

### 3. Interaction Verification

**What to verify:**

- Selection works correctly with new element representation
- Resize handles appear and function properly
- Drag operations work correctly
- Drawing preview appears correctly

**Verification method:**

- Automated UI tests
- Manual interaction testing
- Side-by-side comparison with old implementation

### 4. Command System Verification

**What to verify:**

- Add/remove element commands work correctly
- Move/resize commands function properly
- Undo/redo maintains correct state
- Textures update after command execution

**Verification method:**

- Command execution tests
- Visual verification after command execution
- State inspection after undo/redo

### 5. Performance Verification

**What to verify:**

- Rendering performance with large documents
- Memory usage stays within acceptable limits
- Texture caching works efficiently

**Performance metrics:**

- Frames per second
- Memory usage over time
- Texture cache hit rate
- Texture generation time

**Verification method:**

- Performance benchmark suite
- Memory profiling
- Frame timing instrumentation

### 6. Visual Quality Verification

**What to verify:**

- Strokes maintain visual quality when rendered as textures
- No visual artifacts during transformation
- Text remains crisp at all zoom levels

**Verification method:**

- Visual comparison at different zoom levels
- Stroke quality comparison
- Text rendering quality assessment

## Error Handling

### TextureGenerationError

```rust
#[derive(Debug, thiserror::Error)]
pub enum TextureGenerationError {
    #[error("Element has no visual representation")]
    NoVisualRepresentation,

    #[error("Texture dimensions too large: {width}x{height}")]
    TextureTooLarge { width: usize, height: usize },

    #[error("Failed to allocate memory for texture")]
    AllocationFailure,

    #[error("Internal error: {0}")]
    InternalError(String),
}
```

### Texture Manager Implementation

```rust
pub struct TextureManager {
    // Map from (element_id, texture_version) to TextureHandle
    texture_cache: HashMap<(usize, u64), egui::TextureHandle>,
    // Track when textures were last used
    last_used: HashMap<(usize, u64), u64>,
    // Current frame number
    current_frame: u64,
    // Maximum cache size
    max_cache_size: usize,
}

impl TextureManager {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            texture_cache: HashMap::new(),
            last_used: HashMap::new(),
            current_frame: 0,
            max_cache_size,
        }
    }

    pub fn begin_frame(&mut self) {
        self.current_frame += 1;
    }

    pub fn get_or_create_texture(
        &mut self,
        element: &mut dyn Element,
        ctx: &egui::Context
    ) -> Result<egui::TextureId, TextureGenerationError> {
        let element_id = element.id();
        let texture_version = element.texture_version();
        let cache_key = (element_id, texture_version);

        // Check if we have a cached texture
        if let Some(handle) = self.texture_cache.get(&cache_key) {
            // Update last used time
            self.last_used.insert(cache_key, self.current_frame);
            return Ok(handle.id());
        }

        // No cached texture, generate a new one
        let texture_handle = element.generate_texture(ctx)?;

        // Store in cache
        self.texture_cache.insert(cache_key, texture_handle.clone());
        self.last_used.insert(cache_key, self.current_frame);

        // Update the element's texture handle
        element.set_texture_handle(Some(texture_handle.clone()));

        // Manage cache size
        self.prune_cache_if_needed();

        Ok(texture_handle.id())
    }

    pub fn invalidate_texture(&mut self, element_id: usize) {
        // Remove all textures for this element
        self.texture_cache.retain(|&(id, _), _| id != element_id);
        self.last_used.retain(|&(id, _), _| id != element_id);
    }

    fn prune_cache_if_needed(&mut self) {
        if self.texture_cache.len() <= self.max_cache_size {
            return;
        }

        // Find the least recently used textures
        let mut entries: Vec<_> = self.last_used.iter().collect();
        entries.sort_by_key(|&(_, &frame)| frame);

        // Remove oldest entries until we're under the limit
        let to_remove = self.texture_cache.len() - self.max_cache_size;
        for (key, _) in entries.iter().take(to_remove) {
            self.texture_cache.remove(key);
            self.last_used.remove(key);
        }
    }
}
```
