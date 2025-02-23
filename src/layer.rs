use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::stroke::Stroke;
use egui::{TextureHandle, Vec2};
use std::fmt;

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

impl PartialEq for LayerContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LayerContent::Strokes(a), LayerContent::Strokes(b)) => a == b,
            (LayerContent::Image { size: size_a, .. }, LayerContent::Image { size: size_b, .. }) => size_a == size_b,
            _ => false,
        }
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
                .field("texture", &"<texture>")
                .finish(),
        }
    }
}

/// Represents a transformation that can be applied to a layer
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    /// Position offset from the original position
    pub position: Vec2,
    /// Scale factor (1.0 = original size)
    pub scale: Vec2,
    /// Rotation in radians
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        const DEFAULT_TRANSFORM: Transform = Transform {
            position: Vec2::ZERO,
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0,
        };
        DEFAULT_TRANSFORM
    }
}

impl Transform {
    /// Creates a new identity transform
    pub fn identity() -> Self {
        Self::default()
    }

    /// Computes the transformation matrix for this transform with rotation around a specific pivot point
    pub fn to_matrix_with_pivot(&self, pivot: Vec2) -> [[f32; 3]; 3] {
        // In screen space, y points down, so we need to flip the sign of y-related
        // components in the rotation matrix to maintain proper rotation direction
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        // First translate to pivot point (origin)
        let mut result = [
            [1.0, 0.0, -pivot.x],
            [0.0, 1.0, -pivot.y],
            [0.0, 0.0, 1.0],
        ];

        // Then scale
        result = multiply_matrices(&[
            [self.scale.x, 0.0, 0.0],
            [0.0, self.scale.y, 0.0],
            [0.0, 0.0, 1.0],
        ], &result);

        // Then rotate (note the flipped sign for sin in the y components)
        result = multiply_matrices(&[
            [cos, sin, 0.0],
            [-sin, cos, 0.0],
            [0.0, 0.0, 1.0],
        ], &result);

        // Finally translate back from pivot and add position
        multiply_matrices(&[
            [1.0, 0.0, pivot.x + self.position.x],
            [0.0, 1.0, pivot.y + self.position.y],
            [0.0, 0.0, 1.0],
        ], &result)
    }

    /// Computes the transformation matrix for this transform
    pub fn to_matrix(&self) -> [[f32; 3]; 3] {
        // For backward compatibility, use (0,0) as pivot
        self.to_matrix_with_pivot(Vec2::ZERO)
    }
}

const fn const_multiply_matrices(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut result = [[0.0; 3]; 3];
    let mut i = 0;
    while i < 3 {
        let mut j = 0;
        while j < 3 {
            result[i][j] = 0.0;
            let mut k = 0;
            while k < 3 {
                result[i][j] += a[i][k] * b[k][j];
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
    result
}

fn multiply_matrices(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    const_multiply_matrices(a, b)
}

/// A unique identifier for a layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub usize);

impl LayerId {
    /// Creates a new LayerId from an index
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    /// Gets the underlying index
    pub fn index(&self) -> usize {
        self.0
    }
}

impl fmt::Display for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    /// Unique identifier for the layer
    pub id: Uuid,
    /// Display name of the layer
    pub name: String,
    /// Whether the layer is currently visible
    pub visible: bool,
    /// Content of the layer
    pub content: LayerContent,
    /// Transform applied to the layer
    pub transform: Transform,
    #[serde(skip)]
    pub needs_thumbnail_update: bool,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            visible: true,
            content: LayerContent::Strokes(Vec::new()),
            transform: Transform::default(),
            needs_thumbnail_update: true,
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
            transform: Transform::default(),
            needs_thumbnail_update: true,
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

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}