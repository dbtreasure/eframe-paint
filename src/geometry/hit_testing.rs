// STRICTLY KEEP THESE CONSTANTS
pub const RESIZE_HANDLE_RADIUS: f32 = 15.0;
pub const STROKE_BASE_PADDING: f32 = 10.0;
pub const IMAGE_PADDING: f32 = 10.0;

// MUST MAINTAIN EXISTING BEHAVIOR FOR BOTH ELEMENT TYPES
pub fn compute_element_rect(element: &crate::state::ElementType) -> egui::Rect {
    match element {
        crate::state::ElementType::Stroke(stroke) => {
            let points = stroke.points();
            if points.is_empty() {
                return egui::Rect::NOTHING;
            }

            // PRESERVE EXACT BOUNDING BOX CALCULATION
            let mut min_x = points[0].x;
            let mut min_y = points[0].y;
            let mut max_x = points[0].x;
            let mut max_y = points[0].y;

            for point in points {
                min_x = min_x.min(point.x);
                min_y = min_y.min(point.y);
                max_x = max_x.max(point.x);
                max_y = max_y.max(point.y);
            }

            // KEEP ORIGINAL PADDING LOGIC
            let padding = STROKE_BASE_PADDING + stroke.thickness();
            
            let rect = egui::Rect::from_min_max(
                egui::pos2(min_x - padding, min_y - padding),
                egui::pos2(max_x + padding, max_y + padding),
            );
            
            println!("Stroke bounding box: {:?}", rect);
            rect
        }
        crate::state::ElementType::Image(image) => {
            // For images, use the image's rect with some padding
            let rect = image.rect();
            // MAINTAIN IMAGE PADDING BEHAVIOR
            let padded_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x - IMAGE_PADDING, rect.min.y - IMAGE_PADDING),
                egui::pos2(rect.max.x + IMAGE_PADDING, rect.max.y + IMAGE_PADDING),
            );
            
            println!("Image bounding box: {:?}", padded_rect);
            padded_rect
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