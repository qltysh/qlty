# ast-grep

[ast-grep](https://github.com/ast-grep/ast-grep) is a fast, polyglot CLI tool for code structural search, lint, and rewriting using abstract syntax tree (AST) patterns.

## Enabling ast-grep

Enabling with the `qlty` CLI:

```bash
qlty plugins enable ast-grep
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "ast-grep"

# OR pin to a specific version
[[plugin]]
name = "ast-grep"
version = "X.Y.Z"
```

## Auto-enabling

ast-grep will be automatically enabled by `qlty init` if a `sgconfig.yml` configuration file is present.

ast-grep will be automatically enabled by `qlty init` if a `sgconfig.yml` configuration file is present.

## Languages and file types

ast-grep supports all file types through language-specific parsers. The languages and rules to scan are defined in your `sgconfig.yml`.

## Configuration files

- [`sgconfig.yml`](https://ast-grep.github.io/reference/sgconfig.html)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running ast-grep.

## Troubleshooting

**ast-grep does not match any files even though the project has source files.**
ast-grep requires rule files that specify which language and pattern to match. Without rule files, it runs but finds nothing.
Create a rules directory (or individual `.yaml` rule files) and configure `sgconfig.yml` to point to them. Then add the `sgconfig.yml` path to the plugin's `config_files` in `qlty.toml` if needed.

**ast-grep matches files in the wrong language.**
ast-grep uses a `language` field in each rule file. If the language does not match the file's extension, ast-grep will not apply the rule.
Verify that each rule's `language:` field (for example `language: JavaScript`) matches the files you want to check.

## Links

- [ast-grep on GitHub](https://github.com/ast-grep/ast-grep)
- [ast-grep plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/ast-grep)
- [ast-grep releases](https://github.com/ast-grep/ast-grep/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

ast-grep is licensed under the [MIT License](https://github.com/ast-grep/ast-grep/blob/main/LICENSE).
