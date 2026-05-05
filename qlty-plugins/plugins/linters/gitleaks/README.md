# Gitleaks

[Gitleaks](https://github.com/gitleaks/gitleaks) is a SAST tool for detecting and preventing hardcoded secrets like passwords, api keys, and tokens in git repos. Gitleaks is an easy-to-use, all-in-one solution for detecting secrets, past or present, in your code.

## Enabling Gitleaks

Enabling with the `qlty` CLI:

```bash
qlty plugins enable gitleaks
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "gitleaks"

# OR pin to a specific version
[[plugin]]
name = "gitleaks"
version = "X.Y.Z"
```

## Auto-enabling

Gitleaks will be automatically enabled by `qlty init` if a `.gitleaks.toml` configuration file is present.

## Configuration files

- [`.gitleaks.toml`](https://github.com/gitleaks/gitleaks/tree/master?tab=readme-ov-file#configuration)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Gitleaks.

## Languages and file types

Gitleaks analyzes: all file types — it scans the full repository history and working tree for leaked secrets.

## Troubleshooting

**gitleaks reports a detected secret on a file that contains only test fixtures or example credentials.**
gitleaks uses pattern matching and may flag example API keys, dummy tokens, or test fixtures as real secrets.
Add a `.gitleaks.toml` configuration file with `[[allowlist]]` rules to whitelist known-safe strings or file paths. Place the config in `.qlty/configs/` so Qlty can stage it for the run.

**gitleaks reports no secrets on a file that you know contains a committed credential.**
By default, qlty runs gitleaks in `--no-git` filesystem mode. If the credential was added in a past commit and then removed from the current state of the file, gitleaks will not find it.
To scan git history, run `gitleaks detect --source=. --log-opts="--all"` directly. Qlty only scans the current file state.

## Links

- [Gitleaks on GitHub](https://github.com/gitleaks/gitleaks)
- [Gitleaks plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/gitleaks)
- [Gitleaks releases](https://github.com/gitleaks/gitleaks/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Gitleaks is licensed under the [MIT License](https://github.com/gitleaks/gitleaks/blob/master/LICENSE).
