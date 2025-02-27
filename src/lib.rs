#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod renderer;
pub mod document;
pub mod stroke;
pub mod state;
pub mod command;
pub mod panels;
pub mod input;
pub mod tools;

pub use app::PaintApp;
pub use renderer::Renderer;
pub use document::Document;
pub use stroke::Stroke;
pub use state::EditorState;
pub use command::Command;
pub use command::CommandHistory;
pub use input::{InputEvent, InputLocation};
pub use tools::Tool;
pub use tools::DrawStrokeTool;
