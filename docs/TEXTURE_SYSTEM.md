# Texture System Design

## Overview

The texture system provides efficient rendering of elements through cached textures, improving performance and visual quality. The system consists of the following components:

- **TextureManager**: Manages element textures with LRU caching
- **Element Trait**: Enhanced with texture generation methods
- **Renderer Integration**: Uses TextureManager for drawing elements

## Key Components

### TextureManager

The TextureManager handles creation, caching, and invalidation of textures:

- **Caching by (element_id, version)**: Textures are cached by element ID and version
- **LRU Eviction**: Least recently used textures are removed when cache is full
- **Invalidation**: Textures can be invalidated per-element or cleared entirely
- **Lazy Generation**: Textures are only generated when needed for rendering

```rust
pub struct TextureManager {
    texture_cache: HashMap<(usize, u64), TextureHandle>,
    last_used: HashMap<(usize, u64), u64>,
    current_frame: u64,
    max_cache_size: usize,
}
```

### Element Trait

The Element trait includes methods for texture handling:

- **texture_version()**: Returns the current texture version
- **needs_texture_update()**: Checks if texture regeneration is needed
- **invalidate_texture()**: Marks the texture as needing regeneration
- **generate_texture()**: Creates a ColorImage for rendering

```rust
trait Element {
    // ...existing methods...
    
    fn needs_texture_update(&self) -> bool;
    fn texture_version(&self) -> u64;
    fn invalidate_texture(&mut self);
    fn generate_texture(&mut self, ctx: &Context) -> Result<ColorImage, TextureGenerationError>;
}
```

### Renderer Integration

The Renderer uses TextureManager to draw elements:

- **draw_element()**: Unified method for drawing any element
- **invalidate_element_texture()**: Invalidates cached textures for an element
- **begin_frame()**: Updates texture frame counters

## Usage Patterns

### Element Modification

When an element is modified:

1. Element's texture is invalidated (`element.invalidate_texture()`)
2. Element's texture version is incremented
3. On next render, TextureManager detects the version change
4. A new texture is generated and cached

### Texture Generation

Texture generation follows these steps:

1. Renderer calls `draw_element(element)`
2. TextureManager checks for cached texture with matching version
3. If not found, calls `element.generate_texture(ctx)`
4. The resulting ColorImage is converted to a texture and cached
5. The texture is drawn to the screen

### Cache Management

TextureManager automatically manages the texture cache:

- Textures are keyed by (element_id, texture_version)
- Each frame updates the "last used" timestamp for accessed textures
- When cache exceeds max_cache_size, oldest textures are removed
- Entire cache can be cleared with `clear_cache()`

## Performance Considerations

- **Render-to-Texture**: Elements are rendered once to texture, then reused
- **Adaptive Resolution**: Texture dimensions can be adjusted based on zoom
- **Memory Management**: Unused textures are evicted from cache
- **Immediate Mode Fallback**: Elements can render directly if texture creation fails

## Implementation Notes

- Element implementations store texture_version and texture_needs_update flags
- TextureManager follows LRU cache pattern for eviction
- Renderer prioritizes texture-based rendering but falls back to direct drawing
- Error handling for invalid textures is provided with descriptive messages

## Future Enhancements

- Adaptive texture resolution based on zoom level
- More sophisticated cache strategies (weighted by texture size)
- Multi-threaded texture generation for complex elements
- Shared texture atlases for small elements