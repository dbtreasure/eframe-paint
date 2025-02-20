# Chapter 3: The Dawn of Painting – Integrating Graphics and Immediate Mode UI

In this chapter, we move from configuring our project to actually _bringing it to life_. This commit marks our first foray into graphics territory by introducing basic painting functionality. It's the moment when our application starts to "paint" on the screen, combining Rust's power with GPU-based rendering.

For readers new to Rust, graphics programming, or immediate mode GUIs, this chapter is designed to demystify the process. We'll break down the new code into digestible pieces, explain key Rust concepts, and shed light on how libraries like @eframe, @egui, and the OpenGL binding (@glow) come together to power our app.

## Introducing the Renderer Module

The heart of our new functionality is the **Renderer** module. This module encapsulates GPU-based rendering by taking advantage of the OpenGL context provided by @eframe (via the @glow library). Its main responsibilities are:

- **Initializing GPU Resources:** It clones the OpenGL context from the eframe creation context.
- **Rendering Graphics:** It contains a method to draw graphics—in our case, a simple semi-transparent blue rectangle.

Here's a snippet from `src/renderer.rs` that illustrates these ideas:

```rust
// src/renderer.rs
use eframe::egui;
use eframe::glow::HasContext; // This trait gives access to OpenGL functions

pub struct Renderer {
    // Holds an optional GPU context; using Option because GPU resources are non-serializable
    gl: Option<std::sync::Arc<eframe::glow::Context>>,
}

impl Renderer {
    /// Create a new Renderer instance by cloning the OpenGL context provided in the creation context.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone();
        Self { gl }
    }

    /// Render the current frame. Here, we simply fill the specified rectangle with a semi-transparent blue color.
    pub fn render(&mut self, ctx: &egui::Context, painter: &egui::Painter, rect: egui::Rect) {
        // Draw a semi-transparent blue rectangle to serve as our background
        painter.rect_filled(
            rect,
            0.0, // No rounded corners
            egui::Color32::from_rgba_premultiplied(0, 127, 255, 200),
        );

        // Request a repaint to achieve continuous animation if needed
        ctx.request_repaint();
    }
}
```

Notice how the `Renderer::new` function grabs the GPU context (via `cc.gl.clone()`), ensuring our renderer has access to the necessary graphics API. The `render` method then uses @egui's Painter API to draw directly onto the UI frame.

## Integrating Rendering into PaintApp

Next, the **PaintApp** structure was updated to incorporate our new Renderer as well as to manage UI state for a modal window. The changes include:

- Adding a `renderer` field (wrapped in `Option`) to hold the Renderer instance. This field is marked with `#[serde(skip)]` because GPU resources cannot be serialized.
- Adding a `show_modal` boolean flag to manage the display of a modal window.

Here's an excerpt from the updated `src/app.rs`:

```rust
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PaintApp {
    // Renderer is skipped during serialization since it holds GPU resources
    #[serde(skip)]
    renderer: Option<Renderer>,
    // Flag to control the display of a modal window
    #[serde(skip)]
    show_modal: bool,
}

impl Default for PaintApp {
    fn default() -> Self {
        Self {
            renderer: None,
            show_modal: false,
        }
    }
}

impl PaintApp {
    /// Called once before the first frame is rendered
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize the renderer with the GPU context
        let renderer = Renderer::new(cc);
        Self {
            renderer: Some(renderer),
            show_modal: false,
        }
    }
}
```

This design ensures that our non-serializable GPU context does not interfere with state persistence while still allowing us to update and render graphics every frame.

## Painting Each Frame: The Update Loop

The real magic happens in the `update` method of the PaintApp, where the UI is constructed and the renderer is called to paint on the screen. Here's a closer look at this method:

```rust
impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set up the central panel of the UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Paint App");

            // Allocate a painter area that fills the available space
            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(
                available_size,
                egui::Sense::drag()
            );

            // Define the window/area where painting will occur
            let rect = response.rect;

            // Invoke the renderer to draw our semi-transparent blue background
            if let Some(renderer) = &mut self.renderer {
                renderer.render(ctx, &painter, rect);
            }

            // Overlay an "Open Modal" button at the center
            ui.put(
                egui::Rect::from_center_size(rect.center(), egui::vec2(100.0, 30.0)),
                egui::Button::new("Open Modal")
            ).clicked().then(|| self.show_modal = true);
        });

        // If the modal state is active, display the modal window
        if self.show_modal {
            egui::Window::new("Example Modal")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("This is a modal window!");
                    if ui.button("Close").clicked() {
                        self.show_modal = false;
                    }
                });
        }
    }
}
```

This method illustrates several important Rust and UI programming concepts:

- **Immediate Mode Rendering:** Each frame, the UI and painting area are rebuilt from scratch. This means that every frame you get a fresh canvas, making it easy to implement dynamic and interactive graphics.
- **Allocation of a Painter:** The call to `ui.allocate_painter` provides a drawing surface that integrates seamlessly with other UI elements. You don't have to manage complex state or persistent graphics contexts manually.
- **Conditional UI Rendering:** By checking the `show_modal` flag, we conditionally display a modal window—a powerful feature that shows how logic and rendering can be intertwined in an immediate mode GUI.

## Bridging the Gap Between Graphics and UI

This commit is a turning point: it blends low-level graphics rendering with high-level UI construction. For an engineer new to Rust, notice how the Renderer abstracts away the complexity of GPU programming, allowing you to focus on building interactive applications.

Key takeaways:

- **Rust's Ownership and Type Safety:** By leveraging Rust's strong typing and ownership model, we ensure that GPU resources are managed safely and efficiently.
- **Integration with @egui and @eframe:** These libraries provide a high-level interface for immediate mode GUIs, letting you mix traditional UI elements with custom drawing code.
- **Modularity:** The separation of concerns between the Renderer and PaintApp makes the code easier to manage and extend—critical for building larger applications.

## Conclusion

In this chapter, we witnessed the breaking of new ground with the introduction of painting functionality. By encapsulating GPU-based rendering into a dedicated module and integrating it with an immediate mode UI, we not only enabled basic painting but also set a clear path for future enhancements. As you continue your journey into Rust and graphics programming, remember that every frame is an opportunity to experiment, learn, and create.

Welcome to the art of programming—where code meets creativity!
