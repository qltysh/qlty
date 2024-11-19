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
[plugins.enabled]
google-java-format = "latest"

# OR enable a specific version
[plugins.enabled]
google-java-format = "X.Y.Z"
```

## Links

- [Google-java-format on GitHub](https://github.com/google/google-java-format)
- [Google-java-format plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/google-java-format)
- [Google-java-format releases](https://github.com/google/google-java-format/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Google-java-format is licensed under the [Apache License Version 2.0](https://github.com/google/google-java-format/blob/v1.22.0/LICENSE).
