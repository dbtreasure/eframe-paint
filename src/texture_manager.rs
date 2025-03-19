use std::collections::HashMap;
use egui::{Context, TextureHandle, TextureId, ColorImage, TextureOptions};
use thiserror::Error;

/// Errors that can occur during texture generation
#[derive(Error, Debug)]
pub enum TextureGenerationError {
    #[error("Failed to generate texture")]
    GenerationFailed,
    #[error("Invalid texture dimensions")]
    InvalidDimensions,
}

/// Manages textures for elements, providing caching and invalidation
pub struct TextureManager {
    /// Cache of textures by (element_id, version)
    texture_cache: HashMap<(usize, u64), TextureHandle>,
    /// Tracks when each texture was last used
    last_used: HashMap<(usize, u64), u64>,
    /// Current frame counter for LRU tracking
    current_frame: u64,
    /// Maximum number of textures to cache
    max_cache_size: usize,
}

impl TextureManager {
    /// Creates a new texture manager with the specified cache size
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            texture_cache: HashMap::new(),
            last_used: HashMap::new(),
            current_frame: 0,
            max_cache_size,
        }
    }

    /// Increments the frame counter, should be called at the start of each frame
    pub fn begin_frame(&mut self) {
        self.current_frame += 1;
    }

    /// Gets or creates a texture for the given element
    pub fn get_or_create_texture<F>(
        &mut self,
        element_id: usize,
        texture_version: u64,
        generator: F,
        ctx: &Context,
    ) -> Result<TextureId, TextureGenerationError>
    where
        F: FnOnce() -> Result<ColorImage, TextureGenerationError>,
    {
        let cache_key = (element_id, texture_version);

        // Check if the texture is already in the cache
        if let Some(handle) = self.texture_cache.get(&cache_key) {
            // Update last used time
            self.last_used.insert(cache_key, self.current_frame);
            return Ok(handle.id());
        }

        // Prune cache if needed
        self.prune_cache_if_needed();

        // Generate a new texture
        let image = generator()?;
        
        // Create the texture
        let name = format!("element_{}_v{}", element_id, texture_version);
        let handle = ctx.load_texture(&name, image, TextureOptions::LINEAR);
        
        // Store in cache
        self.texture_cache.insert(cache_key, handle.clone());
        self.last_used.insert(cache_key, self.current_frame);
        
        Ok(handle.id())
    }

    /// Invalidates all textures for a specific element
    pub fn invalidate_element(&mut self, element_id: usize) {
        let keys_to_remove: Vec<(usize, u64)> = self.texture_cache
            .keys()
            .filter(|(id, _)| *id == element_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.texture_cache.remove(&key);
            self.last_used.remove(&key);
        }
    }

    /// Prunes the cache if it exceeds the maximum size
    fn prune_cache_if_needed(&mut self) {
        if self.texture_cache.len() <= self.max_cache_size {
            return;
        }

        // Collect keys and their last-used frames
        let mut entries: Vec<((usize, u64), u64)> = self.last_used
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect();

        // Sort by last-used frame (oldest first)
        entries.sort_by_key(|(_, frame)| *frame);

        // Remove oldest entries until we're under the limit
        let to_remove = entries.len() - self.max_cache_size;
        for ((id, version), _) in entries.iter().take(to_remove) {
            self.texture_cache.remove(&(*id, *version));
            self.last_used.remove(&(*id, *version));
        }
    }

    /// Clears all textures from the cache
    pub fn clear_cache(&mut self) {
        self.texture_cache.clear();
        self.last_used.clear();
    }

    /// Returns the number of textures currently in the cache
    pub fn cache_size(&self) -> usize {
        self.texture_cache.len()
    }

    #[cfg(test)]
    pub fn get_texture(&self, element_id: usize, version: u64) -> Option<&TextureHandle> {
        self.texture_cache.get(&(element_id, version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    // Mock a texture generation function for testing
    fn mock_texture_generator() -> Result<ColorImage, TextureGenerationError> {
        Ok(ColorImage::new([10, 10], egui::Color32::WHITE))
    }

    #[test]
    fn test_cache_hit() {
        let ctx = Context::default();
        let mut manager = TextureManager::new(10);
        
        // First call should create a new texture
        let texture_id1 = manager.get_or_create_texture(
            1, 1, mock_texture_generator, &ctx
        ).unwrap();
        
        // Second call with same params should hit the cache
        let texture_id2 = manager.get_or_create_texture(
            1, 1, mock_texture_generator, &ctx
        ).unwrap();
        
        // IDs should be the same
        assert_eq!(texture_id1, texture_id2);
        assert_eq!(manager.cache_size(), 1);
    }

    #[test]
    fn test_invalidation() {
        let ctx = Context::default();
        let mut manager = TextureManager::new(10);
        
        // Create texture
        manager.get_or_create_texture(
            1, 1, mock_texture_generator, &ctx
        ).unwrap();
        
        assert_eq!(manager.cache_size(), 1);
        
        // Invalidate all textures for element 1
        manager.invalidate_element(1);
        
        assert_eq!(manager.cache_size(), 0);
    }

    #[test]
    fn test_lru_eviction() {
        let ctx = Context::default();
        let mut manager = TextureManager::new(2);
        
        // Create three textures to trigger eviction
        manager.get_or_create_texture(1, 1, mock_texture_generator, &ctx).unwrap();
        manager.begin_frame();
        manager.get_or_create_texture(2, 1, mock_texture_generator, &ctx).unwrap();
        manager.begin_frame();
        manager.get_or_create_texture(3, 1, mock_texture_generator, &ctx).unwrap();
        
        // Cache should be at max size with most recent textures
        assert_eq!(manager.cache_size(), 2);
        assert!(manager.get_texture(1, 1).is_none()); // This one should be evicted
        assert!(manager.get_texture(2, 1).is_some());
        assert!(manager.get_texture(3, 1).is_some());
    }

    #[test]
    fn test_version_tracking() {
        let ctx = Context::default();
        let mut manager = TextureManager::new(10);
        
        // Create texture version 1
        manager.get_or_create_texture(1, 1, mock_texture_generator, &ctx).unwrap();
        
        // Create texture version 2
        manager.get_or_create_texture(1, 2, mock_texture_generator, &ctx).unwrap();
        
        // Both versions should be cached
        assert_eq!(manager.cache_size(), 2);
        assert!(manager.get_texture(1, 1).is_some());
        assert!(manager.get_texture(1, 2).is_some());
    }
}