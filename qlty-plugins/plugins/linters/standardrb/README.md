# Standardrb

[Standardrb](https://github.com/standardrb/standard) is a linter & formatter built on RuboCop and provides an unconfigurable configuration to all of RuboCop's built-in rules as well as those included in rubocop-performance. It also supports plugins built with lint_roller, like standard-rails.

## Enabling Standardrb

Enabling with the `qlty` CLI:

```bash
qlty plugins enable standardrb
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
standardrb = "latest"

# OR enable a specific version
[plugins.enabled]
standardrb = "X.Y.Z"
```

## Auto-enabling

Standardrb will be automatically enabled by `qlty init` if a `.standard.yml` configuration file is present.

## Configuration files

- [`.standard.yml`](https://github.com/standardrb/standard?tab=readme-ov-file#yaml-options)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Standardrb.

## Links

- [Standardrb on GitHub](https://github.com/standardrb/standard)
- [Standardrb plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/standardrb)
- [Standardrb releases](https://github.com/standardrb/standard/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Standardrb is licensed under [this License](https://github.com/standardrb/standard/blob/main/LICENSE.txt).
