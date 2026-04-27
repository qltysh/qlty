# SwiftFormat

[SwiftFormat](https://github.com/nicklockwood/SwiftFormat) is a code formatting tool for Swift that rewrites source files to apply a consistent style.

## Enabling SwiftFormat

Enabling with the `qlty` CLI:

```bash
qlty plugins enable swiftformat
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "swiftformat"

# OR pin to a specific version
[[plugin]]
name = "swiftformat"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

SwiftFormat will be automatically enabled by `qlty init` if a `.swiftformat` configuration file is present.

## Languages and file types

SwiftFormat formats: Swift (`.swift`).

## Configuration files

- [`.swiftformat`](https://github.com/nicklockwood/SwiftFormat#options)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running SwiftFormat.

## Platform support

SwiftFormat is only available on **macOS**.

## Troubleshooting

**SwiftFormat reports formatting differences on Swift code that looks correct.**
SwiftFormat enforces a specific code style. Even minor differences like trailing newlines, brace placement, or spacing around operators will be reported.
Run `qlty fmt` to apply SwiftFormat's changes, then review and commit the result. Use a `.swiftformat` configuration file to disable specific rules if the default style does not fit your project.

**SwiftFormat does not run on Swift files in a subdirectory.**
SwiftFormat runs from the directory containing `Package.swift` or the project root. If Swift source files are in a deeply nested subdirectory without a `Package.swift` in an ancestor, SwiftFormat may skip them.
Ensure the `Package.swift` is in the repository root or that SwiftFormat's `--path` argument is configured to include the correct source directory.

## Links

- [SwiftFormat on GitHub](https://github.com/nicklockwood/SwiftFormat)
- [SwiftFormat plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/swiftformat)
- [SwiftFormat releases](https://github.com/nicklockwood/SwiftFormat/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

SwiftFormat is licensed under the [MIT License](https://github.com/nicklockwood/SwiftFormat/blob/main/LICENSE.md).
