use eframe_paint::element::{self, Element, ElementType, compute_element_rect};
use egui::{Color32, Pos2, Rect, Vec2};

#[test]
fn test_stroke_factory() {
    // Create a stroke using the factory
    let stroke = element::factory::create_stroke(
        1,
        vec![Pos2::new(10.0, 10.0), Pos2::new(20.0, 20.0)],
        2.0,
        Color32::RED
    );
    
    // Verify stroke properties
    assert_eq!(stroke.id(), 1);
    assert_eq!(stroke.element_type(), "stroke");
    
    // Verify rect calculation
    let rect = stroke.rect();
    assert!(rect.min.x <= 10.0);
    assert!(rect.min.y <= 10.0);
    assert!(rect.max.x >= 20.0);
    assert!(rect.max.y >= 20.0);
}

#[test]
fn test_image_factory() {
    // Create test image data (2x2 red pixel)
    let data = vec![
        255, 0, 0, 255,
        255, 0, 0, 255,
        255, 0, 0, 255,
        255, 0, 0, 255,
    ];
    
    // Create an image using the factory
    let image = element::factory::create_image(
        2,
        data,
        Vec2::new(2.0, 2.0),
        Pos2::new(5.0, 5.0)
    );
    
    // Verify image properties
    assert_eq!(image.id(), 2);
    assert_eq!(image.element_type(), "image");
    
    // Verify rect calculation
    let rect = image.rect();
    assert_eq!(rect.min, Pos2::new(5.0, 5.0));
    assert_eq!(rect.max, Pos2::new(7.0, 7.0));
}

#[test]
fn test_element_transformation() {
    // Create a stroke using the factory
    let mut stroke = element::factory::create_stroke(
        3,
        vec![Pos2::new(10.0, 10.0), Pos2::new(20.0, 20.0)],
        2.0,
        Color32::RED
    );
    
    // Get original rect
    let original_rect = compute_element_rect(&stroke);
    
    // Translate the element
    let delta = Vec2::new(5.0, 10.0);
    stroke.translate(delta).unwrap();
    
    // Get new rect
    let new_rect = compute_element_rect(&stroke);
    
    // Verify translation
    assert_eq!(new_rect.min.x, original_rect.min.x + 5.0);
    assert_eq!(new_rect.min.y, original_rect.min.y + 10.0);
    
    // Test resizing
    let mut image = element::factory::create_image(
        4,
        vec![0u8; 16], // 2x2 RGBA image
        Vec2::new(2.0, 2.0),
        Pos2::new(0.0, 0.0)
    );
    
    // Get original rect
    let original_rect = compute_element_rect(&image);
    
    // Create a new rect twice the size
    let new_rect = Rect::from_min_size(
        original_rect.min,
        Vec2::new(original_rect.width() * 2.0, original_rect.height() * 2.0)
    );
    
    // Resize the element
    image.resize(new_rect.clone()).unwrap();
    
    // Get the resized rect
    let resized_rect = compute_element_rect(&image);
    
    // Verify resizing (allowing for small rounding errors)
    assert!(resized_rect.width() > original_rect.width() * 1.9);
    assert!(resized_rect.height() > original_rect.height() * 1.9);
}

#[test]
fn test_element_texture_invalidation() {
    // Create elements using the factory
    let mut stroke = element::factory::create_stroke(
        5,
        vec![Pos2::new(10.0, 10.0), Pos2::new(20.0, 20.0)],
        2.0,
        Color32::RED
    );
    
    // Initial texture state
    assert!(stroke.needs_texture_update());
    
    // Get initial texture version
    let initial_version = stroke.texture_version();
    
    // Modify element
    stroke.translate(Vec2::new(10.0, 10.0)).unwrap();
    
    // Verify texture is invalidated
    assert!(stroke.needs_texture_update());
    assert!(stroke.texture_version() > initial_version);
}

#[test]
fn test_unsupported_operations() {
    // Test failure cases
    
    // Create a stroke with only one point (should not be allowed in real app)
    let stroke = element::factory::create_stroke(
        6,
        vec![Pos2::new(10.0, 10.0)],
        2.0,
        Color32::RED
    );
    
    // Create an empty image (should not be allowed in real app)
    let empty_image = element::factory::create_image(
        7,
        vec![],
        Vec2::new(0.0, 0.0),
        Pos2::new(0.0, 0.0)
    );
    
    // Create a valid image to test resize limits
    let mut image = element::factory::create_image(
        8,
        vec![0u8; 16], // 2x2 RGBA image 
        Vec2::new(2.0, 2.0),
        Pos2::new(0.0, 0.0)
    );
    
    // Try to resize to an invalid size (too small)
    let tiny_rect = Rect::from_min_size(
        Pos2::new(0.0, 0.0),
        Vec2::new(0.1, 0.1)
    );
    
    let result = image.resize(tiny_rect.clone());
    assert!(result.is_err());
    
    // Error message should mention minimum size
    let error = result.unwrap_err();
    assert!(error.contains("too small"));
}