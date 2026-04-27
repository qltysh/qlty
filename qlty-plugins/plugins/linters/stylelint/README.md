# Stylelint

[Stylelint](https://github.com/stylelint/stylelint) is a CSS linter that helps you avoid errors and enforce conventions.

## Enabling Stylelint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable stylelint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "stylelint"

# OR pin to a specific version
[[plugin]]
name = "stylelint"
version = "X.Y.Z"
```

## Auto-enabling

Stylelint will be automatically enabled by `qlty init` if a `.stylelintrc` configuration file is present.

## Configuration files

- [`.stylelintrc`](https://github.com/stylelint/stylelint/blob/main/docs/user-guide/configure.md)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Stylelint.

## Languages and file types

Stylelint analyzes: CSS (`.css`), SCSS (`.scss`), and Sass (`.sass`).

## Troubleshooting

**Issues are not shown for certain CSS files.**
Qlty drops all issues from a file when Stylelint produces more than 500 results for it. Large or generated CSS files (e.g. marketing pages, vendor bundles) commonly trigger this.
Add generated CSS paths (e.g. `public/styles/**`) to `exclude_patterns` in `qlty.toml` if those files are not meant to be linted.

## Links

- [Stylelint on GitHub](https://github.com/stylelint/stylelint)
- [Stylelint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/stylelint)
- [Stylelint releases](https://github.com/stylelint/stylelint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Stylelint is licensed under the [MIT License](https://github.com/stylelint/stylelint/blob/main/LICENSE).
