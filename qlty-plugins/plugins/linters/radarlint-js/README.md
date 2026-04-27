# radarlint-js

radarlint-js is a static analysis tool for JavaScript and TypeScript code, running as a language-specific mode of the radarlint engine.

<!-- REVIEW: add upstream docs URL if radarlint has a public home page -->

## Enabling radarlint-js

Enabling with the `qlty` CLI:

```bash
qlty plugins enable radarlint-js
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "radarlint-js"

# OR pin to a specific version
[[plugin]]
name = "radarlint-js"
version = "X.Y.Z"
```

radarlint-js analyzes: JavaScript (`.js`, `.jsx`, `.mjs`, `.cjs`) and TypeScript (`.ts`, `.tsx`).

## Configuration files

- `radarlint.properties`

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running radarlint-js.

## Troubleshooting

**RadarLint reports no issues on the first run.**
RadarLint downloads its analysis engine on first use. In environments with restricted network access, the download may fail silently and no analysis will run.
Check the qlty build log for download errors, ensure outbound HTTPS access to the RadarLint release endpoint is permitted, and re-run.

**RadarLint is slow on large codebases.**
RadarLint performs deep static analysis that is more computationally intensive than standard linters. Large files or modules with high complexity can take significantly longer to analyse.
Add large generated or vendor directories to `exclude_patterns` in `qlty.toml` to reduce the analysis surface.

## Links

- [radarlint-js plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/radarlint-js)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)
