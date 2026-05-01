# ESLint

[ESLint](https://github.com/eslint/eslint) tool for identifying and reporting on patterns found in ECMAScript/JavaScript code.

## Enabling ESLint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable eslint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "eslint"

# OR pin to a specific version
[[plugin]]
name = "eslint"
version = "X.Y.Z"
```

## Auto-enabling

ESLint will be automatically enabled by `qlty init` if a `.eslintrc` configuration file is present.

## Configuration files

- [`.eslintrc`](https://eslint.org/docs/latest/use/configure/configuration-files)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running ESLint.

## Languages and file types

ESLint analyzes: JavaScript (`.js`, `.jsx`, `.mjs`, `.cjs`) and TypeScript (`.ts`, `.tsx`).

## Troubleshooting

**"ESLINT_LEGACY_ECMAFEATURES DeprecationWarning" appears in logs.**
Your `.eslintrc` uses the deprecated `ecmaFeatures` property, which has no effect in recent ESLint versions.
Remove `ecmaFeatures` from your ESLint config, or migrate to the flat config format (`eslint.config.js`).

**ESLint is not using your project config.**
Config files referenced in `.qlty/configs/` that do not actually exist will be silently skipped. ESLint may fall back to a different config or run without rules.
Ensure your ESLint config is present in `.qlty/configs/` under the exact filename ESLint expects.

## Links

- [ESLint on GitHub](https://github.com/eslint/eslint)
- [ESLint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/eslint)
- [ESLint releases](https://github.com/eslint/eslint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

ESLint is licensed under the [MIT license](https://github.com/eslint/eslint/blob/main/LICENSE).
