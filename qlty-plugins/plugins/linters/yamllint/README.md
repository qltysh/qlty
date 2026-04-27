# yamllint

[yamllint](https://github.com/adrienverge/yamllint) is a linter for YAML files that checks for syntax errors, key duplication, line length, trailing spaces, and indentation issues.

## Enabling yamllint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable yamllint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "yamllint"

# OR pin to a specific version
[[plugin]]
name = "yamllint"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

yamllint will be automatically enabled by `qlty init` if a `.yamllint` or `.yamllint.yml` configuration file is present.

## Languages and file types

yamllint analyzes: YAML (`.yml`, `.yaml`).

## Configuration files

- [`.yamllint`](https://yamllint.readthedocs.io/en/stable/configuration.html)

Accepted filenames: `.yamllint`, `.yamllint.yml`, `.yamllint.yaml`.

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running yamllint.

## Troubleshooting

**Issues are not shown for certain YAML files.**
Qlty drops all issues from a file when yamllint produces more than 500 results for it. VCR cassette fixture files (`test/fixtures/vcr_cassettes/**`) routinely contain thousands of lines and trigger this limit.
Add VCR cassette and other bulk fixture paths to `exclude_patterns` in `qlty.toml`.

**"invalid config: level should be 'error' or 'warning'" error.**
Your `.yamllint` config uses an unrecognized severity level value.
Check your `.yamllint` config and ensure all `level:` values are exactly `error` or `warning` (lowercase).

## Links

- [yamllint on GitHub](https://github.com/adrienverge/yamllint)
- [yamllint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/yamllint)
- [yamllint releases](https://github.com/adrienverge/yamllint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

yamllint is licensed under the [GNU General Public License v3.0](https://github.com/adrienverge/yamllint/blob/master/LICENSE).
