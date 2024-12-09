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
[plugins.enabled]
gitleaks = "latest"

# OR enable a specific version
[plugins.enabled]
gitleaks = "X.Y.Z"
```

## Auto-enabling

Gitleaks will be automatically enabled by `qlty init` if a `.gitleaks.toml` configuration file is present.

## Configuration files

- [`.gitleaks.toml`](https://github.com/gitleaks/gitleaks/tree/master?tab=readme-ov-file#configuration)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Gitleaks.

## Links

- [Gitleaks on GitHub](https://github.com/gitleaks/gitleaks)
- [Gitleaks plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/gitleaks)
- [Gitleaks releases](https://github.com/gitleaks/gitleaks/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Gitleaks is licensed under the [MIT License](https://github.com/gitleaks/gitleaks/blob/master/LICENSE).
