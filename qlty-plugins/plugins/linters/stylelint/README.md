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
[plugins.enabled]
stylelint = "latest"

# OR enable a specific version
[plugins.enabled]
stylelint = "X.Y.Z"
```

## Auto-enabling

Stylelint will be automatically enabled by `qlty init` if a `.stylelintrc` configuration file is present.

## Configuration files

- [`.stylelintrc`](https://github.com/stylelint/stylelint/blob/main/docs/user-guide/configure.md)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Stylelint.

## Links

- [Stylelint on GitHub](https://github.com/stylelint/stylelint)
- [Stylelint plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/stylelint)
- [Stylelint releases](https://github.com/stylelint/stylelint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Stylelint is licensed under the [MIT License](https://github.com/stylelint/stylelint/blob/main/LICENSE).
