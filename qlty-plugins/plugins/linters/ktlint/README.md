# ktlint

[ktlint](https://github.com/pinterest/ktlint) is an anti-bikeshedding Kotlin linter and formatter with built-in rules and no configuration required for the defaults.

## Enabling ktlint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable ktlint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "ktlint"

# OR pin to a specific version
[[plugin]]
name = "ktlint"
version = "X.Y.Z"
```

## Auto-enabling

ktlint will be automatically enabled by `qlty init` when Kotlin files are present.

ktlint will be automatically enabled by `qlty init` when Kotlin files are present.

## Languages and file types

ktlint analyzes: Kotlin (`.kt`, `.kts`).

## Configuration files

ktlint reads standard `.editorconfig` properties for indent size, line length, and other style settings. No ktlint-specific config file is required.

- [`.editorconfig` support](https://pinterest.github.io/ktlint/latest/rules/configuration-ktlint/)

## Troubleshooting

**ktlint fails with "java.lang.UnsupportedClassVersionError" or JVM version error.**
ktlint requires a Java 11+ runtime. If the JVM version available to Qlty is older, ktlint fails to start.
Ensure the Qlty-managed Java installation is version 11 or higher.

**ktlint reports violations in files that use experimental Kotlin features.**
ktlint may not yet support the latest Kotlin syntax. Using experimental language features or very new compiler directives can cause parse errors.
Pin the ktlint version in `qlty.toml` to a version that matches the Kotlin version your project uses.

## Links

- [ktlint on GitHub](https://github.com/pinterest/ktlint)
- [ktlint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/ktlint)
- [ktlint releases](https://github.com/pinterest/ktlint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

ktlint is licensed under the [MIT License](https://github.com/pinterest/ktlint/blob/master/LICENSE).
