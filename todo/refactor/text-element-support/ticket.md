# Text Element Support Implementation

**Related Ticket:** Unify Element Representation for Strokes, Images, and Text  
**Estimated Time:** 2-3 days  
**Priority:** P1 (Important feature addition)

## Objective

Implement text as a first-class element type within our unified element representation system. This will allow users to add, edit, and manipulate text elements with the same interaction patterns as strokes and images, while leveraging the existing Element trait and texture-based rendering infrastructure.

## Context

The application currently supports strokes and images as elements through the unified Element trait. The core unified element architecture is now complete, with commands and tools updated to use the element factories and ownership transfer pattern. Adding text support is the next logical step to complete the unified element representation.

## Current Status

- Element trait and unified storage structure implemented
- Texture-based rendering system working for existing element types
- Command system refactored to use ownership transfer pattern
- Tools using factory pattern for element creation
- Text element type was factored into the original design but not implemented

## Implementation Tasks

### 1. Text Element Implementation (1 day)

**Files to Create/Modify:**

- `src/element/text.rs` (new)
- `src/element/mod.rs` (update)

**Implementation Steps:**

```rust
// In src/element/text.rs
pub struct Text {
    id: usize,
    content: String,
    font: egui::FontId,
    position: egui::Pos2,
    color: egui::Color32,
    background: Option<egui::Color32>,
    texture_handle: Option<egui::TextureHandle>,
    texture_version: u64,
    needs_texture_update: bool,
}

impl Text {
    pub fn new(
        id: usize,
        content: String,
        font: egui::FontId,
        position: egui::Pos2,
        color: egui::Color32
    ) -> Self {
        Self {
            id,
            content,
            font,
            position,
            color,
            background: None,
            texture_handle: None,
            texture_version: 0,
            needs_texture_update: true,
        }
    }

    // Text-specific methods
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, content: String) {
        if self.content != content {
            self.content = content;
            self.invalidate_texture();
        }
    }

    pub fn font(&self) -> &egui::FontId {
        &self.font
    }

    pub fn set_font(&mut self, font: egui::FontId) {
        self.font = font;
        self.invalidate_texture();
    }

    pub fn color(&self) -> egui::Color32 {
        self.color
    }

    pub fn set_color(&mut self, color: egui::Color32) {
        self.color = color;
        self.invalidate_texture();
    }

    // Element trait implementation helpers
    fn calculate_text_size(&self, ctx: &egui::Context) -> egui::Vec2 {
        let layout = egui::Layout::left_to_right(egui::Align::Center);
        let text = egui::RichText::new(&self.content).font(self.font).color(self.color);
        let galley = ctx.fonts(|f| f.layout_no_wrap(text.text, text.font, text.color));
        galley.size()
    }
}

// Element trait implementation
impl crate::element::Element for Text {
    fn id(&self) -> usize {
        self.id
    }

    fn element_type(&self) -> &'static str {
        "text"
    }

    fn rect(&self) -> egui::Rect {
        // Return a rect based on the text size and position
        // This would calculate actual text dimensions using the Context
        let text_size = self.texture_handle
            .as_ref()
            .map(|handle| handle.size_vec2())
            .unwrap_or_else(|| egui::vec2(0.0, 0.0));

        egui::Rect::from_min_size(self.position, text_size)
    }

    fn draw(&self, painter: &egui::Painter) {
        if let Some(texture) = self.texture_handle.as_ref() {
            // Draw the texture at the text position
            let rect = self.rect();

            // Draw background if present
            if let Some(bg_color) = self.background {
                painter.rect_filled(rect, 0.0, bg_color);
            }

            // Draw text texture
            painter.image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
        }
    }

    fn hit_test(&self, pos: egui::Pos2) -> bool {
        self.rect().contains(pos)
    }

    fn translate(&mut self, delta: egui::Vec2) -> Result<(), String> {
        self.position += delta;
        Ok(())
    }

    fn resize(&mut self, new_rect: egui::Rect) -> Result<(), String> {
        // Text doesn't support direct resizing yet, but we could scale the font
        // For now, just move the text to the new position
        self.position = new_rect.min;
        Ok(())
    }

    fn texture(&self) -> Option<&egui::TextureHandle> {
        self.texture_handle.as_ref()
    }

    fn needs_texture_update(&self) -> bool {
        self.needs_texture_update
    }

    fn texture_version(&self) -> u64 {
        self.texture_version
    }

    fn invalidate_texture(&mut self) {
        self.needs_texture_update = true;
        self.texture_version += 1;
    }

    fn generate_texture(&mut self, ctx: &egui::Context) -> Result<egui::ColorImage, crate::texture_manager::TextureGenerationError> {
        // Create a temporary UI to layout and render the text
        let text = egui::RichText::new(&self.content)
            .font(self.font)
            .color(self.color);

        // Use egui's text rendering to create a pixel buffer
        let galley = ctx.fonts(|f| f.layout_no_wrap(text.text, text.font, text.color));

        // Calculate the size needed for the texture
        let size = galley.size();
        let width = size.x.ceil() as usize;
        let height = size.y.ceil() as usize;

        if width == 0 || height == 0 {
            return Err(crate::texture_manager::TextureGenerationError::NoVisualRepresentation);
        }

        // Create an empty image with proper sizing
        let mut image = egui::ColorImage::new([width, height], egui::Color32::TRANSPARENT);

        // Paint the text into the image
        let mut painter = egui::Painter::new(
            ctx.clone(),
            egui::TexturesMut::default(),
            egui::Rect::from_min_size(egui::Pos2::ZERO, size)
        );

        painter.galley(egui::Pos2::ZERO, galley, text.color);

        // Convert the painted output to ColorImage
        // (Note: This is a simplification, actual implementation would depend on egui's painting APIs)

        // Mark as updated
        self.needs_texture_update = false;

        Ok(image)
    }
}

// In src/element/mod.rs - Update ElementType enum
pub enum ElementType {
    Stroke(stroke::Stroke),
    Image(image::Image),
    Text(text::Text),
}

// Update Element trait implementation for ElementType
impl Element for ElementType {
    // Add text variant to all match statements...
}

// Add text factory
pub mod factory {
    // Existing factories...

    pub fn create_text(
        id: usize,
        content: String,
        font: egui::FontId,
        position: egui::Pos2,
        color: egui::Color32
    ) -> ElementType {
        ElementType::Text(text::Text::new(id, content, font, position, color))
    }
}
```

### 2. Text Tool Implementation (1 day)

**Files to Create/Modify:**

- `src/tools/text_tool.rs` (new)
- `src/tools/mod.rs` (update)

**Implementation Steps:**

```rust
// In src/tools/text_tool.rs
use crate::element::factory;
use crate::command::Command;
use crate::renderer::Renderer;
use crate::state::EditorModel;
use egui::{Color32, FontId, Key, Pos2, Ui};
use log::info;

#[derive(Clone, Debug)]
pub enum TextToolState {
    Ready,
    Placing { position: Pos2 },
    Editing {
        element_id: usize,
        content: String,
        position: Pos2,
    },
}

#[derive(Clone)]
pub struct TextTool {
    state: TextToolState,
    default_font: FontId,
    default_color: Color32,
    current_preview: Option<String>,
}

impl TextTool {
    pub fn new() -> Self {
        Self {
            state: TextToolState::Ready,
            default_font: FontId::proportional(20.0),
            default_color: Color32::BLACK,
            current_preview: None,
        }
    }

    // Implementation of text editing logic
    pub fn handle_text_input(&mut self, text: &str) {
        if let TextToolState::Editing { ref mut content, .. } = &mut self.state {
            *content += text;
        }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        if let TextToolState::Editing { ref mut content, .. } = &mut self.state {
            match key {
                Key::Backspace => {
                    if !content.is_empty() {
                        content.pop();
                        return true;
                    }
                }
                Key::Enter => {
                    // Finish editing on Enter
                    return true;
                }
                Key::Escape => {
                    // Cancel editing on Escape
                    self.state = TextToolState::Ready;
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn finish_editing(&mut self) -> Option<Command> {
        if let TextToolState::Editing { element_id, content, position } = &self.state {
            if !content.is_empty() {
                // Create a text element using the factory
                let element = factory::create_text(
                    *element_id,
                    content.clone(),
                    self.default_font.clone(),
                    *position,
                    self.default_color
                );

                // Reset state
                self.state = TextToolState::Ready;

                // Create add element command
                return Some(Command::AddElement { element });
            }
        }

        // Reset state without creating a command
        self.state = TextToolState::Ready;
        None
    }
}

// Implement Tool trait for TextTool
impl crate::tools::Tool for TextTool {
    fn name(&self) -> &'static str {
        "Text"
    }

    // Implement required Tool trait methods...
}

// In src/tools/mod.rs - Update ToolType enum
pub enum ToolType {
    DrawStroke(UnifiedDrawStrokeTool),
    Selection(UnifiedSelectionTool),
    Text(TextTool),
}

// Update Tool trait implementation for ToolType
impl Tool for ToolType {
    // Add text variant to all match statements...
}
```

### 3. UI Integration (0.5 day)

**Files to Modify:**

- `src/app.rs`
- `src/ui.rs` (or similar UI handling files)

**Implementation Steps:**

- Add text tool button to the toolbar
- Implement text input handling for the text tool
- Add text properties panel (font, size, color)
- Handle keyboard events for text editing

### 4. Text Editing Commands (0.5 day)

**Files to Modify:**

- `src/command.rs`

**Implementation Steps:**

- Add text-specific edit commands if needed
- Implement undo/redo for text editing operations
- Ensure texture invalidation works correctly for text

### 5. Testing and Documentation (0.5 day)

**Files to Create/Modify:**

- `tests/text_element_tests.rs` (new)
- Documentation files

**Implementation Steps:**

- Create unit tests for Text implementation
- Test text rendering and editing
- Update documentation with text element usage
- Add examples for text element creation

## Acceptance Criteria

1. **Text Creation:**

   - Users can add text elements to the document
   - Text appears at the selected position with default styling
   - Text elements are rendered correctly

2. **Text Editing:**

   - Users can edit text content
   - Font, size, and color can be customized
   - Changes are immediately visible

3. **Element Operations:**

   - Text elements can be selected
   - Text elements can be moved
   - Text elements persist in saved documents

4. **Unified Architecture:**

   - Text elements fully implement the Element trait
   - Text uses the texture-based rendering system
   - Commands work consistently with text elements
   - Ownership transfer pattern is maintained

5. **Error Handling:**
   - Empty text elements are handled gracefully
   - Very large text has appropriate performance optimizations
   - Input validation prevents invalid text states

## Dependencies

- Element Trait Implementation (completed)
- Texture Manager Implementation (completed)
- Command System Update (completed)

## Verification Checkpoints

1. **Text Rendering Test:**

   - Text renders correctly with various fonts and sizes
   - Text appears at the correct position
   - Text maintains quality at different zoom levels

2. **Text Interaction Test:**

   - Text editing behaves as expected
   - Keyboard shortcuts work properly
   - Selection and movement operate smoothly

3. **Architecture Verification:**
   - Text elements use the same patterns as other elements
   - No special cases or bypassing of Element trait
   - Factory pattern used consistently

## Implementation Notes

1. **Font Handling:**

   - Use egui's built-in font system
   - Consider caching frequent font/size combinations
   - Test with various Unicode character ranges

2. **Text Rendering:**

   - Render text to texture using egui's text layout system
   - Cache rendered text until content/style changes
   - Update texture version when text changes

3. **Editing Experience:**
   - Focus on making text editing feel natural
   - Support common keyboard shortcuts
   - Consider implementing a blinking cursor

## Future Enhancements

- Rich text formatting (bold, italic, etc.)
- Text alignment options
- Text wrapping within bounds
- Text on a path
