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
[plugins.enabled]
ruff = "latest"

# OR enable a specific version
[plugins.enabled]
ruff = "X.Y.Z"
```

## Auto-enabling

Ruff will be automatically enabled by `qlty init` if a `ruff.toml` configuration file is present.

## Configuration files

- [`ruff.toml`](https://github.com/astral-sh/ruff?tab=readme-ov-file#configuration)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Ruff.

## Links

- [Ruff on GitHub](https://github.com/astral-sh/ruff)
- [Ruff plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/ruff)
- [Ruff releases](https://github.com/astral-sh/ruff/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Ruff is licensed under the [MIT License](https://github.com/astral-sh/ruff/blob/main/LICENSE).
