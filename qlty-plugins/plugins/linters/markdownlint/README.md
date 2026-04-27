# Markdownlint

[Markdownlint](https://github.com/davidanson/markdownlint) is a Node.js style checker and lint tool for Markdown/CommonMark files.

## Enabling Markdownlint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable markdownlint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "markdownlint"

# OR pin to a specific version
[[plugin]]
name = "markdownlint"
version = "X.Y.Z"
```

## Auto-enabling

Markdownlint will be automatically enabled by `qlty init` if a `.markdownlint.json` configuration file is present.

## Configuration files

- [`.markdownlint.json`](https://github.com/DavidAnson/markdownlint?tab=readme-ov-file#config)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Markdownlint.

## Languages and file types

Markdownlint analyzes: Markdown (`.md`, `.markdown`).

## Troubleshooting

**markdownlint reports "MD013 line length" errors on files with long lines.**
The default `MD013` rule flags lines longer than 80 characters. Many documentation files exceed this in code blocks or URLs.
Add a `.markdownlint.json` or `.markdownlint.yaml` config file with `"MD013": false` to disable line-length checks, or configure `line_length` to a higher value.

**markdownlint produces no output on a file with obvious style violations.**
If a `.markdownlint.json` configuration exists in the project that disables most rules, or if the file is excluded by `exclude_patterns`, markdownlint will produce no output.
Verify the active config with `markdownlint --print-config .` and check that the rules covering your expected violations are not disabled.

## Links

- [Markdownlint on GitHub](https://github.com/davidanson/markdownlint)
- [Markdownlint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/markdownlint)
- [Markdownlint releases](https://github.com/DavidAnson/markdownlint/blob/main/CHANGELOG.md)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Markdownlint is licensed under the [MIT License](https://github.com/DavidAnson/markdownlint/blob/main/LICENSE).
