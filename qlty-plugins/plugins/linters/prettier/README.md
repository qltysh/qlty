# Prettier

[Prettier](https://github.com/prettier/prettier) is an opinionated code formatter. It enforces a consistent style by parsing your code and re-printing it with its own rules that take the maximum line length into account, wrapping code when necessary.

## Enabling Prettier

Enabling with the `qlty` CLI:

```bash
qlty plugins enable prettier
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "prettier"

# OR pin to a specific version
[[plugin]]
name = "prettier"
version = "X.Y.Z"
```

## Auto-enabling

Prettier will be automatically enabled by `qlty init` if a `.prettier.yaml` configuration file is present.

## Configuration files

- [`.prettierrc`](https://prettier.io/docs/en/configuration)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Prettier.

## Languages and file types

Prettier formats: JavaScript, TypeScript, HTML, CSS/SCSS, JSON, Markdown, YAML, GraphQL, and more — see the [full language list](https://prettier.io/docs/en/index.html).

## Troubleshooting

**Prettier fails with "SyntaxError: Map keys must be unique" or "Unexpected token" on certain files.**
Prettier is attempting to format files with invalid syntax — duplicate YAML keys, malformed JSON, or non-standard YAML used in test fixtures (VCR cassettes, seed data) are common causes.
Add those file patterns to `exclude_patterns` in `qlty.toml` (e.g. `test/fixtures/**`, `spec/fixtures/**`, `lib/data/**`).

**Prettier is running with default settings instead of your project config.**
Prettier searches for many config variants (`.prettierrc`, `.prettierrc.js`, `.prettierrc.json`, `prettier.config.js`, etc.). If the file is not present in `.qlty/configs/`, Qlty falls back to defaults.
Copy your Prettier config to `.qlty/configs/` using the same filename.

## Links

- [Prettier on GitHub](https://github.com/prettier/prettier)
- [Prettier plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/prettier)
- [Prettier releases](https://github.com/prettier/prettier/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Prettier is licensed under the [MIT License](https://github.com/prettier/prettier/blob/main/LICENSE).
