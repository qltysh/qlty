# TruffleHog

[TruffleHog](https://github.com/trufflesecurity/trufflehog) is a security tool that scans code repositories to find secrets accidentally committed to a codebase.

## Enabling TruffleHog

Enabling with the `qlty` CLI:

```bash
qlty plugins enable trufflehog
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
trufflehog = "latest"

# OR enable a specific version
[plugins.enabled]
trufflehog = "X.Y.Z"
```

## Links

- [TruffleHog on GitHub](https://github.com/trufflesecurity/trufflehog)
- [TruffleHog plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/trufflehog)
- [TruffleHog releases](https://github.com/trufflesecurity/trufflehog/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

TruffleHog is licensed under the [GNU Affero General Public License v3.0](https://github.com/trufflesecurity/trufflehog/blob/main/LICENSE).
