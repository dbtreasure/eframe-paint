#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod renderer;
pub mod document;
pub mod stroke;
pub mod command;
pub mod panels;
pub mod input;
pub mod tools;
pub mod image;
pub mod file_handler;
pub mod state;
pub mod geometry;
pub mod error;

pub use app::PaintApp;
pub use renderer::Renderer;
pub use document::Document;
pub use stroke::Stroke;
pub use command::Command;
pub use command::CommandHistory;
pub use input::{InputEvent, InputLocation};
pub use tools::Tool;
pub use tools::DrawStrokeToolType;
pub use tools::new_draw_stroke_tool;
pub use file_handler::FileHandler;
pub use state::EditorState;