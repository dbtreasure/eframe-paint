#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod renderer;
mod document;
pub use app::PaintApp;
pub use renderer::Renderer;
pub use document::Document;