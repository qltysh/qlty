# Google-java-format

[Google-java-format](https://github.com/google/google-java-format) is a program that reformats Java source code to comply with [Google Java Style](https://google.github.io/styleguide/javaguide.html).

## Enabling Google-java-format

Enabling with the `qlty` CLI:

```bash
qlty plugins enable google-java-format
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "google-java-format"

# OR pin to a specific version
[[plugin]]
name = "google-java-format"
version = "X.Y.Z"
```

## Languages and file types

Google-java-format formats: Java (`.java`).

## Troubleshooting

**google-java-format fails with "java.lang.UnsupportedClassVersionError" or version error.**
google-java-format requires Java 11 or higher. If the system Java version is older, the tool fails to start.
Ensure the Java runtime available to Qlty is version 11+. Qlty manages its own Java installation, so check that the qlty-managed Java version is compatible.

**google-java-format reformats code in a way that your team does not want.**
google-java-format enforces Google's Java style, which uses 2-space indentation and a 100-character line limit. These cannot be customised — google-java-format has no configuration options.
If your project uses a different style (for example 4-space indentation), google-java-format is not a good fit. Consider using checkstyle with a custom ruleset instead.

## Links

- [Google-java-format on GitHub](https://github.com/google/google-java-format)
- [Google-java-format plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/google-java-format)
- [Google-java-format releases](https://github.com/google/google-java-format/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Google-java-format is licensed under the [Apache License Version 2.0](https://github.com/google/google-java-format/blob/v1.22.0/LICENSE).
