# 🎨 eframe_paint

A fun and intuitive painting application built with Rust and egui!

## ✨ Overview

Create digital art with ease using our lightweight and fast painting application. Whether you're sketching, drawing, or just doodling, eframe_paint provides all the essential tools you need.

## 🚀 Quick Start

```shell
# Clone and run
git clone https://github.com/yourusername/eframe_paint.git
cd eframe_paint
cargo run --release
```

### Testing locally

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Features

This paint application provides advanced drawing functionality:

- 🖌 Brush Tool - Free-form drawing with adjustable thickness
- ⌫ Eraser Tool - Remove parts of your drawing
- ◻ Selection Tool - Select areas of your drawing
- 🎨 Color Picker - Choose any color for your brush
- 📑 Advanced Layer Support:
  - Create and manage multiple layers
  - Transform layers with rotation, scaling, and translation
  - Reorder layers via drag and drop
  - Toggle layer visibility
  - Support for image layers
- ↩ Comprehensive Undo/Redo - Track your drawing and transformation history

### Testing the Features

1. **Basic Drawing**

   - Select the Brush tool and draw on the canvas
   - Adjust brush thickness using the slider
   - Change colors using the color picker

2. **Advanced Layer Management**

   - Create new layers using the layer panel
   - Transform layers:
     - Rotate layers to any angle
     - Scale layers up or down
     - Translate layers to new positions
   - Add image layers with full transformation support
   - Toggle layer visibility
   - Drag and drop to reorder layers
   - Rename layers as needed

3. **Selection and Editing**
   - Use the Selection tool to select parts of your drawing
   - Move and modify selected areas
   - Delete selections using the Eraser tool
   - Apply transformations to selected content

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.

1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.

### Web Deploy

1. Just run `trunk build --release`.
2. It will generate a `dist` directory as a "static html" website
3. Upload the `dist` directory to any of the numerous free hosting websites including [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
4. We already provide a workflow that auto-deploys our app to GitHub pages if you enable it.
   > To enable Github Pages, you need to go to Repository -> Settings -> Pages -> Source -> set to `gh-pages` branch and `/` (root).
   >
   > If `gh-pages` is not available in `Source`, just create and push a branch called `gh-pages` and it should be available.
   >
   > If you renamed the `main` branch to something else (say you re-initialized the repository with `master` as the initial branch), be sure to edit the github workflows `.github/workflows/pages.yml` file to reflect the change
   >
   > ```yml
   > on:
   >   push:
   >     branches:
   >       - <branch name>
   > ```

## 📜 Development History

Curious about how eframe_paint was developed? Dive into our [Development History Introduction](Development_History/Introduction.md) to learn about the technical evolution, design decisions, and creative breakthroughs that shaped this project from its inception to its current state.
