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
[[plugin]]
name = "trivy"

# OR pin to a specific version
[[plugin]]
name = "trivy"
version = "X.Y.Z"
```

## Auto-enabling

Trivy will not be automatically enabled by `qlty init`

## Configuration files

- [`trivy.yaml`](https://aquasecurity.github.io/trivy/v0.50/docs/references/configuration/config-file/)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Trivy.

## Languages and file types

Trivy analyzes: all file types — it scans lockfiles, IaC configs (Dockerfiles, Terraform, etc.), and source files for vulnerabilities.

## Troubleshooting

**First run or cold-start run is slow.**
Trivy downloads its vulnerability database and misconfiguration bundle on the first run and after the database TTL expires. This is expected behavior.
Subsequent runs use the cached database and complete much faster. No action is required.

## Links

- [Trivy on GitHub](https://github.com/aquasecurity/trivy)
- [Trivy plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/trivy)
- [Trivy releases](https://github.com/aquasecurity/trivy/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Trivy is licensed under the [Apache License v2.0](https://github.com/aquasecurity/trivy/blob/main/LICENSE).
