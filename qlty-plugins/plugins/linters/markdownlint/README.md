# Markdownlint

[Markdownlint](https://github.com/davidanson/markdownlint) is a Node.js style checker and lint tool for Markdown/CommonMark files.

## Enabling Markdownlint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable markdownlint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
markdownlint = "latest"

# OR enable a specific version
[plugins.enabled]
markdownlint = "X.Y.Z"
```

## Auto-enabling

Markdownlint will be automatically enabled by `qlty init` if a `.markdownlint.json` configuration file is present.

## Configuration files

- [`.markdownlint.json`](https://github.com/DavidAnson/markdownlint?tab=readme-ov-file#config)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Markdownlint.

## Links

- [Markdownlint on GitHub](https://github.com/davidanson/markdownlint)
- [Markdownlint plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/markdownlint)
- [Markdownlint releases](https://github.com/DavidAnson/markdownlint/blob/main/CHANGELOG.md)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Markdownlint is licensed under the [MIT License](https://github.com/DavidAnson/markdownlint/blob/main/LICENSE).
