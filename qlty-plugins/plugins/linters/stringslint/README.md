# StringsLint

[StringsLint](https://github.com/dral3x/StringsLint) is a tool for iOS/macOS projects that ensures localized strings are complete and never unused across `.strings` files and source code.

## Enabling StringsLint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable stringslint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "stringslint"

# OR pin to a specific version
[[plugin]]
name = "stringslint"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

StringsLint will be automatically enabled by `qlty init` if a `.stringslint` or `.stringslint.yml` configuration file is present.

## Languages and file types

StringsLint analyzes: Swift (`.swift`), Objective-C (`.m`, `.h`), `.strings` files, Interface Builder files (`.xib`, `.storyboard`).

## Configuration files

- [`.stringslint.yml`](https://github.com/dral3x/StringsLint#configuration)

Accepted filenames: `.stringslint`, `.stringslint.yml`, `.stringslint.yaml`.

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running StringsLint.

## Platform support

StringsLint is only available on **macOS**.

## Troubleshooting

**StringsLint reports no issues on a project with hardcoded strings.**
StringsLint checks that all user-facing string literals are localised via `NSLocalizedString` or a similar macro. If the project does not use localisation at all, StringsLint may not have any `.strings` files to compare against.
Ensure `.strings` localisation files exist in the project before enabling StringsLint. If localisation is intentionally not used, disable StringsLint in `qlty.toml`.

**StringsLint reports a string as "unused" but it is used via a dynamic key.**
StringsLint performs static analysis and cannot follow dynamic key construction (for example `"key_\(variable)"`). Strings referenced through string interpolation will appear unused.
Add the dynamically-referenced key patterns to the StringsLint ignore list in `.stringslint.yml`.

## Links

- [StringsLint on GitHub](https://github.com/dral3x/StringsLint)
- [StringsLint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/stringslint)
- [StringsLint releases](https://github.com/dral3x/StringsLint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

StringsLint is licensed under the [MIT License](https://github.com/dral3x/StringsLint/blob/master/LICENSE).
