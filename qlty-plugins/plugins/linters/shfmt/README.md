# shfmt

[shfmt](https://github.com/mvdan/sh) is a shell code formatter that supports POSIX shell, Bash, and mksh, producing consistent formatting with no style debates.

## Enabling shfmt

Enabling with the `qlty` CLI:

```bash
qlty plugins enable shfmt
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "shfmt"

# OR pin to a specific version
[[plugin]]
name = "shfmt"
version = "X.Y.Z"
```

## Auto-enabling

shfmt will be automatically enabled by `qlty init` when shell script files are present.

shfmt will be automatically enabled by `qlty init` when shell script files are present.

## Languages and file types

shfmt formats: shell scripts (`.sh`, `.bash`) and files detected as shell by shebang line.

## Configuration files

shfmt reads indent size and other style settings from `.editorconfig`. No shfmt-specific config file is required.

- [EditorConfig support](https://github.com/mvdan/sh/blob/master/cmd/shfmt/shfmt.1.scd#editorconfig)

## Troubleshooting

**shfmt reports formatting differences but `qlty fmt` does not apply them.**
shfmt formatting is applied by `qlty fmt`, not by `qlty check`. Running `qlty check` will report unformatted files as `fmt`-level issues but will not modify them.
Run `qlty fmt` to auto-apply shfmt's formatting, or pass `--fix` to `qlty check`.

**shfmt changes indentation differently from what your editor uses.**
shfmt uses tabs by default. If your project uses spaces, shfmt will report every indented line as unformatted.
Add a `.editorconfig` with `indent_style = tab` (or `indent_style = space` with `indent_size = N`) in your project root, or add a `shfmt` config key in `.editorconfig`: `indent_style = space`.

## Links

- [shfmt on GitHub](https://github.com/mvdan/sh)
- [shfmt plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/shfmt)
- [shfmt releases](https://github.com/mvdan/sh/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

shfmt is licensed under the [BSD 3-Clause License](https://github.com/mvdan/sh/blob/master/LICENSE).
