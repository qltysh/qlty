# oxc

[oxc](https://github.com/oxc-project/oxc) (oxlint) is a high-performance JavaScript and TypeScript linter written in Rust, designed to be significantly faster than ESLint while covering common correctness rules.

## Enabling oxc

Enabling with the `qlty` CLI:

```bash
qlty plugins enable oxc
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "oxc"

# OR pin to a specific version
[[plugin]]
name = "oxc"
version = "X.Y.Z"
```

## Auto-enabling

oxc will be automatically enabled by `qlty init` when JavaScript or TypeScript files are present.

oxc will be automatically enabled by `qlty init` if an `oxlintrc.json` configuration file is present.

## Languages and file types

oxc analyzes: JavaScript (`.js`, `.jsx`, `.mjs`, `.cjs`) and TypeScript (`.ts`, `.tsx`).

## Configuration files

- [`oxlintrc.json`](https://oxc.rs/docs/guide/usage/linter/config)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running oxc.

## Troubleshooting

**oxc reports no issues on a JavaScript file that has obvious problems.**
oxc's default ruleset is intentionally small and focused on correctness. Many style rules are not enabled unless a configuration file explicitly enables them.
Create an `oxlint.json` configuration file that enables the rule categories you need, or run `oxlint --list` to see available rules and their default state.

**oxc conflicts with eslint on the same files.**
Running both oxc and eslint on the same TypeScript/JavaScript files may produce duplicate or conflicting reports on the same lines.
If using both tools, configure oxc to disable rules already covered by eslint (and vice versa), or choose one tool for each rule category.

## Links

- [oxc on GitHub](https://github.com/oxc-project/oxc)
- [oxc plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/oxc)
- [oxc releases](https://github.com/oxc-project/oxc/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

oxc is licensed under the [MIT License](https://github.com/oxc-project/oxc/blob/main/LICENSE).
