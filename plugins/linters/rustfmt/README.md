# Rustfmt

[Rustfmt](https://github.com/rust-lang/rustfmt) is a tool for formatting Rust code according to style guidelines.

## Enabling Rustfmt

Enabling with the `qlty` CLI:

```bash
qlty plugins enable rustfmt
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
rustfmt = "latest"

# OR enable a specific version
[plugins.enabled]
rustfmt = "X.Y.Z"
```

## Auto-enabling

Rustfmt will be automatically enabled by `qlty init` if a `.rustfmt.toml` configuration file is present.

## Configuration files

- [`.rustfmt.toml`](https://github.com/rust-lang/rustfmt?tab=readme-ov-file#configuring-rustfmt)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Rustfmt.

## Links

- [Rustfmt on GitHub](https://github.com/rust-lang/rustfmt)
- [Rustfmt plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/rustfmt)
- [Rustfmt releases](https://github.com/rust-lang/rustfmt/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Rustfmt is licensed under the [MIT License](https://github.com/rust-lang/rustfmt/blob/master/LICENSE-MIT) and [Apache License 2.0](https://github.com/rust-lang/rustfmt/blob/master/LICENSE-APACHE).
