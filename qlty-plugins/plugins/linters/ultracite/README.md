# Ultracite

[Ultracite](https://github.com/haydenbleasel/ultracite) is a highly opinionated, zero-configuration linter and formatter for JavaScript and TypeScript projects. It's built on top of Biome, providing instant code analysis with hundreds of preconfigured rules.

## Enabling Ultracite

Enabling with the `qlty` CLI:

```bash
qlty plugins enable ultracite
```

Or by editing `qlty.toml`:

```toml
[plugins.enabled]
ultracite = "latest"
```

## Configuration files

- [`biome.json`](https://biomejs.dev/reference/configuration/)
- [`biome.jsonc`](https://biomejs.dev/reference/configuration/)
- [`eslint.config.js`](https://eslint.org/docs/latest/use/configure/configuration-files)
- [`eslint.config.mjs`](https://eslint.org/docs/latest/use/configure/configuration-files)
- [`eslint.config.ts`](https://eslint.org/docs/latest/use/configure/configuration-files)

## Links

- [Ultracite on GitHub](https://github.com/haydenbleasel/ultracite)
- [Ultracite documentation](https://docs.ultracite.ai/)
- [Ultracite plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/ultracite)

## License

Ultracite is licensed under the [MIT License](https://github.com/haydenbleasel/ultracite/blob/main/license.md).
