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

## Pull requests and AI review

Keep a pull request in draft while you iterate, and mark it ready when it's done. Marking it ready, and every push
after that, starts a Claude Code Review, so batch fixes into one push.

Before marking a PR ready, run `/code-review` and fix or answer each 🔴 Important finding. The review rules, which
Claude Code Review on GitHub follows too, are in REVIEW.md:

@REVIEW.md

Never resolve a 🔴 thread you didn't fix. If you disagree with a finding, reply with your reasoning and ask a person to
review.

### Decisions

Every pull request description has a Decisions section listing the consequential decisions in the
change: the ones a reviewer should weigh in on before merge. A decision is consequential when at
least one of these holds:

- It is hard to reverse after merge: persisted data or schema, a public API or webhook shape, a
  package or module boundary, a new dependency.
- It is a product choice the issue or specification didn't settle: a new default, an empty or
  error state, who sees a feature, a limit or charge, a status or comment format, behavior in an
  edge case. Name the line of the issue that leaves it open, or say there is no issue.
- It is an architectural or code design choice that later code will copy: where a responsibility
  lives, which module depends on which, a new abstraction or interface, how errors, retries, state
  or concurrency are handled.
- It trades off security, cost, performance or reliability.

Leave out choices a later PR can change cheaply, such as naming, local structure or test layout.
Leave out choices the codebase already settles, where you followed the existing pattern and nothing
else was realistic. Don't invent alternatives no one would pick. If no decision qualifies, write
`Decisions: none. Follows <file or pattern>.` If more than four qualify, say so: the PR is probably
doing too much, and the description should propose how to split it.

Open the section with one row per decision, so a reviewer sees the ratings first:

```markdown
|     | Decision   | Claude        |
| --- | ---------- | ------------- |
| D1  | Short name | Agree         |
| D2  | Short name | Lean disagree |
```

Then give each decision its own block:

```markdown
#### D1. Short name of the decision

**Decision.** What this PR does, in one or two sentences, with the file and line.

**Why it needs a reviewer.** Which of the reasons above applies, in one sentence.

**Options.** The chosen option first, then each realistic alternative, with enough room to judge it.

1. **Chosen option (this PR).** How it works.
   - For: one reason per line.
   - Against: one cost or risk per line.
2. **Alternative.** How it would work, and where the codebase already does it this way, if it does.
   - For: one reason per line.
   - Against: one cost or risk per line.

**Considered by the author.** Which options were tried or weighed before this PR, and what settled
it. "Not considered" is a valid answer.

**Why this option.** The reasons it beats each alternative, as a list, and what would change the
choice.

**Claude:** Agree ○ ○ ○ ● ○ Disagree: **Lean disagree**

**Why not Agree.** Required for any rating except Agree. Write it so a reviewer can act without
opening the code:

- The concern, in one sentence.
- The evidence: a file and line, a failing case, or a production fact.
- What would move the rating to Agree.
```

The five positions, left to right, are Agree, Lean agree, Torn, Lean disagree and Disagree.

The ratings come from a Claude run that did not write the change. Once the description is drafted,
run a fresh subagent that sees only the diff, the linked issue and the Decisions section, not the
conversation that wrote the code. It fills in the Claude column, each **Claude:** line and each
**Why not Agree**. The agent that wrote the code does not edit them. It may add one line,
**Author's response:**, under a rating.
