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
[[plugin]]
name = "rustfmt"

# OR pin to a specific version
[[plugin]]
name = "rustfmt"
version = "X.Y.Z"
```

## Auto-enabling

Rustfmt will be automatically enabled by `qlty init` if a `.rustfmt.toml` configuration file is present.

## Configuration files

- [`.rustfmt.toml`](https://github.com/rust-lang/rustfmt?tab=readme-ov-file#configuring-rustfmt)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Rustfmt.

## Languages and file types

Rustfmt formats: Rust (`.rs`).

## Troubleshooting

**rustfmt does not format files in a workspace subdirectory.**
rustfmt requires a `rustfmt.toml` or `rust-project.json` at the crate root. In Cargo workspaces, each member crate may need its own configuration or rustfmt may not find the correct edition setting.
Ensure a `rustfmt.toml` is present in the workspace root with `edition = "2021"` (or whatever edition the workspace uses). Rustfmt inherits this configuration for member crates.

**rustfmt reports "error: the attribute `#![feature(...)]` may not be used on stable" on nightly-only code.**
rustfmt on stable Rust cannot parse some nightly-only syntax. If the project uses nightly-only features, rustfmt will fail to parse those files.
Add a `rustfmt.toml` with `unstable_features = true` and run qlty with a nightly toolchain via `RUSTUP_TOOLCHAIN=nightly`.

## Links

- [Rustfmt on GitHub](https://github.com/rust-lang/rustfmt)
- [Rustfmt plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/rustfmt)
- [Rustfmt releases](https://github.com/rust-lang/rustfmt/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Rustfmt is licensed under the [MIT License](https://github.com/rust-lang/rustfmt/blob/master/LICENSE-MIT) and [Apache License 2.0](https://github.com/rust-lang/rustfmt/blob/master/LICENSE-APACHE).
