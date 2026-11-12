# Review instructions

This is the Qlty CLI, a Rust workspace that people install and run on their own machines and in CI. It downloads and runs third-party linters, formatters and language runtimes. Changes reach users on their next upgrade, so backwards compatibility for existing `qlty.toml` configs, command-line flags and output formats matters, as does the integrity of anything the CLI downloads.

@review-common.md

## Also Important here

- A change that breaks existing `qlty.toml` configs, command-line flags, exit codes, machine-readable output such as JSON or SARIF, or how Qlty Cloud's hosted builds call the CLI.
- A download that isn't verified against a pinned checksum or signature, or a change that weakens that verification.
- A missing test for parsers, configuration loading, new plugins or changed plugin driver or parsing logic, or the installer and upgrade paths.

## Also do not report

- clippy and rustfmt findings
