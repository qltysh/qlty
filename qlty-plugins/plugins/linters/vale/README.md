# Vale

[Vale](https://github.com/errata-ai/vale) is a command-line prose linter that enforces editorial style guides in documentation, comments, and any plain-text file.

## Enabling Vale

Enabling with the `qlty` CLI:

```bash
qlty plugins enable vale
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "vale"

# OR pin to a specific version
[[plugin]]
name = "vale"
version = "X.Y.Z"
```

## Auto-enabling

Vale will be automatically enabled by `qlty init` if a `.vale.ini` configuration file is present.

Vale will be automatically enabled by `qlty init` if a `.vale.ini` configuration file is present.

## Languages and file types

Vale analyzes all text-bearing file types including Markdown, reStructuredText, AsciiDoc, HTML, and plain text.

## Configuration files

- [`.vale.ini`](https://vale.sh/docs/vale-cli/structure/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Vale.

## Troubleshooting

**"E100 [.vale.ini not found] Runtime error" when running vale.**
Vale requires a `.vale.ini` configuration file to define which styles and vocabularies to use. Without it, vale exits with code 2 and refuses to run.
Create a `.vale.ini` in your project root (or in `.qlty/configs/`) and run `vale sync` to download the configured styles before your first check.

**Vale reports no issues even though prose problems exist.**
Vale only checks files against rules defined in the active styles. A freshly created `.vale.ini` with no packages configured will produce no output.
Run `vale sync` after adding packages to `.vale.ini` to download style files into the `StylesPath` directory.

## Links

- [Vale on GitHub](https://github.com/errata-ai/vale)
- [Vale plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/vale)
- [Vale releases](https://github.com/errata-ai/vale/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Vale is licensed under the [MIT License](https://github.com/errata-ai/vale/blob/v3/LICENSE).
