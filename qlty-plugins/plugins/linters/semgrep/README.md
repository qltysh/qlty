# Semgrep

[Semgrep](https://github.com/semgrep/semgrep) is a fast, open-source, static analysis tool for searching code, finding bugs, and enforcing code standards at editor, commit, and CI time.

## Enabling Semgrep

Enabling with the `qlty` CLI:

```bash
qlty plugins enable semgrep
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "semgrep"

# OR pin to a specific version
[[plugin]]
name = "semgrep"
version = "X.Y.Z"
```

## Auto-enabling

Semgrep will be automatically enabled by `qlty init` if a `.semgrep.yaml` configuration file is present.

## Configuration files

- [`.semgrep.yaml`](https://semgrep.dev/docs/writing-rules/overview/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Semgrep.

## Languages and file types

Semgrep analyzes: all file types — target languages are determined by the rules in your `.semgrep.yaml` configuration.

## Troubleshooting

**"Skipping N invalid rules" warning appears in logs.**
One or more rules in your `.semgrep.yaml` contain a schema error or reference an unsupported metavariable.
Run `semgrep --validate --config .semgrep.yaml` locally to identify and fix or remove the invalid rule.

**Semgrep exits with code 7 but no issues appear in Qlty.**
Exit 7 means Semgrep found findings — this is expected. If Qlty is not surfacing them, check that your rule `severity` values map to levels Qlty recognizes (`ERROR` or `WARNING`).

## Links

- [Semgrep on GitHub](https://github.com/semgrep/semgrep)
- [Semgrep plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/semgrep)
- [Semgrep releases](https://github.com/semgrep/semgrep/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Semgrep is licensed under the [GNU Lesser General Public License v2.1](https://github.com/semgrep/semgrep/blob/develop/LICENSE).
