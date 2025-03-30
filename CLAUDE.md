# Qlty Development Guide for AI Assistants

## Build & Test Commands

- Typecheck (faster than build): `cargo check`
- To auto-format or lint always used this command: `qlty check --fix`
- Run all tests: `cargo test`
- Build: `cargo build`
- Run specific test: `cargo test test_name_here`
- NEVER use `cargo insta review` to accept snapshots. Instead use `INSTA_UPDATE=always cargo test ...`

## Code Style Guidelines

- Follow standard Rust idioms and `.qlty/configs/.rustfmt.toml` settings
- Use `anyhow::Error` for errors and `thiserror` for defining error types
- Use `anyhow::Result` for return values instead of the built-in `Result`
- Naming: snake_case for functions/variables, UpperCamelCase for types/enums
- Always use strong typing with enums for bounded sets of values
- Imports: group std first, then external crates, then internal modules
- Comprehensive error handling with proper context using `context()` or `with_context()`
- Use descriptive variable names that clearly express intent
- Write docstrings for public APIs and complex functions

## Testing

- Unit tests live below implementation `#[cfg(test)]` blocks
- Integration tests live in `tests/` in each crate
- Test one thing per test
- Do not add comments to tests
- Do not use custom assertion messages
- Do not use control flow like if statements or loops in tests
- `.unwrap()` is OK to use in tests

## Development Workflow

- Never commit to `main` branch. Always work on a new branch from `main` with a descriptive name
- IMPORTANT: Before every commit, typecheck, run auto-formatting and linting, and run all the tests
- Always open PRs in draft mode
