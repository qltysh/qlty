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
[plugins.enabled]
shellcheck = "latest"

# OR enable a specific version
[plugins.enabled]
shellcheck = "X.Y.Z"
```

## Auto-enabling

Shellcheck will be automatically enabled by `qlty init` if a `shellcheckrc` configuration file is present.

## Configuration files

- `shellcheckrc`

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Shellcheck.

## Links

- [Shellcheck on GitHub](https://github.com/koalaman/shellcheck)
- [Shellcheck plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/shellcheck)
- [Shellcheck releases](https://github.com/koalaman/shellcheck/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Shellcheck is licensed under the [GNU General Public License v3.0](https://github.com/koalaman/shellcheck/blob/master/LICENSE).
