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
[[plugin]]
name = "trufflehog"

# OR pin to a specific version
[[plugin]]
name = "trufflehog"
version = "X.Y.Z"
```

## Languages and file types

TruffleHog analyzes: all file types — it scans the entire repository for committed secrets.

## Troubleshooting

**TruffleHog reports no secrets on the first run, then finds them on a second run.**
TruffleHog uses `--only-verified` by default, which performs live network requests to verify whether discovered credentials are still active. On the first run in a new environment, network timeouts or rate limits may cause verifications to be skipped, resulting in no output.
If you expect secrets to be reported, run `qlty check --filter trufflehog` a second time once the network is stable, or add `--no-verification` to the driver script in your plugin configuration to report all matches without verification.

**TruffleHog is slow on large repositories.**
TruffleHog scans every file in the repository on each run. Repositories with many large binary files or deep git histories can cause slow runs.
Add large binary directories (for example `assets/`, `fixtures/`) to `exclude_patterns` in `qlty.toml` to skip them.

## Links

- [TruffleHog on GitHub](https://github.com/trufflesecurity/trufflehog)
- [TruffleHog plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/trufflehog)
- [TruffleHog releases](https://github.com/trufflesecurity/trufflehog/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

TruffleHog is licensed under the [GNU Affero General Public License v3.0](https://github.com/trufflesecurity/trufflehog/blob/main/LICENSE).
