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

## Links

- [syntax_tree on GitHub](https://github.com/ruby-syntax-tree/syntax_tree)
- [ruby-stree plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/ruby-stree)
- [syntax_tree releases](https://github.com/ruby-syntax-tree/syntax_tree/tags)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

syntax_tree is licensed under the [MIT License](https://github.com/ruby-syntax-tree/syntax_tree/blob/main/LICENSE).
