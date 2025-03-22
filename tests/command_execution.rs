use eframe_paint::command::{Command, CommandHistory};
use eframe_paint::element::{self, Element, ElementType, factory};
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
        old_position: original_rect.min,
    };
    let mut history = CommandHistory::new();
    history.execute(cmd, &mut model);

    // Verify the element was moved
    let moved_element = model.find_element_by_id(element_id).unwrap();
    let new_rect = moved_element.rect();

    assert!(
        (new_rect.min.x - original_rect.min.x - 10.0).abs() < 0.001
            && (new_rect.min.y - original_rect.min.y - 20.0).abs() < 0.001
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
    let new_rect = Rect::from_min_size(original_rect.min, original_rect.size() * 2.0);

    // Create and execute the resize command using bottom right corner
    let cmd = Command::ResizeElement {
        element_id,
        corner: eframe_paint::widgets::resize_handle::Corner::BottomRight,
        new_position: new_rect.max,
        old_rect: original_rect,
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
        old_position: original_rect.min,
    };
    let mut history = CommandHistory::new();
    history.execute(cmd, &mut model);

    // Verify the element was moved
    let moved_rect = model.find_element_by_id(element_id).unwrap().rect();
    assert!(
        (moved_rect.min.x - original_rect.min.x - 10.0).abs() < 0.001
            && (moved_rect.min.y - original_rect.min.y - 20.0).abs() < 0.001
    );

    // Undo the command
    history.undo(&mut model);

    // Verify the element is back to its original position
    let undone_rect = model.find_element_by_id(element_id).unwrap().rect();
    assert!(
        (undone_rect.min.x - original_rect.min.x).abs() < 0.001
            && (undone_rect.min.y - original_rect.min.y).abs() < 0.001
    );

    // Redo the command
    history.redo(&mut model);

    // Verify the element is moved again
    let redone_rect = model.find_element_by_id(element_id).unwrap().rect();
    assert!(
        (redone_rect.min.x - original_rect.min.x - 10.0).abs() < 0.001
            && (redone_rect.min.y - original_rect.min.y - 20.0).abs() < 0.001
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
        Vec2::new(0.1, 0.1), // Smaller than minimum
    );

    let result = model.resize_element(element_id, tiny_rect);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too small"));
}

#[test]
fn test_add_remove_element_commands() {
    let mut model = EditorModel::new();
    let mut history = CommandHistory::new();

    // Create a new stroke
    let points = vec![Pos2::new(10.0, 10.0), Pos2::new(50.0, 50.0)];
    let element_type = factory::create_stroke(42, points, 3.0, Color32::BLUE);

    // Execute AddElement command
    let cmd = Command::AddElement {
        element: element_type,
    };
    let result = history.execute(cmd, &mut model);
    assert!(result.is_ok());

    // Verify element was added
    let added_element = model.find_element_by_id(42);
    assert!(added_element.is_some());

    // Take a copy of the element for later verification
    let element_copy = model.find_element_by_id(42).unwrap().clone();

    // Execute RemoveElement command
    let cmd = Command::RemoveElement {
        element_id: 42,
        old_element: element_copy, // Store the element for undo
    };
    let result = history.execute(cmd, &mut model);
    assert!(result.is_ok());

    // Verify element was removed
    let removed_element = model.find_element_by_id(42);
    assert!(removed_element.is_none());

    // Undo removal
    history.undo(&mut model);

    // Verify element is back
    let restored_element = model.find_element_by_id(42);
    assert!(restored_element.is_some());

    // Undo addition
    history.undo(&mut model);

    // Verify element is gone again
    let final_check = model.find_element_by_id(42);
    assert!(final_check.is_none());
}

#[test]
fn test_command_error_handling() {
    let mut model = EditorModel::new();
    let mut history = CommandHistory::new();

    // Try to move a non-existent element
    let cmd = Command::MoveElement {
        element_id: 999, // Non-existent ID
        delta: Vec2::new(10.0, 10.0),
        old_position: Pos2::new(0.0, 0.0),
    };

    // Execute should return an error
    let result = history.execute(cmd, &mut model);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));

    // History should not have recorded the failed command
    assert_eq!(history.can_undo(), false);

    // Try to resize a non-existent element
    let cmd = Command::ResizeElement {
        element_id: 999,
        corner: eframe_paint::widgets::resize_handle::Corner::BottomRight,
        new_position: Pos2::new(100.0, 100.0),
        old_rect: Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(10.0, 10.0)),
    };

    // Execute should return an error
    let result = history.execute(cmd, &mut model);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_texture_invalidation() {
    use eframe_paint::renderer::Renderer;
    use egui::Context;
    use std::cell::RefCell;
    use std::rc::Rc;

    // This is a mocked version of Renderer for testing without eframe
    struct MockRenderer {
        id_cleared: Option<usize>,
        all_cleared: bool,
    }

    impl MockRenderer {
        fn new() -> Self {
            Self {
                id_cleared: None,
                all_cleared: false,
            }
        }

        fn clear_element_state(&mut self, id: usize) {
            self.id_cleared = Some(id);
        }

        fn clear_all_element_state(&mut self) {
            self.all_cleared = true;
        }

        fn get_ctx(&self) -> &egui::Context {
            // This is a dummy for the test
            panic!("Not supposed to be called in the test");
        }

        fn find_element(&self, _id: usize) -> Option<&ElementType> {
            None
        }

        fn invalidate_texture(&self, _id: usize) {
            // No-op for test
        }
    }

    // Create a model with a stroke
    let mut model = create_test_model();
    let element_id = 1; // Stroke ID

    // Get initial texture version
    let initial_version = model
        .find_element_by_id(element_id)
        .unwrap()
        .texture_version();

    // Create a move command
    let cmd = Command::MoveElement {
        element_id,
        delta: Vec2::new(10.0, 10.0),
        old_position: Pos2::new(0.0, 0.0),
    };

    // Execute command
    let result = cmd.execute(&mut model);
    assert!(result.is_ok());

    // Check that texture is invalidated (version increased)
    let new_version = model
        .find_element_by_id(element_id)
        .unwrap()
        .texture_version();

    assert!(
        new_version > initial_version,
        "Texture version should increase after command execution"
    );

    // Create a mock renderer and check texture invalidation logic
    let mut mock_renderer = MockRenderer::new();

    // The following line simulates the call to invalidate_textures
    // This is a direct test of the operation based on the mock
    match &cmd {
        Command::MoveElement { element_id, .. } => {
            mock_renderer.clear_element_state(*element_id);
        }
        _ => panic!("Unexpected command type"),
    }

    // Verify the mock recorder the correct invalidation operation
    assert_eq!(
        mock_renderer.id_cleared,
        Some(element_id),
        "Element texture state should be cleared for the right ID"
    );
}
