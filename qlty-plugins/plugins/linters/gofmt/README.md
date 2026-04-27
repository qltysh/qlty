# gofmt

[gofmt](https://pkg.go.dev/cmd/gofmt) is the standard Go source code formatter, part of the Go toolchain. It enforces the canonical Go code style with no configuration required.

## Enabling gofmt

Enabling with the `qlty` CLI:

```bash
qlty plugins enable gofmt
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "gofmt"

# OR pin to a specific version
[[plugin]]
name = "gofmt"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

gofmt will be automatically enabled by `qlty init` when Go files are present.

## Languages and file types

gofmt formats: Go (`.go`).

## Troubleshooting

**gofmt reports formatting differences on files that look correctly formatted.**
gofmt enforces a specific canonical style (tabs for indentation, specific spacing around operators). Editors that use spaces instead of tabs, or that format slightly differently from gofmt, will cause persistent differences.
Run `gofmt -w .` from the repository root to apply gofmt's formatting, then commit the result. Use `qlty fmt` for ongoing formatting.

**gofmt does not check files outside the main module.**
gofmt only processes `.go` files that are valid Go source. Generated `.pb.go` or `.gen.go` files may be in `exclude_patterns` or simply not in the scanned set.
Ensure generated files are added to `exclude_patterns` if you do not want them formatted.

## Links

- [gofmt documentation](https://pkg.go.dev/cmd/gofmt)
- [gofmt plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/gofmt)
- [Go releases](https://github.com/golang/go/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

gofmt is part of the Go project, licensed under the [BSD 3-Clause License](https://github.com/golang/go/blob/master/LICENSE).
