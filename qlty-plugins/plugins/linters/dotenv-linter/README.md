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
[[plugin]]
name = "dotenv-linter"

# OR pin to a specific version
[[plugin]]
name = "dotenv-linter"
version = "X.Y.Z"
```

## Languages and file types

Dotenv-linter analyzes: `.env` files and files matching the `.env.*` naming pattern.

## Troubleshooting

**dotenv-linter reports no issues on a `.env` file that has obvious problems.**
dotenv-linter only checks files named `.env`, `.env.example`, `.env.test`, and similar standard variants. Non-standard names like `.env.local.bak` may be skipped.
Verify that your `.env` file has a name that dotenv-linter recognises, and that it is not in a directory covered by `exclude_patterns`.

**dotenv-linter reports "Leading space detected" on a commented-out line.**
dotenv-linter is strict about leading spaces. Lines that start with `# ` followed by indented comments may trigger this.
Remove any leading spaces before comment characters in the `.env` file.

## Links

- [Dotenv-linter on GitHub](https://github.com/dotenv-linter/dotenv-linter)
- [Dotenv-linter plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/dotenv-linter)
- [Dotenv-linter releases](https://github.com/dotenv-linter/dotenv-linter/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Dotenv-linter is licensed under the [MIT License](https://github.com/dotenv-linter/dotenv-linter/blob/master/LICENSE).
