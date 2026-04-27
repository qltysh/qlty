# Redocly

[Redocly CLI](https://github.com/Redocly/redocly-cli) is a linter and bundler for OpenAPI and Arazzo API descriptions that identifies structural problems, missing fields, and standards violations.

## Enabling Redocly

Enabling with the `qlty` CLI:

```bash
qlty plugins enable redocly
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "redocly"

# OR pin to a specific version
[[plugin]]
name = "redocly"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

Redocly will be automatically enabled by `qlty init` if a `redocly.yaml` configuration file is present.

## Languages and file types

Redocly analyzes: OpenAPI 2.x/3.x descriptions and Arazzo workflow files (`.yaml`, `.yml`, `.json`).

## Configuration files

- [`redocly.yaml`](https://redocly.com/docs/cli/configuration/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Redocly.

## Troubleshooting

**Redocly exits with "No config file found" on every run.**
Redocly requires a `.redocly.yaml` (or `redocly.yaml`) configuration file that lists the OpenAPI spec files to lint. Without it, Redocly cannot determine which files to check.
Create a `.redocly.yaml` in the project root (or in `.qlty/configs/`) with an `apis:` key pointing to your spec files.

**Redocly reports "spec file not found" even though the file exists.**
Redocly resolves spec file paths relative to the config file's location. If the config is in `.qlty/configs/` and the spec file is in a different directory, the relative path in the config may be wrong.
Use an absolute path or ensure the relative path in `.redocly.yaml` is correct relative to where the config file is located.

## Links

- [Redocly CLI on GitHub](https://github.com/Redocly/redocly-cli)
- [Redocly plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/redocly)
- [Redocly CLI releases](https://github.com/Redocly/redocly-cli/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Redocly CLI is licensed under the [MIT License](https://github.com/Redocly/redocly-cli/blob/main/LICENSE).
