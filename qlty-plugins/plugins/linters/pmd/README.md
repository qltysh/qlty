# PMD

[PMD](https://github.com/pmd/pmd) is a source code analyzer that finds common programming flaws such as unused variables, empty catch blocks, and unnecessary object creation in Java and Apex code.

## Enabling PMD

Enabling with the `qlty` CLI:

```bash
qlty plugins enable pmd
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "pmd"

# OR pin to a specific version
[[plugin]]
name = "pmd"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

PMD will be automatically enabled by `qlty init` when Java or Apex files are present.

## Languages and file types

PMD analyzes: Java (`.java`) and Apex (`.cls`, `.trigger`). Qlty runs PMD with the built-in `quickstart` ruleset for each language.

## Troubleshooting

**PMD reports no issues on Java code that has obvious violations.**
PMD requires a ruleset XML file or a built-in ruleset name to be specified. Without a configuration, it may run with no active rules.
Create a `pmd-ruleset.xml` or use a built-in preset like `rulesets/java/quickstart.xml` in your PMD configuration to enable the checks you need.

**PMD exits with "no ruleset found" or similar configuration error.**
PMD's configuration file path must be accessible at run time. If `pmd-ruleset.xml` is in a subdirectory that Qlty does not stage alongside the source files, PMD cannot find it.
Place the ruleset file in `.qlty/configs/` so Qlty can stage it for the run.

## Links

- [PMD on GitHub](https://github.com/pmd/pmd)
- [PMD plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/pmd)
- [PMD releases](https://github.com/pmd/pmd/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

PMD is licensed under the [BSD 4-Clause License](https://github.com/pmd/pmd/blob/master/LICENSE).
