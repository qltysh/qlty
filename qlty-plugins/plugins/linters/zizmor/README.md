# zizmor

[zizmor](https://github.com/woodruffw/zizmor) is a static analysis tool for GitHub Actions workflows that identifies security vulnerabilities such as script injection, excessive permissions, and unsafe use of third-party actions.

## Enabling zizmor

Enabling with the `qlty` CLI:

```bash
qlty plugins enable zizmor
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "zizmor"

# OR pin to a specific version
[[plugin]]
name = "zizmor"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

zizmor will be automatically enabled by `qlty init` when GitHub Actions workflow files are present.

## Languages and file types

zizmor analyzes: GitHub Actions workflow files (`.github/workflows/*.yml`, `.github/workflows/*.yaml`).

## Configuration files

- [`zizmor.yml`](https://docs.zizmor.sh/configuration/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running zizmor.

## Troubleshooting

**zizmor reports "artipacked" issues on workflows that upload or download artifacts.**
`artipacked` is a zizmor finding for workflows where GitHub Actions artifacts are used in a way that could allow credential persistence — for example, using `actions/upload-artifact` with `path: .` in a job that has write access to secrets.
These are security findings, not false positives. Scope artifact uploads to only the files needed, and avoid uploading the workspace root.

**zizmor reports no issues even though a workflow has `${{ github.event.pull_request.head.sha }}`.**
zizmor may be running an older version that does not yet detect a specific injection pattern, or the workflow structure places the expansion outside the context zizmor currently analyzes.
Check which zizmor version Qlty is using (`qlty plugins list`) and upgrade the version in `qlty.toml` if a newer release covers the pattern you expect.

## Links

- [zizmor on GitHub](https://github.com/woodruffw/zizmor)
- [zizmor plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/zizmor)
- [zizmor releases](https://github.com/woodruffw/zizmor/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

zizmor is licensed under the [MIT License](https://github.com/woodruffw/zizmor/blob/main/LICENSE).
