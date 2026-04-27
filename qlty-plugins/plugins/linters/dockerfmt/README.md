# dockerfmt

[dockerfmt](https://github.com/reteps/dockerfmt) is a Dockerfile formatter that parses and rewrites Dockerfiles with consistent style.

## Enabling dockerfmt

Enabling with the `qlty` CLI:

```bash
qlty plugins enable dockerfmt
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "dockerfmt"

# OR pin to a specific version
[[plugin]]
name = "dockerfmt"
version = "X.Y.Z"
```

## Languages and file types

dockerfmt formats: Dockerfiles (`Dockerfile`, `*.dockerfile`).

## Troubleshooting

**dockerfmt reports formatting differences but the Dockerfile looks correct.**
dockerfmt enforces its own canonical formatting for Dockerfiles which may differ from existing project conventions — for example, normalising argument order in `RUN` instructions or adding line continuation backslashes.
These are formatting-only differences with no semantic impact. Run `qlty fmt` to apply dockerfmt's style, or add the Dockerfile to `exclude_patterns` in `qlty.toml` if you want to keep the existing format.

**dockerfmt produces no output on a Dockerfile with obvious style issues.**
If the Dockerfile already conforms to dockerfmt's style, or if it is excluded by `exclude_patterns`, dockerfmt will produce no output.
Verify that the file is not matched by an exclusion pattern and run `dockerfmt` directly on the file to confirm it detects a difference.

## Links

- [dockerfmt on GitHub](https://github.com/reteps/dockerfmt)
- [dockerfmt plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/dockerfmt)
- [dockerfmt releases](https://github.com/reteps/dockerfmt/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

dockerfmt is licensed under the [MIT License](https://github.com/reteps/dockerfmt/blob/main/LICENSE).
