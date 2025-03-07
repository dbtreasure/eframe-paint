// STRICTLY KEEP THESE CONSTANTS
pub const RESIZE_HANDLE_RADIUS: f32 = 15.0;
pub const STROKE_BASE_PADDING: f32 = 10.0;
pub const IMAGE_PADDING: f32 = 10.0;

use std::collections::HashMap;
use std::sync::Arc;
use egui::Rect;
use crate::state::ElementType;
use crate::element::Element;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cache for hit testing results to improve performance
pub struct HitTestCache {
    last_version: u64,
    element_bounds: HashMap<u64, Rect>,
}

impl HitTestCache {
    pub fn new() -> Self {
        Self {
            last_version: 0,
            element_bounds: HashMap::new(),
        }
    }

    /// Generate a hash for an element to use as a cache key
    fn element_hash(element: &ElementType) -> u64 {
        let mut hasher = DefaultHasher::new();
        match element {
            ElementType::Stroke(stroke) => {
                // For strokes, we don't have an ID, so hash the Arc pointer value
                let ptr = Arc::as_ptr(stroke) as usize;
                ptr.hash(&mut hasher);
            },
            ElementType::Image(image) => {
                // For images, we can use the ID
                image.id().hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Update the cache if the state version has changed
    pub fn update(&mut self, state: &crate::state::EditorState) {
        if state.version() != self.last_version {
            self.element_bounds.clear();
            
            // Cache bounds for all selected elements
            for element in state.selected_elements() {
                let hash = Self::element_hash(&element);
                self.element_bounds.insert(hash, compute_element_rect(&element));
            }
            
            self.last_version = state.version();
        }
    }
    
    /// Get cached bounds for an element if available
    pub fn get_bounds(&self, element: &ElementType) -> Option<&Rect> {
        let hash = Self::element_hash(element);
        self.element_bounds.get(&hash)
    }
    
    /// Check if a point is near any cached element's handle
    pub fn is_point_near_any_handle(&self, pos: egui::Pos2) -> bool {
        for (_hash, rect) in &self.element_bounds {
            // Check all four corners
            let corners = [
                rect.left_top(),
                rect.right_top(),
                rect.left_bottom(),
                rect.right_bottom(),
            ];

            for corner in corners.iter() {
                let distance = pos.distance(*corner);
                if distance <= RESIZE_HANDLE_RADIUS {
                    return true;
                }
            }
        }
        false
    }
}

// MUST MAINTAIN EXISTING BEHAVIOR FOR BOTH ELEMENT TYPES
pub fn compute_element_rect(element: &crate::state::ElementType) -> egui::Rect {
    // Get the base rectangle from the Element trait
    let base_rect = element.rect();
    
    // Apply padding based on element type
    match element {
        crate::state::ElementType::Stroke(stroke) => {
            // For strokes, add the base padding
            let padding = STROKE_BASE_PADDING;
            
            egui::Rect::from_min_max(
                egui::pos2(base_rect.min.x - padding, base_rect.min.y - padding),
                egui::pos2(base_rect.max.x + padding, base_rect.max.y + padding),
            )
        }
        crate::state::ElementType::Image(_) => {
            // For images, add the image padding
            egui::Rect::from_min_max(
                egui::pos2(base_rect.min.x - IMAGE_PADDING, base_rect.min.y - IMAGE_PADDING),
                egui::pos2(base_rect.max.x + IMAGE_PADDING, base_rect.max.y + IMAGE_PADDING),
            )
        }
    }
}

// MUST REPLICATE EXACT HIT TESTING BEHAVIOR
pub fn is_point_near_handle(pos: egui::Pos2, element: &crate::state::ElementType) -> bool {
    let rect = compute_element_rect(element);
    
    // PRESERVE CORNER CHECK ORDER
    let corners = [
        (rect.left_top(), "left_top"),
        (rect.right_top(), "right_top"),
        (rect.left_bottom(), "left_bottom"),
        (rect.right_bottom(), "right_bottom"),
    ];

    for (corner, name) in corners.iter() {
        let distance = pos.distance(*corner);
        
        if distance <= RESIZE_HANDLE_RADIUS {
            println!("Found resize handle at corner: {}, distance: {}", name, distance);
            return true;
        }
    }
    
    false
} 