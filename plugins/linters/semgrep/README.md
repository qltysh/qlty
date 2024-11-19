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
[plugins.enabled]
semgrep = "latest"

# OR enable a specific version
[plugins.enabled]
semgrep = "X.Y.Z"
```

## Auto-enabling

Semgrep will be automatically enabled by `qlty init` if a `.semgrep.yaml` configuration file is present.

## Configuration files

- [`.semgrep.yaml`](https://semgrep.dev/docs/writing-rules/overview/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Semgrep.

## Links

- [Semgrep on GitHub](https://github.com/semgrep/semgrep)
- [Semgrep plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/semgrep)
- [Semgrep releases](https://github.com/semgrep/semgrep/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Semgrep is licensed under the [GNU Lesser General Public License v2.1](https://github.com/semgrep/semgrep/blob/develop/LICENSE).
