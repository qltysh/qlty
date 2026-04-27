# Prisma

[Prisma](https://github.com/prisma/prisma) includes a built-in formatter for Prisma schema files that enforces consistent style and validates schema syntax.

## Enabling Prisma

Enabling with the `qlty` CLI:

```bash
qlty plugins enable prisma
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "prisma"

# OR pin to a specific version
[[plugin]]
name = "prisma"
version = "X.Y.Z"
```

## Auto-enabling

Prisma will be automatically enabled by `qlty init` when `.prisma` schema files are present.

Prisma will be automatically enabled by `qlty init` when Prisma schema files are present.

## Languages and file types

Prisma formats: Prisma schema files (`.prisma`).

## Troubleshooting

**Prisma reports no issues even though the schema has validation errors.**
Qlty runs `prisma validate` on `.prisma` schema files. If the `DATABASE_URL` environment variable is not set, some validation checks may be skipped.
Set `DATABASE_URL` to a valid connection string (or a placeholder like `postgresql://user:pass@localhost/db`) when running Qlty in CI to ensure full schema validation.

**Prisma validator exits with "environment variable not found: DATABASE_URL".**
The Prisma schema's `datasource` block references a `DATABASE_URL` environment variable. If that variable is not set in the environment where Qlty runs, Prisma exits with an error.
Set `DATABASE_URL` to any valid connection string before running `qlty check`, or use the `env()` function with a fallback in the schema.

## Links

- [Prisma on GitHub](https://github.com/prisma/prisma)
- [Prisma plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/prisma)
- [Prisma releases](https://github.com/prisma/prisma/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Prisma is licensed under the [Apache License 2.0](https://github.com/prisma/prisma/blob/main/LICENSE).
