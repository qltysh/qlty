# golangci-lint

[golangci-lint](https://github.com/golangci/golangci-lint) is a fast Go linters runner that aggregates dozens of linters into a single tool with shared parsing and caching.

## Enabling golangci-lint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable golangci-lint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "golangci-lint"

# OR pin to a specific version
[[plugin]]
name = "golangci-lint"
version = "X.Y.Z"
```

## Auto-enabling

golangci-lint will be automatically enabled by `qlty init` if a `.golangci.yml` (or `.golangci.json`, `.golangci.toml`) configuration file is present.

golangci-lint will be automatically enabled by `qlty init` if a `.golangci.yml` configuration file is present.

## Languages and file types

golangci-lint analyzes: Go (`.go`). It requires a `go.mod` file in the project root or a parent directory.

## Configuration files

- [`.golangci.yml`](https://golangci-lint.run/usage/configuration/)

Accepted filenames: `.golangci.yml`, `.golangci.yaml`, `.golangci.toml`, `.golangci.json`.

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running golangci-lint.

## Troubleshooting

**golangci-lint exits with "failed to load packages" or "build constraints" errors.**
golangci-lint requires the project to be a valid Go module. If dependencies are missing (`go mod download` has not been run) or the module graph is broken, the loader fails before linting starts.
Run `go mod tidy && go mod download` in the project root and retry. Ensure a `go.sum` file is present and up to date.

**golangci-lint runs many linters and is slow.**
By default, golangci-lint runs a large set of linters in parallel. On large codebases this can be slow and may time out.
Create a `.golangci.yml` configuration file that enables only the linters you need under `linters.enable`, and disable the rest with `linters.disable-all: true`.

## Links

- [golangci-lint on GitHub](https://github.com/golangci/golangci-lint)
- [golangci-lint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/golangci-lint)
- [golangci-lint releases](https://github.com/golangci/golangci-lint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

golangci-lint is licensed under the [GNU General Public License v3.0](https://github.com/golangci/golangci-lint/blob/master/LICENSE).
