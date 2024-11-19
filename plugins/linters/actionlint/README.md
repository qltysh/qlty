# Actionlint

[Actionlint](https://github.com/rhysd/actionlint) is a static checker for GitHub Actions workflow files.

## Enabling Actionlint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable actionlint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
actionlint = "latest"

# OR enable a specific version
[plugins.enabled]
actionlint = "X.Y.Z"
```

## Auto-enabling

Actionlint will be automatically enabled by `qlty init` if a `.github/actionlint.yaml` configuration file is present.

## Configuration files

- [`.github/actionlint.yaml`](https://github.com/rhysd/actionlint/blob/main/docs/config.md#configuration-file)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Actionlint.

## Links

- [Actionlint on GitHub](https://github.com/rhysd/actionlint)
- [Actionlint plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/actionlint)
- [Actionlint releases](https://github.com/rhysd/actionlint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Actionlint is licensed under the [MIT License](https://github.com/rhysd/actionlint/blob/main/LICENSE.txt).
