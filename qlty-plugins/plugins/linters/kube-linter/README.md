# kube-linter

[kube-linter](https://github.com/stackrox/kube-linter) is a static analysis tool that checks Kubernetes YAML files and Helm charts for security misconfigurations and best-practice violations.

## Enabling kube-linter

Enabling with the `qlty` CLI:

```bash
qlty plugins enable kube-linter
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "kube-linter"

# OR pin to a specific version
[[plugin]]
name = "kube-linter"
version = "X.Y.Z"
```

## Auto-enabling

kube-linter will be automatically enabled by `qlty init` if a `.kube-linter.yaml` configuration file is present.

kube-linter will be automatically enabled by `qlty init` if a `.kube-linter.yaml` configuration file is present.

## Languages and file types

kube-linter analyzes: Kubernetes YAML manifests and Helm chart templates (`.yaml`, `.yml`).

## Configuration files

- [`.kube-linter.yaml`](https://docs.kubelinter.io/#/configuring-kubelinter)

Accepted filenames: `.kube-linter.yaml`, `.kube-linter.yml`.

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running kube-linter.

## Troubleshooting

**kube-linter reports no issues on Kubernetes manifests that have obvious problems.**
kube-linter runs checks against a configurable policy. If no `.kube-linter.yaml` configuration file is present, it uses the default check set, which may not include the check you are expecting.
Create a `.kube-linter.yaml` file to enable specific checks, or run `kube-linter checks list` to see all available checks and which ones are enabled by default.

**kube-linter fails with "unable to find Kubernetes schema" on custom resource definitions.**
kube-linter uses a bundled Kubernetes schema. Custom Resource Definitions (CRDs) are not in the bundled schema, so manifests for CRDs will fail schema validation.
Add `objectkindoverrides` in `.kube-linter.yaml` to skip schema validation for CRD-based resources, or disable the `invalid-target-ports` and similar checks that trigger on unknown schema types.

## Links

- [kube-linter on GitHub](https://github.com/stackrox/kube-linter)
- [kube-linter plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/kube-linter)
- [kube-linter releases](https://github.com/stackrox/kube-linter/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

kube-linter is licensed under the [Apache License 2.0](https://github.com/stackrox/kube-linter/blob/main/LICENSE).
