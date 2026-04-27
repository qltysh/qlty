# Checkstyle

[Checkstyle](https://github.com/checkstyle/checkstyle) is a Java code quality tool that enforces coding standards by checking Java source code against a configurable set of rules.

## Enabling Checkstyle

Enabling with the `qlty` CLI:

```bash
qlty plugins enable checkstyle
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "checkstyle"

# OR pin to a specific version
[[plugin]]
name = "checkstyle"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

Checkstyle will be automatically enabled by `qlty init` if a `checkstyle.xml` configuration file is present.

## Languages and file types

Checkstyle analyzes: Java (`.java`).

## Configuration files

- [`checkstyle.xml`](https://checkstyle.sourceforge.io/config.html)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Checkstyle.

## Troubleshooting

**"Could not find config XML file" error and Checkstyle exits.**
Checkstyle requires exactly one `checkstyle.xml` configuration file and it must be accessible at run time. If the file path is wrong, or if the file is in a directory that Qlty stages separately, Checkstyle cannot locate it.
Place `checkstyle.xml` in `.qlty/configs/` so Qlty can stage it alongside the source files, or ensure the path in the plugin's `config_files` matches the actual file location.

**"has more than one config file, but only one is supported" error.**
When multiple `checkstyle.xml` files exist in the project (for example one in the root and one in a subdirectory), Qlty may pass both paths to Checkstyle. Checkstyle only accepts a single configuration file.
Consolidate to a single `checkstyle.xml` at the project root or in `.qlty/configs/`, and remove or rename the duplicate.

## Links

- [Checkstyle on GitHub](https://github.com/checkstyle/checkstyle)
- [Checkstyle plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/checkstyle)
- [Checkstyle releases](https://github.com/checkstyle/checkstyle/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Checkstyle is licensed under the [GNU Lesser General Public License v2.1](https://github.com/checkstyle/checkstyle/blob/master/LICENSE).
