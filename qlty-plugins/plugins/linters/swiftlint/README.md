# SwiftLint

[SwiftLint](https://github.com/realm/SwiftLint) is a tool for enforcing Swift style and conventions, based on the Ray Wenderlich Swift Style Guide.

## Enabling SwiftLint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable swiftlint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "swiftlint"

# OR pin to a specific version
[[plugin]]
name = "swiftlint"
version = "X.Y.Z"
```

## Languages and file types

SwiftLint analyzes: Swift (`.swift`).

## Configuration files

- [`.swiftlint.yml`](https://github.com/realm/SwiftLint#configuration)

Accepted filenames: `.swiftlint.yml`, `.swiftlint.yaml`, `.swiftlint`.

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running SwiftLint.

## Troubleshooting

**SwiftLint reports no issues on Swift files that have obvious violations.**
SwiftLint uses its default rule set if no `.swiftlint.yml` is present. Some rules are disabled by default.
Create a `.swiftlint.yml` that explicitly lists the rules you want under `opt_in_rules:`, or set `only_rules:` to limit to a specific set.

**SwiftLint exits with "Linting Swift files in current working directory" but produces no output.**
SwiftLint finds no `.swift` files to lint. This happens when the project root does not contain Swift files and SwiftLint is not given a path.
Verify that the `.swift` files are not in a directory covered by `exclude_patterns`, and that the `exclude_patterns` in `qlty.toml` do not inadvertently exclude the Swift source directory.

## Links

- [SwiftLint on GitHub](https://github.com/realm/SwiftLint)
- [SwiftLint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/swiftlint)
- [SwiftLint releases](https://github.com/realm/SwiftLint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

SwiftLint is licensed under the [MIT License](https://github.com/realm/SwiftLint/blob/main/LICENSE).
