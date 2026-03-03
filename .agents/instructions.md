## Qlty (code quality)

Qlty is a code quality tool for linting, auto-formatting, and code smell detection.
Full documentation: https://docs.qlty.sh

### Setup

Install: `curl https://qlty.sh | sh`

Run all commands from the git repository root.

Initialize in a repo (auto-detects languages and enables relevant plugins):
```
qlty init --no
```

The `--no` flag skips all interactive prompts. This creates a `.qlty/` directory with `qlty.toml` and a `.gitignore`. If already initialized, this command will error — that's fine, the config already exists.

### Key commands

```
qlty fmt                          # Auto-format changed files
qlty fmt --all                    # Auto-format all files
qlty check                        # Lint changed files
qlty check --all                  # Lint all files
qlty check --fix                  # Lint and auto-fix changed files
qlty check --fix --all            # Lint and auto-fix all files
qlty check --filter=eslint        # Lint with a specific plugin only
qlty smells --all                 # Find code smells (complexity, duplication)
qlty metrics --all --max-depth=2  # Show quality metrics summary
qlty plugins list                 # List available plugins
qlty plugins enable <name>        # Enable a plugin
```

### Workflow

- Before committing, run `qlty fmt` to auto-format
- Before finishing, run `qlty check --fix --level=low` and fix any remaining lint errors
- Use `qlty smells` to identify complex or duplicated code

### Configuration

Config file location: `.qlty/qlty.toml` (inside the `.qlty/` directory, NOT the repo root)

Minimal valid config:
```toml
config_version = "0"

exclude_patterns = ["**/node_modules/**"]

[[source]]
name = "default"
default = true
```

Add plugins with `qlty plugins enable <name>` or by adding to the config:
```toml
[[plugin]]
name = "eslint"
```

### Common mistakes to avoid

- Run all `qlty` commands from the git repository root
- Do NOT create `qlty.toml` in the repo root. It must be at `.qlty/qlty.toml`
- Do NOT run `qlty check` or `qlty fmt` without initializing first (`qlty init`)
- Do NOT manually enable plugins that `qlty init` already auto-detected — check `qlty plugins list` first
- Do NOT use `[sources.name]` syntax — the correct syntax is `[[source]]` with `name = "..."` inside
- The `config_version` field is required and its value must be the string `"0"`
- `exclude_patterns` only affects linting/formatting, NOT code coverage

### Reference

- Config reference: https://docs.qlty.sh/qlty-toml
- Available plugins: https://docs.qlty.sh/plugins
- Analysis configuration: https://docs.qlty.sh/analysis-configuration
- Full docs (LLM-optimized): https://docs.qlty.sh/llms-full.txt
