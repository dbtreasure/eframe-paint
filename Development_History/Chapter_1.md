# Chapter 1: The Inception of Our App

In this initial commit, the foundation of our cross-platform GUI application was laid using the powerful Rust ecosystem. This chapter details the conception, setup, and key code decisions that form the backbone of our project.

## Project Setup and Key Libraries

The project is configured via a comprehensive `Cargo.toml` file which manages all dependencies and project metadata. Key libraries include:

- **@egui**: The core immediate-mode GUI library, which allows for fast and dynamic UI updates.
- **@eframe**: A framework layer built on top of @egui, providing support for both native and @Web deployments.
- Other essential libraries such as `serde` for serialization, `log` and `env_logger` for logging, which help ensure robust application behavior.

## Code Structure and Important Files

Several important files and directories were introduced in this commit:

- **Cargo.toml**: Sets up project dependencies and configurations.
- **src/main.rs**: Contains the entry point for both native and web builds. It detects the build target and configures the application accordingly.
- **src/app.rs**: Implements a simple demo application showcasing interactive UI elements like buttons, sliders, and text inputs. This file highlights the power of immediate mode programming provided by @egui.
- **src/lib.rs**: Exposes core application logic, allowing for reuse between different build targets.
- **Web Assets (@Web)**: Includes `index.html`, service worker (`assets/sw.js`), and associated assets, which together enable a smooth web experience with offline capabilities.

## Development Insights

This commit is more than just a code dump—it establishes a solid, well-organized foundation that promotes clarity and ease of contribution. The design decisions here reflect a commitment to:

- **Modularity and Clarity**: Separating the code for native and web builds ensures that developers new to Rust and GUI programming can understand each part independently.
- **Best Practices in Commit Messaging**: Inspired by resources such as [Git Commit Message Guidelines](https://github.com/joelparkerhenderson/git-commit-message), every change is meticulously documented to foster a transparent development process.
- **Immediate Mode GUI Principles**: Leveraging the simplicity and responsiveness of @egui, the project demonstrates how immediate mode UIs can result in a more interactive and flexible design.

## Conclusion

The initial commit marks the beginning of our journey into building a modern, cross-platform GUI application with Rust. It sets the stage for future developments by establishing a robust framework that can be expanded with more features and improvements.

As we move forward, this chapter will serve as a historical anchor, reminding us of the core principles and decisions that shaped the early development of our application.

## Code Samples and Walkthroughs

To better understand how the application works, we include selected code samples from the project:

### 1. Native vs. Web Entry Points

```rust
// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        // Example viewport and options configuration
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .build(),
        ..Default::default()
    };
    eframe::run_native("eframe template", native_options, Box::new(|cc| Ok(Box::new(eframe_template::TemplateApp::new(cc))))
}

// When targeting the web:
#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    // Web-specific initialization and canvas binding happens here.
}
```

### 2. Building the GUI with @egui and @eframe

```rust
impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("eframe template");
            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });
            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }
        });
    }
}
```

These samples provide a glimpse into the conditional build targets and the simplicity of creating and updating the UI. They illustrate how @eframe builds on the immediacy provided by @egui to create interactive, dynamic applications.

### 3. Project Manifest and Dependency Management (Cargo.toml)

```toml
[package]
name = "eframe_template"
version = "0.1.0"
authors = ["Emil Ernerfeldt <emil.ernerfeldt@gmail.com>"]
edition = "2021"
rust-version = "1.81"

[dependencies]
egui = "0.30"
efame = { version = "0.30", default-features = false, features = [
    "accesskit",
    "default_fonts",
    "glow",
    "persistence",
    "wayland"
] }
log = "0.4"
serde = { version = "1", features = ["derive"] }
```

This snippet illustrates the structured dependency management that sets up @eframe and @egui along with supportive libraries for logging and serialization.

### 4. Web HTML Setup for @Web Deployments

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>eframe template</title>
    <link data-trunk rel="rust" data-wasm-opt="2" />
    <link rel="manifest" href="manifest.json" />
  </head>
  <body>
    <canvas id="the_canvas_id"></canvas>
    <div id="loading_text">Loading…</div>
    <script>
      if ("serviceWorker" in navigator && window.location.hash !== "#dev") {
        window.addEventListener("load", function () {
          navigator.serviceWorker.register("sw.js");
        });
      }
    </script>
  </body>
</html>
```

This HTML structure is crucial for @Web deployments, enabling the application to run in browsers as a Progressive Web App (PWA) with offline capabilities.
