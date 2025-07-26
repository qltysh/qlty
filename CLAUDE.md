# Qlty Development Guide for AI Assistants

## Common commands

- Typecheck (faster than build): `cargo check`
- Run all tests: `cargo test`
- Run specific test: `cargo test test_name_here`
- Build: `cargo build`
- Auto-format: `qlty fmt`
- Lint: `qlty check --level=low --fix`

## Code Style Guidelines

- Follow standard Rust idioms
- Use `anyhow::Error` for errors and `thiserror` for defining error types
- Use `anyhow::Result` for return values instead of the built-in `Result`
- Add use directives for `anyhow` to import it instead of qualifying it
- Naming: snake_case for functions/variables, UpperCamelCase for types/enums
- Always use strong typing with enums for bounded sets of values
- Imports: group std first, then external crates, then internal modules
- Comprehensive error handling with proper context using `context()` or `with_context()`
- Use descriptive variable names that clearly express intent
- Do not add low value comments, let the code speak for itself unless there is something non-obvious

## Testing

- Unit tests live below implementation `#[cfg(test)]` blocks
- Integration tests live in `tests/` in each crate
- Test one thing per test
- Do not add comments to tests
- Do not use custom assertion messages
- Do not use control flow like if statements or loops in tests
- `.unwrap()` is OK to use in tests

## Important

- Never commit to `main` branch. Always work on a new branch from `main` with a descriptive name
- IMPORTANT: Before every commit, typecheck, run auto-formatting and linting, and run all the tests
- Always open PRs in draft mode
- Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format for commit messages
- Also use Conventional Commits format for PR titles and descriptions
