# Sqlfluff

[Sqlfluff](https://github.com/sqlfluff/sqlfluff) is a dialect-flexible and configurable SQL linter. Designed with ELT applications in mind, SQLFluff also works with Jinja templating and dbt. SQLFluff will auto-fix most linting errors, allowing you to focus your time on what matters.

## Enabling Sqlfluff

Enabling with the `qlty` CLI:

```bash
qlty plugins enable sqlfluff
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
sqlfluff = "latest"

# OR enable a specific version
[plugins.enabled]
sqlfluff = "X.Y.Z"
```

## Auto-enabling

Sqlfluff will be automatically enabled by `qlty init` if a `.sqlfluff` configuration file is present.

## Configuration files

- [`.sqlfluff`](https://docs.sqlfluff.com/en/stable/configuration.html#configuration-files)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Sqlfluff.

## Links

- [Sqlfluff on GitHub](https://github.com/sqlfluff/sqlfluff)
- [Sqlfluff plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/sqlfluff)
- [Sqlfluff releases](https://github.com/sqlfluff/sqlfluff/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Sqlfluff is licensed under the [MIT License](https://github.com/sqlfluff/sqlfluff/blob/main/LICENSE.md).
