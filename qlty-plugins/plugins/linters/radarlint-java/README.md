# radarlint-java

radarlint-java is a static analysis tool for Java code, running as a language-specific mode of the radarlint engine.

<!-- REVIEW: add upstream docs URL if radarlint has a public home page -->

## Enabling radarlint-java

Enabling with the `qlty` CLI:

```bash
qlty plugins enable radarlint-java
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "radarlint-java"

# OR pin to a specific version
[[plugin]]
name = "radarlint-java"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

## Languages and file types

radarlint-java analyzes: Java (`.java`).

## Configuration files

- `radarlint.properties`

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running radarlint-java.

## Troubleshooting

**RadarLint reports no issues on the first run.**
RadarLint downloads its analysis engine on first use. In environments with restricted network access, the download may fail silently and no analysis will run.
Check the qlty build log for download errors, ensure outbound HTTPS access to the RadarLint release endpoint is permitted, and re-run.

**RadarLint is slow on large codebases.**
RadarLint performs deep static analysis that is more computationally intensive than standard linters. Large files or modules with high complexity can take significantly longer to analyse.
Add large generated or vendor directories to `exclude_patterns` in `qlty.toml` to reduce the analysis surface.

## Links

- [radarlint-java plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/radarlint-java)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)
