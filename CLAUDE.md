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
- Do not remove or modify existing comments unless explicitly asked to

## Regular expressions

Reach for a regex last. Before writing one, try these in order:

1. **A parser, when one exists for the format.** `url::Url` for URLs, `serde_json`, `toml`, `globset` for path patterns. Use `std::path::Path` for paths on this machine, because `Path::starts_with("vendor")` doesn't match `vendored/x`. A path from another machine, such as one in a coverage report, may use `\`, so change it to `/` first. Check a URL with `url.scheme() == "https" && url.host_str() == Some("api.github.com")`. A prefix check such as `starts_with("https://api.github.com")` also accepts `https://api.github.com.evil.example`. If the workspace already depends on a parser for the format, add it to the crate rather than write a regex. A crate new to the workspace needs a person to approve it.
2. **A `str` method, when the input is flat text.** `starts_with`, `strip_prefix`, `ends_with`, `contains`, `split`, `replace`. Use these only when the input has no quoting or nesting.

A regex is the right tool for:

- **A flat format no parser covers**, such as a linter's `file:line:col` output prefix.
- **A search over unstructured text**, such as a linter's free-form messages.
- **A pattern supplied at runtime**, such as a plugin definition or a user's config.

Compile a constant pattern once, in a static, not on every call.

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

## GitHub Actions workflows

- Name each action by its full release tag, such as `actions/checkout@v5.0.0`; `.github/workflows/actions.lock` locks each one to a commit
- After adding a workflow or changing a `uses:` line, run `gh actions-lock` and commit `actions.lock` with the change
- Never edit `actions.lock` by hand
- Install the tool with `gh extension install github/gh-actions-lock --pin v0.4.1`, the version `verify-lockfile.yml` pins

## Pull requests and AI review

Keep a pull request in draft while you iterate, and mark it ready when it's done. Marking it ready, and every push
after that, starts a Claude Code Review, so batch fixes into one push.

Before marking a PR ready, run `/code-review` and fix or answer each 🔴 Important finding. The review rules, which
Claude Code Review on GitHub follows too, are in REVIEW.md:

@REVIEW.md

Never resolve a 🔴 thread you didn't fix. If you disagree with a finding, reply with your reasoning and ask a person to
review.
