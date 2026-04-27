# Brakeman

[Brakeman](https://github.com/presidentbeef/brakeman) is a static analysis tool that checks Ruby on Rails applications for security vulnerabilities.

## Enabling Brakeman

Enabling with the `qlty` CLI:

```bash
qlty plugins enable brakeman
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "brakeman"

# OR pin to a specific version
[[plugin]]
name = "brakeman"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

Brakeman will be automatically enabled by `qlty init` when Ruby files are present in a Rails application structure.

## Languages and file types

Brakeman analyzes Ruby (`.rb`) files in Rails applications. It requires an `app/` directory in the scanned project to identify Rails-specific patterns.

## Configuration files

- [`brakeman.ignore`](https://brakemanscanner.org/docs/ignoring_false_positives/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Brakeman.

## Troubleshooting

**Brakeman reports no issues even though the project contains Rails code.**
Brakeman requires an `app/` directory at the scan root to identify a Rails application. If your Rails app lives in a subdirectory, Qlty may not be targeting the right location.
Verify that the build log shows Brakeman scanning the Rails app root, and confirm your project structure matches what Brakeman expects.

**`brakeman.ignore` is not being applied.**
A `brakeman.ignore` file located inside `config/` or another directory that matches `exclude_patterns` will be silently skipped.
Move `brakeman.ignore` to `.qlty/configs/` so Qlty can stage it for the run.

## Links

- [Brakeman on GitHub](https://github.com/presidentbeef/brakeman)
- [Brakeman plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/brakeman)
- [Brakeman releases](https://github.com/presidentbeef/brakeman/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Brakeman is licensed under the [MIT License](https://github.com/presidentbeef/brakeman/blob/main/MIT-LICENSE).
