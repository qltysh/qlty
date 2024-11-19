# Bandit

[Bandit](https://github.com/pycqa/bandit) is a tool designed to find common security issues in Python code.

## Enabling Bandit

Enabling with the `qlty` CLI:

```bash
qlty plugins enable bandit
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
bandit = "latest"

# OR enable a specific version
[plugins.enabled]
bandit = "X.Y.Z"
```

## Auto-enabling

Bandit will be automatically enabled when python files are present.

## Configuration files

- [`.bandit`](https://bandit.readthedocs.io/en/latest/config.html)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Bandit.

## Links

- [Bandit on GitHub](https://github.com/pycqa/bandit)
- [Bandit plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/bandit)
- [Bandit releases](https://github.com/pycqa/bandit/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Black is licensed under the [Apache License 2.0](https://github.com/PyCQA/bandit/blob/main/LICENSE).
