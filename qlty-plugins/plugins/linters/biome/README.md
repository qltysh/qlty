# Biome

[Biome](https://github.com/biomejs/biome) is a fast, all-in-one toolchain for web projects — linting, formatting, and import sorting in a single tool with no external dependencies.

## Enabling Biome

Enabling with the `qlty` CLI:

```bash
qlty plugins enable biome
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "biome"

# OR pin to a specific version
[[plugin]]
name = "biome"
version = "X.Y.Z"
```

## Auto-enabling

Biome will be automatically enabled by `qlty init` if a `biome.json` or `biome.jsonc` configuration file is present.

## Languages and file types

Biome analyzes: TypeScript (`.ts`, `.tsx`), JavaScript (`.js`, `.jsx`), JSON (`.json`, `.jsonc`), and CSS (`.css`).

## Configuration files

- [`biome.json`](https://biomejs.dev/reference/configuration/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Biome.

## Troubleshooting

**"Found an unknown key `<key>`" errors from biome.json and Biome exits.**
Your `biome.json` uses configuration keys (for example `includes`, `assist`, `domains`, or `tailwindDirectives`) that were introduced in a newer Biome release than the version Qlty has installed. Biome treats unknown keys as fatal configuration errors and refuses to run.
Check which version of Biome Qlty is using (`qlty plugins list`) and either pin your `biome.json` to the schema for that version, or upgrade the Biome plugin version in `qlty.toml` to match your project's `biome.json`.

**Biome does not lint CSS or JSON files.**
By default, Biome's CSS and JSON formatters/linters are disabled. You need to explicitly enable them in `biome.json`.
Add `"css": { "linter": { "enabled": true } }` and `"json": { "linter": { "enabled": true } }` to your `biome.json`.

## Links

- [Biome on GitHub](https://github.com/biomejs/biome)
- [Biome plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/biome)
- [Biome releases](https://github.com/biomejs/biome/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Biome is dual-licensed under the [MIT License](https://github.com/biomejs/biome/blob/main/LICENSE-MIT) and the [Apache License 2.0](https://github.com/biomejs/biome/blob/main/LICENSE-APACHE).
