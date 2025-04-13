# Qlty CLI

## Universal linting, auto-formatting, maintainability, and security scanning

[Qlty CLI](https://qlty.sh) is a multi-language code quality CLI for linting, auto-formatting, maintainability, and security with support for 50+ static analysis tools.

With Qlty CLI, polyglot team can take advantage of the best code quality static analysis with fast, consistent, and unified results through a single tool. Configuration is done through a simple `.qlty/qlty.toml` file in your repository, which can be auto-generated based on the languages you use.

The Qlty CLI and this Visual Studio Code extension are **completely free for all use**, including for commercial projects, with no limits on contributors.

## 📖 Table of Contents

- [✨ Key Features](#-key-features)
- [🚀 Quick Start](#-quick-start)
  - [📦 Installation](#-quick-start)
  - [Setting up Qlty in a new repository](#setting-up-qlty-in-a-new-repository)
  - [Linting and auto-formatting](#linting-and-auto-formatting)
  - [Maintinability and quality metrics](#maintainability-and-quality-metrics)
  - [Configuration](#configuration)
- [🧹 Available Linters](#-available-linters)
- [🖥️ System Requirements](#%EF%B8%8F-system-requirements)
- [🛟 Help or Feedback](#-help-or-feedback)
- [🧑‍💻 Contributing](#-contributing)
- [⚖️ License](#️-license)

## ✨ Key Features

### What We Do

| | Feature | Advantage |
|-----|-----|-----|
| 🐛 | Linting | Comprehensive language support in one tool |
| 🖌️ | Auto-formatting | Consistent code style everywhere |
| 💩 | Maintainability | Code smells like copy-paste detection and complexity |
| 🚨 | Security scanning | SAST, SCA, secret detection, IaC analysis, and more |
| 🚦 | Code coverage | Total coverage and diff coverage |
| 📊 | Quality metrics | Complexity, duplication, LOC, etc. |

### How We Do It

| | Feature | Advantage |
|-----|-----|-----|
| 🌲 | Git-aware | Focus on newly introduced quality issues |
| ⚡ | Auto-initialization | Get up and running in two minutes |
| ✅ | Autofixes | Including tool-generated and AI-generated fixes |
| ⚙️ | Config as code | Version controlled with maximum flexibility |
| 🏎️ | Caching and concurrency | The absolute fastest way to run static analysis |
| 🪝 | Git hooks | Integrate with pre-commit and pre-push hooks |
| 🤖 | Pull request reviews | Automated feedback in comments and statuses |
| 🌐 | Runs anywhere | Mac, Linux, and Windows with no dependency on Docker |
| 🦀 | Written in Rust | Fast execution and easy to contribute |
| 🎁 | 100% free | Including for commercial projects, with no contributor limits |
| ⚖️ | Fair Source | Public on GitHub with delayed open source publication (DOSP). PRs accepted! |

💡 Learn more in the [Documentation](https://docs.qlty.sh/).

### Qlty Software: Code quality and coverage done right

Qlty CLI is part of Qlty Software's comprehensive platform for code quality. Bring code quality into every step of your software development workflow with:

- [Qlty CLI](https://github.com/qltysh/qlty) -- Polyglot code quality CLI written in Rust 
- [Qlty Cloud](https://qlty.sh) -- Automated code review and quality trends
- [Visual Studio Code Extension](https://github.com/qltysh/qlty-vscode) -- Linting and auto-formatting in your IDE
- [GitHub Action](https://github.com/qltysh/qlty-action) -- Run Qlty CLI within your CI workflows
- [Chrome and Firefox Extension](https://github.com/qltysh/qlty-browser) -- Adds code coverage data to GitHub.com

## 🚀 Quick Start

### Installation

The fastest way to install Qlty CLI is using our installer scripts which install our native binaries:

```bash
# Install on MacOS or Linux
curl https://qlty.sh | bash 


# Install on Windows
powershell -c "iwr https://qlty.sh | iex"
```

We also package the CLI as a [Docker image](https://github.com/qltysh/qlty/pkgs/container/qlty) on GitHub Container Registry (GHCR).

_Note:_ To achieve the fastest performance, the Qlty CLI does _not_ depend on running linters via Docker. The Docker packages are for cases where running Docker containers is preferred over native binaries.

### Setting up Qlty in a new repository

Setup Qlty within a Git repository:

```bash
cd my_repo/
qlty init
```

### Linting and auto-formatting

View a sample of lint issues:

```bash
qlty check --sample=5
```

Auto-format the codebase:

```bash
qlty fmt --all
```

### Maintainability and quality metrics
Scan for code smells like duplication:

```bash
qlty smells --all
```

Review a summary of code quality metrics:

```bash
qlty metrics --all --max-depth=2 --sort complexity --limit 10
```

### Configuration

Qlty CLI is configured using a `.qlty/qlty.toml` file in your Git repository. You can generate a default configuration with `qlty init` and then customize it.

Read our documentation about [configuration](https://docs.qlty.sh/analysis-configuration) for more information.

## 🧹 Available Linters

Over 20,000 code quality rules are available via the Qlty CLI through its 60+ linter plugins.

To enable new plugins by adding them to your `.qlty/qlty.toml` file run:

```sh
qlty plugins enable <NAME>
```

| Technology | Available code quality tools |
|-|-|
| All files | gitleaks, ripgrep, semgrep, trivy, trufflehog, vale |
| Apex | pmd |
| C/C++ | osv-scanner, trivy |
| CloudFormation | checkov |
| CoffeeScript | coffeelint |
| CSS | biome, prettier, stylelint |
| Dart | osv-scanner, trivy |
| Docker | checkov, dockerfmt, hadolint, radarlint, trivy |
| Dotenv | dotenv-linter |
| Elixer | osv-scanner, trivy |
| Erlang | osv-scanner, trivy |
| GitHub Actions | actionlint |
| Go | gofmt, golangci-lint, osv-scanner, radarlint |
| GraphQL | prettier |
| HTML | prettier |
| Java | checkstyle, google-java-format, osv-scanner, pmd, radarlint, trivy |
| JavaScript | biome, eslint, knip, osv-scanner, oxc, prettier, radarlint, trivy |
| JSON | biome, prettier |
| Kotlin | osv-scanner, radarlint, trivy |
| Kubernetes | kube-linter |
| Markdown | markdownlint, prettier |
| OpenAPI | redocly |
| PHP | osv-scanner, php-codesniffer, php-cs-fixer, phpstan, radarlint, trivy |
| Prisma | prisma |
| Python | bandit, black, flake8, mypy, osv-scanner, radarlint, ruff, trivy |
| R | osv-scanner, trivy |
| Ruby | brakeman, osv-scanner, radarlint, reek, rubocop, ruby-stree, standardrb, trivy |
| Rust | clippy, osv-scanner, rustfmt, trivy |
| SASS | prettier, stylelint |
| Scala | radarlint |
| Shell | shellcheck, shfmt |
| SQL | sqlfluff |
| Swift | swiftlint | 
| Terraform | checkov, osv-scanner, radarlint, radarlint, tflint, trivy, trivy |
| TypeScript | biome, eslint, knip, oxc, prettier |
| YAML | prettier, trivy, yamllint |

The [full list of plugins](https://github.com/qltysh/qlty/tree/main/qlty-plugins/plugins/linters) is available on GitHub.

## 🖥️ System Requirements

Qlty CLI is available for MacOS, Linux, and Windows on x86 and ARM platforms.

### Additional requirements for PHP linters

Certain PHP linters require a working installation of PHP available in your `$PATH`. To install PHP, use [Homebrew](https://brew.sh/) or an alternative method.

## 🛟 Help or Feedback

- Read the [documentation](https://docs.qlty.sh)
- Join our [Discord](https://qlty.sh/discord) chat
- [Community support](https://github.com/orgs/qltysh/discussions/categories/q-a) via GitHub Discussions
- [Feature requests](https://github.com/orgs/qltysh/discussions/categories/feedback) via GitHub Discussions
- [Bug reports](https://github.com/qltysh/qlty/issues/new/choose) via GitHub Issues
- [Plugin request](https://github.com/qltysh/qlty/issues/new/choose) via GitHub Issues

## 🧑‍💻 Contributing

### Adding plugins

Creating a plugin can be as easy as writing a small plugin definition TOML file. If the tool has a custom output format (instead of a standard like [SARIF](https://sarifweb.azurewebsites.net/)), then writing a simple output parser in Rust is also needed.

We also happily accept requests for new plugins via [GitHub issues](https://github.com/qltysh/qlty/issues/new/choose).

### Developing the CLI

Developing on Qlty CLI requires a working [Rust toolchain](https://rustup.rs/) and adheres to the standard Rust development process:

```bash
git clone https://github.com/qltysh/qlty.git
cd qlty
cargo build
cargo test
```

### More information

More information about how to contribute can be found in CONTRIBUTING.md.

Reports of security vulnerabilities should be handled with the process outlined in SECURITY.md.

## ⚖️ License

Qlty CLI is published under a [Fair Source](https://fair.io/) license. As Fair Source, the Qlty CLI is free to use (including in commercial contexts), modify, and distribute in accordance with its license.

This code is made available under the Business Source License 1.1 (BSL) and transitions into Open Source via a Delayed Open Source Publication (DOSP). More details are available in LICENSE.md.

### Acknowledgements

We would like to thank all of the developers of code quality tooling like linters and meta-linters as well as everyone who has contributed to the field of open source static analysis. Qlty CLI stands on the shoulders of decades of this excellent work.

Licenses for code incorporated into Qlty CLI can be found in the docs/licenses folder.