# Hadolint

[Hadolint](https://github.com/hadolint/hadolint) is a Dockerfile linter written in Haskell.

## Enabling Hadolint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable hadolint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "hadolint"

# OR pin to a specific version
[[plugin]]
name = "hadolint"
version = "X.Y.Z"
```

## Auto-enabling

Hadolint will be automatically enabled by `qlty init` if a `.hadolint.yaml` configuration file is present.

## Configuration files

- [`.hadolint.yaml`](https://github.com/hadolint/hadolint?tab=readme-ov-file#configure)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Hadolint.

## Languages and file types

Hadolint analyzes: Dockerfiles (`Dockerfile`, `*.dockerfile`).

## Troubleshooting

**hadolint reports no issues on a Dockerfile that contains best-practice violations.**
hadolint may be ignoring certain rules via a `.hadolint.yaml` config, or the Dockerfile may not be in a location that hadolint scans.
Check your `.hadolint.yaml` for `ignore:` entries that suppress the expected rules, and verify the Dockerfile path is not in `exclude_patterns`.

**hadolint flags a `RUN` instruction as `DL3008` (pin versions in apt-get) but you cannot pin them.**
Some base images or generated Dockerfiles intentionally use unversioned `apt-get install`. hadolint's `DL3008` rule is strict about this.
Add the rule to the `ignore:` list in `.hadolint.yaml` to suppress it project-wide, or add `# hadolint ignore=DL3008` on the line before the `RUN` instruction to suppress it locally.

## Links

- [Hadolint on GitHub](https://github.com/hadolint/hadolint)
- [Hadolint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/hadolint)
- [Hadolint releases](https://github.com/hadolint/hadolint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Hadolint is licensed under the [GNU General Public License v3.0](https://github.com/hadolint/hadolint/blob/master/LICENSE).
