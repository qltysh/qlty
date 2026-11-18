# Review instructions

This is the Qlty CLI, a Rust workspace that people install and run on their own machines and in CI. It downloads and runs third-party linters, formatters and language runtimes. Changes reach users on their next upgrade, so backwards compatibility for existing `qlty.toml` configs, command-line flags and output formats matters, as does the integrity of anything the CLI downloads.

@review-common.md

## Also Important here

- A change that breaks existing `qlty.toml` configs, command-line flags, exit codes, machine-readable output such as JSON or SARIF, or how Qlty Cloud's hosted builds call the CLI.
- A download that isn't verified against a pinned checksum or signature, or a change that weakens that verification.
- A change that renames or deletes `.github/workflows/build.yml`, moves its `actions/attest-build-provenance` step into another workflow, a reusable workflow or a composite action, or lets that step run on a ref other than `refs/heads/main` (new triggers, tags, other branches, or a looser `if:` on the `compile` job). qltysh/qlty-action and the release promote job accept a CLI archive only if it was attested by `build.yml@refs/heads/main`, so this breaks every qlty-action user at the next release unless the action, the CLI upgrade check and the promote job change first. Also flag a change that weakens `.github/workflows/attestation_signer.yml`, which checks this in CI.
- A missing test for parsers, configuration loading, new plugins or changed plugin driver or parsing logic, or the installer and upgrade paths.

## Also do not report

- clippy and rustfmt findings
