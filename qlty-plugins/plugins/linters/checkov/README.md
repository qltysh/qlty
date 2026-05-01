# Checkov

[Checkov](https://github.com/bridgecrewio/checkov) is a static code analysis tool for infrastructure as code (IaC) and also a software composition analysis (SCA) tool for images and open source packages.

## Enabling Checkov

Enabling with the `qlty` CLI:

```bash
qlty plugins enable checkov
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "checkov"

# OR pin to a specific version
[[plugin]]
name = "checkov"
version = "X.Y.Z"
```

## Auto-enabling

Checkov will be automatically enabled by `qlty init` if a `.checkov.yaml` configuration file is present.

## Configuration files

- [`.checkov.yaml`](https://github.com/bridgecrewio/checkov?tab=readme-ov-file#configuration-using-a-config-file)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Checkov.

## Languages and file types

Checkov analyzes: Dockerfiles, Terraform HCL (`.tf`, `.tfvars`), YAML (`.yml`, `.yaml`), and JSON (`.json`) configuration files.

## Troubleshooting

**"Unsupported instruction" warnings for Dockerfile.**
Checkov does not recognize some ARG-based or parameterized instructions (e.g. `TZDATA="${TZDATA_VERSION}"`) used in multi-stage Dockerfiles.
These warnings are informational — the rest of the Dockerfile is still analyzed. No action is required unless you need Checkov to evaluate those specific instructions.

## Links

- [Checkov on GitHub](https://github.com/bridgecrewio/checkov)
- [Checkov plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/checkov)
- [Checkov releases](https://github.com/bridgecrewio/checkov/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Checkov is licensed under the [Apache License v2.0](https://github.com/bridgecrewio/checkov/blob/main/LICENSE).
