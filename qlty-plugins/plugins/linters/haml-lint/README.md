# haml-lint

[haml-lint](https://github.com/sds/haml-lint) is a tool to help keep HAML files clean and readable by enforcing a set of configurable lint rules.

## Enabling haml-lint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable haml-lint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "haml-lint"

# OR pin to a specific version
[[plugin]]
name = "haml-lint"
version = "X.Y.Z"
```

## Auto-enabling

haml-lint will be automatically enabled by `qlty init` if a `.haml-lint.yml` configuration file is present.

haml-lint will be automatically enabled by `qlty init` if a `.haml-lint.yml` configuration file is present.

## Languages and file types

haml-lint analyzes: HAML (`.haml`).

## Configuration files

- [`.haml-lint.yml`](https://github.com/sds/haml-lint#configuration)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running haml-lint.

## Troubleshooting

**"Error installing ruby" with "The dependencies below are missing" on macOS.**
Qlty manages its own Ruby installation for Ruby-based tools. If `libyaml` is not present on the system, the Ruby build fails before haml-lint can run.
Install the missing dependency with `brew install libyaml` and re-run `qlty check`.

**haml-lint reports no issues on files that contain obvious lint violations.**
haml-lint disables some linters by default (for example `RuboCop`). Check your `.haml-lint.yml` to verify the desired linters are enabled, or create one from the [default config](https://github.com/sds/haml-lint/blob/main/config/default.yml).

## Links

- [haml-lint on GitHub](https://github.com/sds/haml-lint)
- [haml-lint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/haml-lint)
- [haml-lint releases](https://github.com/sds/haml-lint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

haml-lint is licensed under the [MIT License](https://github.com/sds/haml-lint/blob/main/LICENSE.md).
