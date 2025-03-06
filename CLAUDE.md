# eframe-paint Development Guidelines

## Build Commands
- Build: `cargo build` (native) or `trunk build` (web)
- Run: `cargo run`
- Run tests: `cargo test --workspace --all-targets --all-features`
- Run specific test: `cargo test test_name -- --nocapture`
- Lint: `cargo clippy --workspace --all-targets --all-features`
- Format: `cargo fmt --all`
- CI checks: `./check.sh`

## Code Style
- Snake_case for variables, functions, methods
- PascalCase for types, traits, enums, structs
- 4-space indentation, consistent spacing around operators
- Group imports: external crates first, then internal with `crate::`
- Descriptive variable/function names that indicate purpose
- Document public APIs with `///` comments
- Use Result<T, String> for error handling with descriptive messages
- Use `log::info!`, `log::warn!` for logging important operations
- Prefer immutable variables and explicit ownership
- Follow command pattern for operations, state pattern for tools
- Keep methods focused on single responsibility