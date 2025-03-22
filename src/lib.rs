#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod command;
pub mod element;
pub mod file_handler;
pub mod id_generator;
pub mod input;
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
pub use input::{InputEvent, InputLocation};
pub use renderer::Renderer;
pub use state::EditorModel;
pub use texture_manager::TextureManager;
pub use tools::Tool;
pub use tools::UnifiedDrawStrokeTool;
pub use tools::UnifiedSelectionTool;
pub use tools::new_draw_stroke_tool;
pub use tools::new_selection_tool;
pub use widgets::{Corner, ResizeHandle};
