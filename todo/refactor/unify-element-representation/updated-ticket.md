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
- Command system works exclusively with the Element trait methods
- Tools create concrete types but transfer ownership to EditorModel

### Performance Optimization

- Lazy texture generation on first draw
- Texture invalidation on element modification
- Clear separation between model and rendering state

## Implementation Tasks

1. **Core Structure Changes** (2 days)

   - Create element module structure
   - Implement Element trait and ElementType enum
   - Remove Arc wrappers
   - Create concrete element implementations
   - Implement factory module for element creation
   - Write unit tests for element creation and basic operations

2. **Storage Refactoring** (1 day)

   - Update EditorModel to store Vec<ElementType>
   - Implement ownership transfer methods (take/add)
   - Replace type-specific methods with unified methods
   - Update find/filter functions to use Element trait
   - Integrate factory functions for element creation

3. **Rendering System** (2 days)

   - Implement texture-based rendering
   - Add regenerate_texture method to Element trait
   - Update Renderer to work with Element trait
   - Ensure texture invalidation on element modifications

4. **Command System Update** (2 days)

   - Replace variant-specific commands with element commands
   - Convert to ownership transfer pattern
   - Update command execution to use Element trait methods
   - Update undo/redo to work with unified elements
   - Use factory functions for element recreation in commands

5. **Text Element Support** (2 days)

   - Implement Text struct
   - Add text rendering via TextureHandle
   - Create TextTool for adding/editing text
   - Add factory functions for text elements

6. **Testing & Cleanup** (1 day)
   - Update tests for unified elements
   - Remove obsolete code
   - Document new architecture

## Acceptance Criteria

- All code outside of element/ module only accesses elements via the Element trait
- Element creation occurs exclusively through factory functions
- Element-specific details (stroke points, image data, text content) are encapsulated
- Command system works uniformly across all element types using ownership transfer
- Cloning is minimized and only used where necessary
- Text elements can be created, edited, and rendered
- No performance degradation compared to current implementation

## Migration Strategy

1. First implement element module structure with trait and factories
2. Replace ElementTypeMut with direct Element trait methods
3. Update EditorModel to use Vec<ElementType> with ownership transfer
4. Update commands to use Element trait and ownership transfer pattern
5. Update tools to use factory functions for element creation
6. Add text support last (after core refactoring is complete)
