#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod command;
pub mod document;
pub mod event;
pub mod gizmo;
pub mod layer;
pub mod renderer;
pub mod selection;
pub mod state;
pub mod stroke;
pub mod tool;
pub mod input;
pub mod util;

// Re-export commonly used types
pub use app::PaintApp;
pub use document::Document;
pub use layer::{Layer, LayerContent, Transform};
pub use selection::Selection;
pub use stroke::Stroke;
pub use tool::Tool;