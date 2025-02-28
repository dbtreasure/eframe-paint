use egui::{Pos2, Rect, Vec2};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// Static counter for generating unique IDs
static NEXT_IMAGE_ID: AtomicUsize = AtomicUsize::new(1);

// Immutable image for sharing
#[derive(Clone)]
pub struct Image {
    id: usize,             // Unique identifier for this image
    data: Vec<u8>,         // Raw image data
    size: Vec2,            // Width and height
    position: Pos2,        // Position in the document
}

// Mutable image for editing
pub struct MutableImage {
    id: usize,             // Unique identifier for this image
    data: Vec<u8>,
    size: Vec2,
    position: Pos2,
}

// Define a reference-counted type alias for Image
pub type ImageRef = Arc<Image>;

impl Image {
    // Create a new immutable image
    pub fn new(data: Vec<u8>, size: Vec2, position: Pos2) -> Self {
        let id = NEXT_IMAGE_ID.fetch_add(1, Ordering::SeqCst);
        Self {
            id,
            data,
            size,
            position,
        }
    }

    // Create a new reference-counted Image
    pub fn new_ref(data: Vec<u8>, size: Vec2, position: Pos2) -> ImageRef {
        Arc::new(Self::new(data, size, position))
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn position(&self) -> Pos2 {
        self.position
    }

    pub fn rect(&self) -> Rect {
        Rect::from_min_size(self.position, self.size)
    }
}

impl MutableImage {
    // Create a new mutable image for editing
    pub fn new(data: Vec<u8>, size: Vec2, position: Pos2) -> Self {
        let id = NEXT_IMAGE_ID.fetch_add(1, Ordering::SeqCst);
        Self {
            id,
            data,
            size,
            position,
        }
    }

    // Move the image
    pub fn set_position(&mut self, position: Pos2) {
        self.position = position;
    }

    // Resize the image (this doesn't actually resize the data, just changes the display size)
    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    // Convert to an immutable Image
    pub fn to_image(&self) -> Image {
        Image {
            id: self.id,
            data: self.data.clone(),
            size: self.size,
            position: self.position,
        }
    }

    // Convert to a reference-counted ImageRef
    pub fn to_image_ref(&self) -> ImageRef {
        Arc::new(self.to_image())
    }
}
