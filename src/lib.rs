#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod renderer;
mod document;
mod stroke;
mod layer;
mod command;

pub use app::PaintApp;
pub use renderer::Renderer;
pub use document::Document;
pub use stroke::Stroke;
pub use layer::Layer;
pub use command::Command;