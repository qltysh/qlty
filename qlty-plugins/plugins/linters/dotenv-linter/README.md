# Dotenv-linter

[Dotenv-linter](https://github.com/dotenv-linter/dotenv-linter) is a linter and formatter for `.env` files.

## Enabling Dotenv-linter

Enabling with the `qlty` CLI:

```bash
qlty plugins enable dotenv-linter
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
dotenv-linter = "latest"

# OR enable a specific version
[plugins.enabled]
dotenv-linter = "X.Y.Z"
```

## Links

- [Dotenv-linter on GitHub](https://github.com/dotenv-linter/dotenv-linter)
- [Dotenv-linter plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/dotenv-linter)
- [Dotenv-linter releases](https://github.com/dotenv-linter/dotenv-linter/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Dotenv-linter is licensed under the [MIT License](https://github.com/dotenv-linter/dotenv-linter/blob/master/LICENSE).
