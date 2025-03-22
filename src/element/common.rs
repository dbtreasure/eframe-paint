use egui::{Pos2, Rect};

// Common constants for all element types
pub const MIN_ELEMENT_SIZE: f32 = 2.0;
pub const STROKE_BASE_PADDING: f32 = 10.0;
pub const IMAGE_PADDING: f32 = 10.0;

/// Validates that a rectangle has minimum dimensions
pub(crate) fn validate_rect(rect: &Rect) -> Result<(), String> {
    if rect.width() < MIN_ELEMENT_SIZE || rect.height() < MIN_ELEMENT_SIZE {
        Err(format!(
            "Element dimensions too small (min: {}). Width: {}, Height: {}",
            MIN_ELEMENT_SIZE,
            rect.width(),
            rect.height()
        ))
    } else {
        Ok(())
    }
}

/// Calculate distance from a point to a line segment (useful for stroke hit testing)
pub(crate) fn distance_to_line_segment(point: Pos2, line_start: Pos2, line_end: Pos2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;

    let line_len = line_vec.length();
    if line_len == 0.0 {
        return point_vec.length();
    }

    let t = ((point_vec.x * line_vec.x + point_vec.y * line_vec.y) / line_len).clamp(0.0, line_len);
    let projection = line_start + (line_vec * t / line_len);
    (point - projection).length()
}

/// Calculate the bounding box for a set of points
pub(crate) fn calculate_bounds(points: &[Pos2], padding: f32) -> Rect {
    if points.is_empty() {
        return Rect::NOTHING;
    }

    // Calculate the bounding box of all points
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    Rect::from_min_max(
        Pos2::new(min_x - padding, min_y - padding),
        Pos2::new(max_x + padding, max_y + padding),
    )
}

// Unused utility functions have been removed
