# Knip

[Knip](https://github.com/webpro-nl/knip) finds unused files, dependencies, and exports in JavaScript and TypeScript projects, helping keep codebases lean and free of dead code.

## Enabling Knip

Enabling with the `qlty` CLI:

```bash
qlty plugins enable knip
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "knip"

# OR pin to a specific version
[[plugin]]
name = "knip"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

Knip will be automatically enabled by `qlty init` if a `knip.json` configuration file is present.

## Languages and file types

Knip analyzes: JavaScript (`.js`, `.jsx`, `.mjs`, `.cjs`) and TypeScript (`.ts`, `.tsx`). It requires a `package.json` in the project root.

## Configuration files

- [`knip.json`](https://knip.dev/overview/configuration)

Accepted filenames: `knip.json`, `knip.jsonc`, `.knip.json`, `.knip.jsonc`, `knip.ts`, `knip.js`, `knip.config.js`.

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Knip.

## Troubleshooting

**knip reports no unused exports even though there are clearly unused files.**
knip uses the project's entry points to trace which exports are used. If the entry points are not configured correctly (for example if `knip.json` is absent or lists wrong entry files), knip may not traverse all code paths.
Create a `knip.json` configuration file that specifies your project's `entry` files and `project` glob patterns so knip can trace the dependency graph correctly.

**knip reports false positives on dynamic imports or plugin systems.**
Knip performs static analysis and cannot follow dynamic `require()` calls, `require.context()`, or plugin loader patterns. Modules loaded dynamically will appear unused.
Add the dynamically-loaded paths to the `ignore` list in `knip.json` to suppress false positives.

## Links

- [Knip on GitHub](https://github.com/webpro-nl/knip)
- [Knip plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/knip)
- [Knip releases](https://github.com/webpro-nl/knip/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Knip is licensed under the [ISC License](https://github.com/webpro-nl/knip/blob/main/LICENSE).
