# Flake8

[Flake8](https://github.com/pycqa/flake8) is a wrapper around PyFlakes, pycodestyle and Ned Batchelder's McCabe script.
Flake8 runs all the tools by launching the single flake8 command. It displays the warnings in a per-file, merged output.

## Enabling Flake8

Enabling with the `qlty` CLI:

```bash
qlty plugins enable flake8
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
flake8 = "latest"

# OR enable a specific version
[plugins.enabled]
flake8 = "X.Y.Z"
```

## Auto-enabling

Flake8 will be automatically enabled by `qlty init` if a `.flake8` configuration file is present.

## Configuration files

- [`.flake8`](https://flake8.pycqa.org/en/latest/user/configuration.html#configuration-locations)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Flake8.

## Links

- [Flake8 on GitHub](https://github.com/pycqa/flake8)
- [Flake8 plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/flake8)
- [Flake8 releases](https://flake8.pycqa.org/en/latest/release-notes/index.html)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Flake8 is licensed under the [MIT license](https://github.com/PyCQA/flake8/blob/main/LICENSE).
