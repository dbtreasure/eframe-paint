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
pub mod components;

// Re-export commonly used types
pub use app::PaintApp;
pub use document::Document;
pub use layer::{Layer, LayerContent, Transform};
pub use selection::Selection;
pub use stroke::Stroke;
pub use tool::Tool;
pub use renderer::Renderer;
pub use gizmo::TransformGizmo;

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into());
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    }
}

// Re-export egui for convenience
pub use eframe::egui;

// Re-export image crate
pub use image;