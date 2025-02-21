#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod document;
mod layer;
mod renderer;
mod stroke;
mod command;
mod gizmo;
pub mod selection;

pub use app::PaintApp;
pub use document::Document;
pub use layer::Layer;
pub use renderer::Renderer;
pub use stroke::Stroke;
pub use command::Command;
pub use gizmo::TransformGizmo;