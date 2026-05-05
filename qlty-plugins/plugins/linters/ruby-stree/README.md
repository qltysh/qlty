# ruby-stree

Ruby [syntax_tree](https://github.com/ruby-syntax-tree/syntax_tree) is a library to parse Ruby code which provides an auto-formatter.

## Enabling ruby-stree

Enabling with the `qlty` CLI:

```bash
qlty plugins enable ruby-stree
```

Or by editing `qlty.toml`:

```toml
[[plugin]]
name = "ruby-stree"
```

## Auto-enabling

ruby-stree will be automatically enabled by `qlty init` if a `.streerc` configuration file is present.

## Configuration files

- [`.streerc`](https://github.com/ruby-syntax-tree/syntax_tree?tab=readme-ov-file#configuration)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running ruby-stree.

## Languages and file types

ruby-stree formats: Ruby (`.rb`).

## Troubleshooting

**"Error installing ruby" with "The dependencies below are missing" on macOS.**
Qlty manages its own Ruby installation for Ruby-based tools. If `libyaml` is not present on the system, the Ruby build fails before ruby-stree can run.
Install the missing dependency with `brew install libyaml` and re-run `qlty check`.

**ruby-stree produces no output on files that differ from stree's style.**
Unlike linters, ruby-stree is a formatter: it reports formatting differences as `fmt`-level issues, not error-level issues. Run `qlty fmt` (not `qlty check`) to apply stree's formatting, or pass `--level fmt` to see formatting issues in check output.

## Links

- [syntax_tree on GitHub](https://github.com/ruby-syntax-tree/syntax_tree)
- [ruby-stree plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/ruby-stree)
- [syntax_tree releases](https://github.com/ruby-syntax-tree/syntax_tree/tags)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

syntax_tree is licensed under the [MIT License](https://github.com/ruby-syntax-tree/syntax_tree/blob/main/LICENSE).
