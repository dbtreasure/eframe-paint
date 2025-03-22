use eframe_paint::element::{self, Element, ElementType};
use egui::{Color32, Pos2, Rect, Vec2};

fn create_test_stroke() -> ElementType {
    let points = vec![Pos2::new(10.0, 10.0), Pos2::new(20.0, 20.0)];
    element::factory::create_stroke(1, points, 2.0, Color32::RED)
}

fn create_test_image() -> ElementType {
    let data = vec![0u8; 100]; // Dummy data
    let size = Vec2::new(100.0, 50.0);
    let position = Pos2::new(10.0, 20.0);
    element::factory::create_image(2, data, size, position)
}

#[test]
fn test_element_creation() {
    // Create a stroke
    let stroke = create_test_stroke();
    assert_eq!(stroke.id(), 1);
    assert_eq!(stroke.element_type(), "stroke");

    // Create an image
    let image = create_test_image();
    assert_eq!(image.id(), 2);
    assert_eq!(image.element_type(), "image");
}

#[test]
fn test_element_rect() {
    // Check stroke rectangle
    let stroke = create_test_stroke();
    let rect = stroke.rect();

    // Rectangle should contain all points
    assert!(rect.contains(Pos2::new(10.0, 10.0)));
    assert!(rect.contains(Pos2::new(20.0, 20.0)));

    // Check image rectangle
    let image = create_test_image();
    let rect = image.rect();

    assert_eq!(rect.min, Pos2::new(10.0, 20.0));
    assert_eq!(rect.size(), Vec2::new(100.0, 50.0));
}

#[test]
fn test_element_translate() {
    // Test stroke translation
    let mut stroke = create_test_stroke();
    let original_rect = stroke.rect();

    // Translate by (5, 10)
    let delta = Vec2::new(5.0, 10.0);
    stroke.translate(delta).unwrap();

    let new_rect = stroke.rect();

    // Check that the rectangle has been translated correctly
    assert!((new_rect.min.x - original_rect.min.x - 5.0).abs() < 0.001);
    assert!((new_rect.min.y - original_rect.min.y - 10.0).abs() < 0.001);
    assert!((new_rect.max.x - original_rect.max.x - 5.0).abs() < 0.001);
    assert!((new_rect.max.y - original_rect.max.y - 10.0).abs() < 0.001);

    // Test image translation
    let mut image = create_test_image();
    let original_rect = image.rect();

    // Translate by (15, 25)
    let delta = Vec2::new(15.0, 25.0);
    image.translate(delta).unwrap();

    let new_rect = image.rect();

    // Check that the rectangle has been translated correctly
    assert_eq!(new_rect.min.x, original_rect.min.x + 15.0);
    assert_eq!(new_rect.min.y, original_rect.min.y + 25.0);
    assert_eq!(new_rect.max.x, original_rect.max.x + 15.0);
    assert_eq!(new_rect.max.y, original_rect.max.y + 25.0);
}

#[test]
fn test_element_resize() {
    // Test stroke resizing
    let mut stroke = create_test_stroke();
    let original_rect = stroke.rect();

    // Double the size
    let new_rect = Rect::from_min_size(original_rect.min, original_rect.size() * 2.0);

    stroke.resize(new_rect.clone()).unwrap();

    // Check that the rectangle has been resized correctly
    let resized_rect = stroke.rect();
    assert!(resized_rect.width() > original_rect.width() * 1.9); // Allow for small rounding errors
    assert!(resized_rect.height() > original_rect.height() * 1.9);

    // Test image resizing
    let mut image = create_test_image();
    let original_rect = image.rect();

    // Half the size
    let new_rect = Rect::from_min_size(original_rect.min, original_rect.size() / 2.0);

    image.resize(new_rect.clone()).unwrap();

    // Check that the rectangle has been resized correctly
    let resized_rect = image.rect();
    assert_eq!(resized_rect.width(), original_rect.width() / 2.0);
    assert_eq!(resized_rect.height(), original_rect.height() / 2.0);
}

#[test]
fn test_element_hit_testing() {
    // Create a stroke
    let stroke = create_test_stroke();

    // Point on the line should be a hit
    assert!(stroke.hit_test(Pos2::new(15.0, 15.0)));

    // Point far from the line should not be a hit
    assert!(!stroke.hit_test(Pos2::new(50.0, 50.0)));

    // Create an image
    let image = create_test_image();

    // Point inside the image should be a hit
    assert!(image.hit_test(Pos2::new(50.0, 40.0)));

    // Point outside the image should not be a hit
    assert!(!image.hit_test(Pos2::new(200.0, 200.0)));
}

#[test]
fn test_element_texture_invalidation() {
    // Create a stroke
    let mut stroke = create_test_stroke();

    // Initial texture version should be 0
    assert_eq!(stroke.texture_version(), 0);
    assert!(stroke.needs_texture_update());

    // Modify the element
    stroke.translate(Vec2::new(5.0, 5.0)).unwrap();

    // Texture version should be incremented
    assert_eq!(stroke.texture_version(), 1);
    assert!(stroke.needs_texture_update());
}

#[test]
fn test_invalid_element_operations() {
    // Test resize with invalid dimensions
    let mut stroke = create_test_stroke();

    // Try to resize to a very small rectangle (should fail)
    let tiny_rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(0.1, 0.1));

    let result = stroke.resize(tiny_rect);
    assert!(result.is_err());

    // Error message should mention the minimum size
    let error = result.unwrap_err();
    assert!(error.contains("too small"));
}
