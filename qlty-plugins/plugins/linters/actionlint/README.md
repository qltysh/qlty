# Actionlint

[Actionlint](https://github.com/rhysd/actionlint) is a static checker for GitHub Actions workflow files.

## Enabling Actionlint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable actionlint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "actionlint"

# OR pin to a specific version
[[plugin]]
name = "actionlint"
version = "X.Y.Z"
```

## Auto-enabling

Actionlint will be automatically enabled by `qlty init` if a `.github/actionlint.yaml` configuration file is present.

## Configuration files

- [`.github/actionlint.yaml`](https://github.com/rhysd/actionlint/blob/main/docs/config.md#configuration-file)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Actionlint.

## Languages and file types

Actionlint analyzes: GitHub Actions workflow files (`.github/workflows/*.yml`, `.github/workflows/*.yaml`).

## Troubleshooting

**actionlint reports no issues in a workflow that contains syntax errors.**
If the workflow file cannot be parsed (for example due to invalid YAML), actionlint may exit without producing issue output rather than failing visibly.
Run `actionlint` directly on the workflow file (`actionlint .github/workflows/ci.yml`) to see parser errors that are not surfaced through Qlty.

**actionlint warns about expressions in `env:` or `with:` that reference undefined context variables.**
actionlint statically checks GitHub Actions context variable names. If a workflow uses a context variable that actionlint does not recognise (for example a custom reusable workflow input), it may flag it as undefined.
Use a `actionlint.yml` config file with `self-hosted-runner.labels` or similar to teach actionlint about your environment, or add a `.actionlintignore` to suppress false positives.

## Links

- [Actionlint on GitHub](https://github.com/rhysd/actionlint)
- [Actionlint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/actionlint)
- [Actionlint releases](https://github.com/rhysd/actionlint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Actionlint is licensed under the [MIT License](https://github.com/rhysd/actionlint/blob/main/LICENSE.txt).
