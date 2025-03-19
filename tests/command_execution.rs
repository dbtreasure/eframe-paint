use eframe_paint::element::{self, Element, ElementType, factory};
use eframe_paint::command::{Command, CommandHistory};
use eframe_paint::state::EditorModel;
use egui::{Color32, Pos2, Rect, Vec2};

// Helper to create a test model with some predefined elements
fn create_test_model() -> EditorModel {
    let mut model = EditorModel::new();
    
    // Add a stroke
    let points = vec![Pos2::new(10.0, 10.0), Pos2::new(30.0, 30.0)];
    let stroke = factory::create_stroke(1, points, 2.0, Color32::RED);
    model.add_element(stroke);
    
    // Add an image
    let data = vec![0u8; 100]; // Dummy data
    let size = Vec2::new(100.0, 100.0);
    let position = Pos2::new(50.0, 50.0);
    let image = factory::create_image(2, data, size, position);
    model.add_element(image);
    
    model
}

#[test]
fn test_move_command_ownership_transfer() {
    let mut model = create_test_model();
    let element_id = 1; // Our stroke ID
    let delta = Vec2::new(10.0, 20.0);
    
    // Get the initial position
    let original_rect = model.find_element_by_id(element_id).unwrap().rect();
    
    // Create and execute the move command
    let cmd = Command::MoveElement {
        element_id,
        delta,
        original_element: None, // Test without original element to force ownership transfer
    };
    let mut history = CommandHistory::new();
    history.execute(cmd, &mut model);
    
    // Verify the element was moved
    let moved_element = model.find_element_by_id(element_id).unwrap();
    let new_rect = moved_element.rect();
    
    assert!(
        (new_rect.min.x - original_rect.min.x - 10.0).abs() < 0.001 &&
        (new_rect.min.y - original_rect.min.y - 20.0).abs() < 0.001
    );
}

#[test]
fn test_resize_command_ownership_transfer() {
    let mut model = create_test_model();
    let element_id = 2; // Our image ID
    
    // Get the original element and its rect
    let element = model.find_element_by_id(element_id).unwrap();
    let original_rect = element.rect();
    
    // Create a new rect that's twice the size
    let new_rect = Rect::from_min_size(
        original_rect.min,
        original_rect.size() * 2.0
    );
    
    // Create and execute the resize command using bottom right corner
    let cmd = Command::ResizeElement {
        element_id,
        corner: eframe_paint::widgets::resize_handle::Corner::BottomRight,
        new_position: new_rect.max,
        original_element: None, // Test without original element to force ownership transfer
    };
    let mut history = CommandHistory::new();
    history.execute(cmd, &mut model);
    
    // Verify the element was resized
    let resized_element = model.find_element_by_id(element_id).unwrap();
    let final_rect = resized_element.rect();
    
    assert!(final_rect.width() > original_rect.width() * 1.9); // Allow small rounding differences
    assert!(final_rect.height() > original_rect.height() * 1.9);
}

#[test]
fn test_command_undo_redo() {
    let mut model = create_test_model();
    let element_id = 1; // Our stroke ID
    let delta = Vec2::new(10.0, 20.0);
    
    // Get the initial position
    let original_rect = model.find_element_by_id(element_id).unwrap().rect();
    
    // Create and execute the move command
    let cmd = Command::MoveElement {
        element_id,
        delta,
        original_element: None,
    };
    let mut history = CommandHistory::new();
    history.execute(cmd, &mut model);
    
    // Verify the element was moved
    let moved_rect = model.find_element_by_id(element_id).unwrap().rect();
    assert!(
        (moved_rect.min.x - original_rect.min.x - 10.0).abs() < 0.001 &&
        (moved_rect.min.y - original_rect.min.y - 20.0).abs() < 0.001
    );
    
    // Undo the command
    history.undo(&mut model);
    
    // Verify the element is back to its original position
    let undone_rect = model.find_element_by_id(element_id).unwrap().rect();
    assert!(
        (undone_rect.min.x - original_rect.min.x).abs() < 0.001 &&
        (undone_rect.min.y - original_rect.min.y).abs() < 0.001
    );
    
    // Redo the command
    history.redo(&mut model);
    
    // Verify the element is moved again
    let redone_rect = model.find_element_by_id(element_id).unwrap().rect();
    assert!(
        (redone_rect.min.x - original_rect.min.x - 10.0).abs() < 0.001 &&
        (redone_rect.min.y - original_rect.min.y - 20.0).abs() < 0.001
    );
}

#[test]
fn test_element_creation_validation() {
    // Test creating a stroke with too few points (should still work, but won't render)
    let points = vec![Pos2::new(10.0, 10.0)]; // Just one point
    let stroke = factory::create_stroke(1, points, 2.0, Color32::RED);
    
    // The element should be created but its rect may be empty
    let rect = stroke.rect();
    assert!(rect.width() < 5.0 && rect.height() < 5.0); // Very small or empty
    
    // Test resize validation fails with too small dimensions
    let mut model = create_test_model();
    let element_id = 1; // Stroke
    
    // Try to resize to a very small rect
    let tiny_rect = Rect::from_min_size(
        Pos2::new(10.0, 10.0),
        Vec2::new(0.1, 0.1) // Smaller than minimum
    );
    
    let result = model.resize_element(element_id, tiny_rect);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too small"));
}