use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::stroke::Stroke;
use egui::{Color32, ColorImage, TextureHandle, TextureOptions};

/// Represents a single layer in the document
#[derive(Clone, Serialize, Deserialize)]
pub enum LayerContent {
    Strokes(Vec<Stroke>),
    Image {
        #[serde(skip)]
        texture: Option<TextureHandle>,
        size: [usize; 2],
    }
}

impl LayerContent {
    pub fn strokes(&self) -> Option<&Vec<Stroke>> {
        match self {
            LayerContent::Strokes(strokes) => Some(strokes),
            LayerContent::Image { .. } => None,
        }
    }
}

impl std::fmt::Debug for LayerContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerContent::Strokes(strokes) => f.debug_tuple("Strokes").field(strokes).finish(),
            LayerContent::Image { size, .. } => f
                .debug_struct("Image")
                .field("size", size)
                .finish(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Unique identifier for the layer
    pub id: Uuid,
    /// Display name of the layer
    pub name: String,
    /// Whether the layer is currently visible
    pub visible: bool,
    /// Content of the layer
    pub content: LayerContent,
    /// GPU texture for the layer
    #[serde(skip)]
    pub gpu_texture: Option<TextureHandle>,
    /// Size of the layer
    #[serde(skip)]
    pub size: [usize; 2],
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            visible: true,
            content: LayerContent::Strokes(Vec::new()),
            gpu_texture: None,
            size: [800, 600], // Default size
        }
    }

    pub fn new_image(name: &str, texture: TextureHandle, size: [usize; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            visible: true,
            content: LayerContent::Image {
                texture: Some(texture),
                size,
            },
            gpu_texture: None,
            size,
        }
    }

    /// Adds a stroke to the layer
    pub fn add_stroke(&mut self, stroke: Stroke) {
        if let LayerContent::Strokes(strokes) = &mut self.content {
            strokes.push(stroke);
        }
    }

    /// Removes and returns the last stroke from the layer
    pub fn remove_last_stroke(&mut self) -> Option<Stroke> {
        match &mut self.content {
            LayerContent::Strokes(strokes) => strokes.pop(),
            LayerContent::Image { .. } => None,
        }
    }

    /// Updates the GPU texture for the layer
    pub fn update_gpu_texture(&mut self, ctx: &egui::Context) {
        match &self.content {
            LayerContent::Strokes(strokes) => {
                // Create a new ColorImage for the strokes
                let mut image = ColorImage::new(self.size, Color32::TRANSPARENT);
                
                // Rasterize each stroke
                for stroke in strokes {
                    // For each stroke, we'll draw lines between consecutive points
                    for points in stroke.points.windows(2) {
                        if let [(x1, y1), (x2, y2)] = points {
                            draw_line(
                                &mut image,
                                (*x1, *y1),
                                (*x2, *y2),
                                stroke.thickness,
                                stroke.color,
                            );
                        }
                    }
                }

                // Create or update texture
                let texture_name = format!("layer_{}", self.id);
                self.gpu_texture = Some(ctx.load_texture(
                    &texture_name,
                    image,
                    TextureOptions::default()
                ));
            }
            LayerContent::Image { texture, size } => {
                // For image layers, we can reuse the existing texture
                self.gpu_texture = texture.clone();
                self.size = *size;
            }
        }
    }
}

impl std::fmt::Debug for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layer")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("visible", &self.visible)
            .field("content", &self.content)
            .field("size", &self.size)
            .finish()
    }
}

// Helper function to draw an anti-aliased line
pub fn draw_line(
    image: &mut ColorImage,
    (x1, y1): (f32, f32),
    (x2, y2): (f32, f32),
    thickness: f32,
    color: Color32,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let length = (dx * dx + dy * dy).sqrt();
    
    if length < 1.0 {
        return;
    }

    // Calculate the number of steps based on line length
    let steps = (length.ceil() as usize).max(1);
    
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = x1 + dx * t;
        let y = y1 + dy * t;
        
        // Draw a circle at each point for thick lines
        draw_circle(image, x, y, thickness / 2.0, color);
    }
}

// Helper function to draw an anti-aliased circle
fn draw_circle(
    image: &mut ColorImage,
    center_x: f32,
    center_y: f32,
    radius: f32,
    color: Color32,
) {
    let r = radius.ceil() as i32;
    let x_min = (center_x - radius).floor() as i32;
    let x_max = (center_x + radius).ceil() as i32;
    let y_min = (center_y - radius).floor() as i32;
    let y_max = (center_y + radius).ceil() as i32;
    
    let [width, height] = image.size;
    
    for y in y_min..=y_max {
        if y < 0 || y >= height as i32 {
            continue;
        }
        
        for x in x_min..=x_max {
            if x < 0 || x >= width as i32 {
                continue;
            }
            
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance <= radius {
                let alpha = if distance > radius - 1.0 {
                    (radius - distance).max(0.0)
                } else {
                    1.0
                };
                
                let pixel_idx = y as usize * width + x as usize;
                let existing = image.pixels[pixel_idx];
                let new_color = blend_colors(existing, color, alpha);
                image.pixels[pixel_idx] = new_color;
            }
        }
    }
}

// Helper function to blend colors with alpha
fn blend_colors(bg: Color32, fg: Color32, alpha: f32) -> Color32 {
    let a = (fg.a() as f32 * alpha) as u8;
    if a == 0 {
        return bg;
    }
    if a == 255 {
        return fg;
    }
    
    let bg_a = bg.a() as f32 / 255.0;
    let fg_a = a as f32 / 255.0;
    let out_a = fg_a + bg_a * (1.0 - fg_a);
    
    if out_a == 0.0 {
        return Color32::TRANSPARENT;
    }
    
    let mix = |bg: u8, fg: u8| -> u8 {
        let bg = bg as f32 * bg_a;
        let fg = fg as f32 * fg_a;
        ((fg + bg * (1.0 - fg_a)) / out_a) as u8
    };
    
    Color32::from_rgba_unmultiplied(
        mix(bg.r(), fg.r()),
        mix(bg.g(), fg.g()),
        mix(bg.b(), fg.b()),
        (out_a * 255.0) as u8,
    )
}
