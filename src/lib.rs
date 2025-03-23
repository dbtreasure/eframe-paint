#![warn(clippy::all, rust_2018_idioms)]

//! # eframe-paint
//!
//! A simple drawing application built with egui/eframe.
//!
//! ## Architecture Overview
//!
//! The application follows a clean architecture with clear separation of concerns:
//!
//! - **UI Components**: The app and panels modules contain the user interface components
//! - **State Management**: The EditorModel manages application state and elements
//! - **Tools**: Tools handle user interactions and generate commands
//! - **Commands**: Commands represent actions that modify the application state
//! - **Rendering**: The Renderer handles visualization of elements and previews
//!
//! ## Input Handling Flow
//!
//! 1. User interacts with the application (mouse/keyboard)
//! 2. EditorPanel routes events directly to the active tool
//! 3. Tool processes the event and generates commands if needed
//! 4. Commands are executed against the EditorModel
//! 5. Tool updates its internal state and preview visualization
//! 6. Renderer displays the elements and any preview effects
//!
//! This architecture ensures that tools maintain their own state,
//! visualization is separate from logic, and the application state
//! is modified only through well-defined commands.

pub mod app;
pub mod command;
pub mod element;
pub mod file_handler;
pub mod id_generator;
pub mod panels;
pub mod renderer;
pub mod state;
pub mod texture_manager;
pub mod tools;
pub mod widgets;

pub use app::PaintApp;
pub use command::Command;
pub use command::CommandHistory;
pub use element::Element;
pub use element::ElementType;
pub use file_handler::FileHandler;
pub use renderer::Renderer;
pub use state::EditorModel;
pub use texture_manager::TextureManager;
pub use tools::Tool;
pub use tools::UnifiedDrawStrokeTool;
pub use tools::UnifiedSelectionTool;
pub use tools::new_draw_stroke_tool;
pub use tools::new_selection_tool;
pub use widgets::{Corner, ResizeHandle};
