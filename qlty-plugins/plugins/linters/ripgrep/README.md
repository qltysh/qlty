# ripgrep

[ripgrep](https://github.com/BurntSushi/ripgrep) is a fast, line-oriented search tool. Under Qlty, it is used to surface code annotations such as `TODO`, `FIXME`, `HACK`, and `BUG` across your codebase.

## Enabling ripgrep

Enabling with the `qlty` CLI:

```bash
qlty plugins enable ripgrep
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "ripgrep"

# OR pin to a specific version
[[plugin]]
name = "ripgrep"
version = "X.Y.Z"
```

## Languages and file types

ripgrep searches across all common source files: JavaScript, TypeScript, Python, Ruby, Go, Rust, Java, Kotlin, PHP, Swift, CSS/SCSS, and shell scripts.

The full list of matched extensions is defined in the plugin definition.

## Configuration files

- [`.ripgreprc`](https://github.com/BurntSushi/ripgrep/blob/master/GUIDE.md#configuration-file)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running ripgrep.

## Troubleshooting

**ripgrep does not find TODOs in a language that you know has annotations.**
ripgrep's Qlty plugin scans a fixed list of file extensions. If your language's extension is not in the list (for example `.liquid`, `.erb`, `.haml`), the files will be skipped.
Add a `file_types` override in `qlty.toml` under `[plugins.definitions.ripgrep]` to include the extra extensions.

**ripgrep floods results with annotations from vendored or generated code.**
Large auto-generated files and vendored code often contain many `TODO` comments, which will all be reported as issues.
Add the generated and vendored directories to `exclude_patterns` in `qlty.toml` to skip them.

## Links

- [ripgrep on GitHub](https://github.com/BurntSushi/ripgrep)
- [ripgrep plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/ripgrep)
- [ripgrep releases](https://github.com/BurntSushi/ripgrep/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

ripgrep is licensed under the [MIT License](https://github.com/BurntSushi/ripgrep/blob/master/LICENSE-MIT).
