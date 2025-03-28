# eframe-paint

A simple drawing application built with egui/eframe.

## Application Architecture

The application follows a clean architecture with clear separation of concerns:

- **UI Components**: The app and panels modules contain the user interface components
- **State Management**: The EditorModel manages application state and elements
- **Tools**: Tools handle user interactions and generate commands
- **Commands**: Commands represent actions that modify the application state
- **Rendering**: The Renderer handles visualization of elements and previews

### Input Handling Flow

```
┌─────────────┐  Events   ┌─────────────┐  Commands  ┌─────────────┐
│  User Input │ ─────────▶│ Active Tool │ ──────────▶│ EditorModel │
└─────────────┘           └─────────────┘            └─────────────┘
                               │                           │
                               │ Preview                   │ Data
                               ▼                           ▼
                         ┌─────────────┐            ┌─────────────┐
                         │  Renderer   │ ◀─────────┤   Elements   │
                         └─────────────┘            └─────────────┘
                               │
                               │ Draw
                               ▼
                         ┌─────────────┐
                         │  UI Canvas  │
                         └─────────────┘
```

1. User interacts with the application (mouse/keyboard)
2. EditorPanel routes events directly to the active tool
3. Tool processes the event and generates commands if needed
4. Commands are executed against the EditorModel
5. Tool updates its internal state and preview visualization
6. Renderer displays the elements and any preview effects

This architecture ensures that tools maintain their own state, visualization is separate from logic, and the application state is modified only through well-defined commands.

## Element Architecture

The application uses a unified element representation system:

- All UI elements inherit from the `Element` trait
- Elements are created through the factory pattern in `element::factory`
- Element mutation follows an ownership transfer pattern:
  - Take element from model with `take_element_by_id`
  - Apply changes
  - Put element back with `add_element`
- Direct variant matching on `ElementType` is discouraged

### Testing locally

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

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
4. we already provide a workflow that auto-deploys our app to GitHub pages if you enable it.
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

You can test the template app at <https://emilk.github.io/eframe_paint/>.

## Updating egui

As of 2023, egui is in active development with frequent releases with breaking changes. [eframe_paint](https://github.com/emilk/eframe_paint/) will be updated in lock-step to always use the latest version of egui.

When updating `egui` and `eframe` it is recommended you do so one version at the time, and read about the changes in [the egui changelog](https://github.com/emilk/egui/blob/master/CHANGELOG.md) and [eframe changelog](https://github.com/emilk/egui/blob/master/crates/eframe/CHANGELOG.md).
