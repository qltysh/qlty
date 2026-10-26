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
[[plugin]]
name = "flake8"

# OR pin to a specific version
[[plugin]]
name = "flake8"
version = "X.Y.Z"
```

## Auto-enabling

Flake8 will be automatically enabled by `qlty init` if a `.flake8` configuration file is present.

## Configuration files

- [`.flake8`](https://flake8.pycqa.org/en/latest/user/configuration.html#configuration-locations)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Flake8.

## Languages and file types

Flake8 analyzes: Python (`.py`).

## Troubleshooting

**flake8 conflicts with black on formatting issues like `E501` (line too long) or `W503`/`W504`.**
Black reformats code in a way that can still trigger some flake8 style rules, leading to issues that can never be fixed without disabling the rule.
Add a `[flake8]` section to `setup.cfg` or a `.flake8` file with `extend-ignore = E501, W503` (or the relevant rules) to suppress the rules that conflict with black's output.

**flake8 does not pick up plugins installed in the project's virtualenv.**
Qlty runs flake8 with its own managed Python environment, not the project's virtualenv. flake8 plugins like `flake8-bugbear` or `flake8-comprehensions` installed in the project venv are not available.
Add the plugin package name to the `additional_dependencies` list in the plugin configuration if you need third-party flake8 plugins to run.

## Links

- [Flake8 on GitHub](https://github.com/pycqa/flake8)
- [Flake8 plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/flake8)
- [Flake8 releases](https://flake8.pycqa.org/en/latest/release-notes/index.html)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Flake8 is licensed under the [MIT license](https://github.com/PyCQA/flake8/blob/main/LICENSE).
