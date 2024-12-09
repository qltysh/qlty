# Hadolint

[Hadolint](https://github.com/hadolint/hadolint) is a Dockerfile linter written in Haskell.

## Enabling Hadolint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable hadolint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
hadolint = "latest"

# OR enable a specific version
[plugins.enabled]
hadolint = "X.Y.Z"
```

## Auto-enabling

Hadolint will be automatically enabled by `qlty init` if a `.hadolint.yaml` configuration file is present.

## Configuration files

- [`.hadolint.yaml`](https://github.com/hadolint/hadolint?tab=readme-ov-file#configure)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Hadolint.

## Links

- [Hadolint on GitHub](https://github.com/hadolint/hadolint)
- [Hadolint plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/hadolint)
- [Hadolint releases](https://github.com/hadolint/hadolint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Hadolint is licensed under the [GNU General Public License v3.0](https://github.com/hadolint/hadolint/blob/master/LICENSE).
