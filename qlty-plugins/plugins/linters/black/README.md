# Black

[Black](https://github.com/psf/black) is a Python code formatter.

## Enabling Black

Enabling with the `qlty` CLI:

```bash
qlty plugins enable black
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
black = "latest"

# OR enable a specific version
[plugins.enabled]
black = "X.Y.Z"
```

## Links

- [Black on GitHub](https://github.com/psf/black)
- [Black plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/black)
- [Black releases](https://github.com/psf/black/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Black is licensed under the [The MIT License](https://github.com/psf/black/blob/main/LICENSE).
