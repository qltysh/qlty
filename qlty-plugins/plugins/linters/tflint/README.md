# TFLint

[TFLint](https://github.com/terraform-linters/tflint) is a pluggable Terraform linter that finds possible errors, enforces best practices, and warns about deprecated syntax that `terraform validate` does not catch.

## Enabling TFLint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable tflint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "tflint"

# OR pin to a specific version
[[plugin]]
name = "tflint"
version = "X.Y.Z"
```

## Auto-enabling

TFLint will be automatically enabled by `qlty init` if a `.tflint.hcl` configuration file is present.

TFLint will be automatically enabled by `qlty init` if a `.tflint.hcl` configuration file is present.

## Languages and file types

TFLint analyzes: HCL Terraform configuration files (`.tf`, `.tfvars`).

## Configuration files

- [`.tflint.hcl`](https://github.com/terraform-linters/tflint/blob/master/docs/user-guide/config.md)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running TFLint.

## Troubleshooting

**tflint reports no issues on Terraform code that has obvious problems.**
tflint requires provider-specific rule plugins (for example `tflint-ruleset-aws`) to lint provider-specific resources. Without a plugin, tflint only checks generic Terraform syntax.
Create a `.tflint.hcl` configuration file that installs and enables the relevant ruleset plugin for your cloud provider, then run `tflint --init`.

**tflint fails with "Failed to initialize plugins" on first run.**
tflint downloads rule plugins from GitHub on first use. In air-gapped environments or CI with restricted outbound network, the download fails.
Pre-download the plugin bundle into `.tflint.d/plugins/` in your project and configure `.tflint.hcl` to use the local path with `plugin_dir = ".tflint.d/plugins"`.

## Links

- [TFLint on GitHub](https://github.com/terraform-linters/tflint)
- [TFLint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/tflint)
- [TFLint releases](https://github.com/terraform-linters/tflint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

TFLint is licensed under the [Mozilla Public License 2.0](https://github.com/terraform-linters/tflint/blob/master/LICENSE).
