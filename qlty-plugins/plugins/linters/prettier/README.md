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
[plugins.enabled]
prettier = "latest"

# OR enable a specific version
[plugins.enabled]
prettier = "X.Y.Z"
```

## Auto-enabling

Prettier will be automatically enabled by `qlty init` if a `.prettier.yaml` configuration file is present.

## Configuration files

- [`.prettierrc`](https://prettier.io/docs/en/configuration)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Prettier.

## Links

- [Prettier on GitHub](https://github.com/prettier/prettier)
- [Prettier plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/prettier)
- [Prettier releases](https://github.com/prettier/prettier/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Prettier is licensed under the [MIT License](https://github.com/prettier/prettier/blob/main/LICENSE).
