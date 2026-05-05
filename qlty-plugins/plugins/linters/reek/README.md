# Reek

[Reek](https://github.com/troessner/reek) is a code smell detector for Ruby that examines classes, modules, and methods to identify patterns that may indicate design problems.

## Enabling Reek

Enabling with the `qlty` CLI:

```bash
qlty plugins enable reek
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "reek"

# OR pin to a specific version
[[plugin]]
name = "reek"
version = "X.Y.Z"
```

## Auto-enabling

Reek will be automatically enabled by `qlty init` if a `.reek.yml` configuration file is present.

Reek will be automatically enabled by `qlty init` if a `.reek.yml` configuration file is present.

## Languages and file types

Reek analyzes: Ruby (`.rb`).

## Configuration files

- [`.reek.yml`](https://github.com/troessner/reek/blob/master/docs/configuration-files.md)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Reek.

## Troubleshooting

**Issues are not shown for some files even though code smells exist.**
Qlty drops all issues from a file when Reek produces more than 100 results for it. Large legacy files — mailers, big model classes, report generators — frequently hit this limit.
Address smells in bulk on those files, or add them to `exclude_patterns` in `qlty.toml` if they are generated or deliberately not maintained.

## Links

- [Reek on GitHub](https://github.com/troessner/reek)
- [Reek plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/reek)
- [Reek releases](https://github.com/troessner/reek/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Reek is licensed under the [MIT License](https://github.com/troessner/reek/blob/master/LICENSE.txt).
