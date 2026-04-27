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
[[plugin]]
name = "sqlfluff"

# OR pin to a specific version
[[plugin]]
name = "sqlfluff"
version = "X.Y.Z"
```

## Auto-enabling

Sqlfluff will be automatically enabled by `qlty init` if a `.sqlfluff` configuration file is present.

## Configuration files

- [`.sqlfluff`](https://docs.sqlfluff.com/en/stable/configuration.html#configuration-files)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Sqlfluff.

## Languages and file types

SQLFluff analyzes: SQL (`.sql`) and Jinja-templated SQL (`.sql.j2`) used by dbt and similar tools.

## Troubleshooting

**"Requested templater 'dbt-cloud' which is not currently available" error.**
The Qlty-bundled SQLFluff only supports the `raw`, `jinja`, `python`, and `placeholder` templaters. The `dbt-cloud` (and `dbt`) templater requires the `sqlfluff-templater-dbt` package, which is not included.
Change `templater = dbt-cloud` (or `templater = dbt`) in your `.sqlfluff` to `templater = jinja` or `templater = placeholder`. The `placeholder` templater is the safest choice for dbt projects when the dbt templater is unavailable.

**SQLFluff reports "dialect was not supplied" on every file.**
A missing or unrecognised `dialect` in `.sqlfluff` causes SQLFluff to fall back to ANSI mode, which may not recognise dialect-specific syntax.
Add `dialect = <your_dialect>` (e.g., `dialect = snowflake`) to the `[sqlfluff]` section of your `.sqlfluff` configuration file.

## Links

- [Sqlfluff on GitHub](https://github.com/sqlfluff/sqlfluff)
- [Sqlfluff plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/sqlfluff)
- [Sqlfluff releases](https://github.com/sqlfluff/sqlfluff/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Sqlfluff is licensed under the [MIT License](https://github.com/sqlfluff/sqlfluff/blob/main/LICENSE.md).
