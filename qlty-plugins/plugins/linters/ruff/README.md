# Ruff

[Ruff](https://github.com/astral-sh/ruff) is an extremely fast Python linter and code formatter, written in Rust.

## Enabling Ruff

Enabling with the `qlty` CLI:

```bash
qlty plugins enable ruff
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "ruff"

# OR pin to a specific version
[[plugin]]
name = "ruff"
version = "X.Y.Z"
```

## Auto-enabling

Ruff will be automatically enabled by `qlty init` if a `ruff.toml` configuration file is present.

## Configuration files

- [`ruff.toml`](https://github.com/astral-sh/ruff?tab=readme-ov-file#configuration)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Ruff.

## Languages and file types

Ruff analyzes: Python (`.py`, `.pyi`).

## Troubleshooting

**ruff reports no issues on a Python file that has obvious style problems.**
If a `pyproject.toml` or `ruff.toml` in the project root disables the relevant rules, or if the file is listed under `exclude` in the ruff configuration, ruff will produce no output.
Check your ruff configuration with `ruff check --show-settings` to see which rules are active and which paths are excluded.

**ruff conflicts with black on line length.**
Ruff's default line length (88 characters, matching Black) may differ from a pre-existing `flake8` config that uses a different `max-line-length`. This causes ruff to report lines that flake8 would accept.
Set `line-length = <N>` in the `[tool.ruff]` section of `pyproject.toml` to match your project's configured length.

## Links

- [Ruff on GitHub](https://github.com/astral-sh/ruff)
- [Ruff plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/ruff)
- [Ruff releases](https://github.com/astral-sh/ruff/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Ruff is licensed under the [MIT License](https://github.com/astral-sh/ruff/blob/main/LICENSE).
