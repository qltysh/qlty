# Rubocop

[Rubocop](https://github.com/rubocop/rubocop) is a Ruby static code analyzer (a.k.a. linter) and code formatter.

## Enabling Rubocop

Enabling with the `qlty` CLI:

```bash
qlty plugins enable rubocop
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
rubocop = "latest"

# OR enable a specific version
[plugins.enabled]
rubocop = "X.Y.Z"
```

## Auto-enabling

Rubocop will be automatically enabled by `qlty init` if a `.rubocop.yml` configuration file is present.

## Configuration files

- [`.rubocop.yml`](https://docs.rubocop.org/rubocop/1.63/configuration.html)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Rubocop.

## Links

- [Rubocop on GitHub](https://github.com/rubocop/rubocop)
- [Rubocop plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/rubocop)
- [Rubocop releases](https://github.com/rubocop/rubocop/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Rubocop is licensed under the [MIT License](https://github.com/rubocop/rubocop/blob/master/LICENSE.txt).
