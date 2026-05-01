# Shellcheck

[Shellcheck](https://github.com/koalaman/shellcheck) is a GPLv3 tool that gives warnings and suggestions for bash/sh shell scripts.

## Enabling Shellcheck

Enabling with the `qlty` CLI:

```bash
qlty plugins enable shellcheck
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "shellcheck"

# OR pin to a specific version
[[plugin]]
name = "shellcheck"
version = "X.Y.Z"
```

## Auto-enabling

Shellcheck will be automatically enabled by `qlty init` if a `shellcheckrc` configuration file is present.

## Configuration files

- `shellcheckrc`

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Shellcheck.

## Languages and file types

Shellcheck analyzes: shell scripts (`.sh`, `.bash`); also files detected as shell via shebang line.

## Troubleshooting

**Issues are not shown for scripts that source other files.**
shellcheck tries to follow `source` directives to analyze sourced scripts. If the sourced file cannot be found, shellcheck emits a `SC1090` or `SC1091` warning and skips analysis of the sourced code.
Add `# shellcheck source=/dev/null` above problematic `source` lines to suppress the warning, or use a `.shellcheckrc` to set `external-sources=true` and provide the correct path.

**shellcheck reports warnings on code that intentionally uses dynamic variable names.**
Patterns like `eval` or nameref variables cause shellcheck to emit false-positive warnings such as `SC2034` (variable appears unused) or `SC2116` (useless echo).
Suppress individual warnings inline with `# shellcheck disable=SC2034` on the affected line, or use a `.shellcheckrc` to disable rules project-wide.

## Links

- [Shellcheck on GitHub](https://github.com/koalaman/shellcheck)
- [Shellcheck plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/shellcheck)
- [Shellcheck releases](https://github.com/koalaman/shellcheck/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Shellcheck is licensed under the [GNU General Public License v3.0](https://github.com/koalaman/shellcheck/blob/master/LICENSE).
