# Trivy

[Trivy](https://github.com/aquasecurity/trivy) is a comprehensive and versatile security scanner. Trivy has scanners that look for security issues, and targets where it can find those issues.

## Enabling Trivy

Enabling with the `qlty` CLI:

```bash
qlty plugins enable trivy
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
trivy = "latest"

# OR enable a specific version
[plugins.enabled]
trivy = "X.Y.Z"
```

## Auto-enabling

Trivy will be automatically enabled by `qlty init` if a `trivy.yaml` configuration file is present.

## Configuration files

- [`trivy.yaml`](https://aquasecurity.github.io/trivy/v0.50/docs/references/configuration/config-file/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Trivy.

## Links

- [Trivy on GitHub](https://github.com/aquasecurity/trivy)
- [Trivy plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/trivy)
- [Trivy releases](https://github.com/aquasecurity/trivy/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Trivy is licensed under the [Apache License v2.0](https://github.com/aquasecurity/trivy/blob/main/LICENSE).
