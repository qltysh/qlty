# Terraform

[Terraform](https://github.com/hashicorp/terraform) is HashiCorp's infrastructure-as-code tool. Under Qlty, the Terraform plugin runs `terraform validate` to catch configuration errors and `terraform fmt` to enforce canonical HCL formatting.

## Enabling Terraform

Enabling with the `qlty` CLI:

```bash
qlty plugins enable terraform
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "terraform"

# OR pin to a specific version
[[plugin]]
name = "terraform"
version = "X.Y.Z"
```

## Auto-enabling

Terraform will be automatically enabled by `qlty init` when Terraform (`.tf`) files are present.

Terraform will be automatically enabled by `qlty init` when Terraform files are present.

## Languages and file types

The Terraform plugin analyzes: HCL Terraform configuration files (`.tf`, `.tfvars`).

## Troubleshooting

**Terraform reports "Error: Backend initialization required" and does not run.**
`terraform validate` (which Qlty uses) requires the Terraform backend and providers to be initialized. If `.terraform/` is not present, validation fails.
Run `terraform init` in the module directory first. In CI, cache the `.terraform/` directory across runs to avoid re-initializing on every check.

**Terraform reports no issues on a module with obvious misconfigurations.**
Qlty runs `terraform validate`, which only checks syntax and internal consistency. It does not catch semantic misconfigurations (for example incorrect resource names or missing required fields) that only a plan would reveal.
Use tflint with a provider ruleset (see the tflint plugin) for deeper static analysis of provider-specific resource configurations.

## Links

- [Terraform on GitHub](https://github.com/hashicorp/terraform)
- [Terraform plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/terraform)
- [Terraform releases](https://github.com/hashicorp/terraform/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Terraform is licensed under the [Business Source License 1.1](https://github.com/hashicorp/terraform/blob/main/LICENSE).
