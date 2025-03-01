use crate::stroke::StrokeRef;
use crate::image::ImageRef;

pub struct Document {
    strokes: Vec<StrokeRef>,
    images: Vec<ImageRef>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            strokes: Vec::new(),
            images: Vec::new(),
        }
    }

    pub fn add_stroke(&mut self, stroke: StrokeRef) {
        self.strokes.push(stroke);
    }

    pub fn strokes(&self) -> &[StrokeRef] {
        &self.strokes
    }

    pub fn remove_last_stroke(&mut self) -> Option<StrokeRef> {
        self.strokes.pop()
    }
    
    pub fn add_image(&mut self, image: ImageRef) {
        self.images.push(image);
    }
    
    pub fn images(&self) -> &[ImageRef] {
        &self.images
    }
    
    pub fn remove_last_image(&mut self) -> Option<ImageRef> {
        let image = self.images.pop();
        image
    }
} 