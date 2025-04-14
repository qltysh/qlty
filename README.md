<div align="left" id="top">
<a href="https://qlty.sh"><img alt="Qlty" src="https://cdn.brandfetch.io/idGrC4YgF4/theme/dark/idPHbenxLP.svg?c=1bxid64Mup7aczewSAYMX&t=1734797742010" height="75"></a>
</div>

## Universal linting, auto-formatting, maintainability, and security scanning

Qlty CLI is a multi-language code quality tool for linting, auto-formatting, maintainability, and security with support for 70+ static analysis tools for 40+ languages and technologies.

With Qlty CLI, polyglot team can take advantage of the best code quality static analysis with fast, consistent, and unified results through a single tool. Configuration is done through a simple `.qlty/qlty.toml` file in your repository, which can be auto-generated based on the languages you use.

The Qlty CLI is **completely free for all use**, including for commercial projects, with no limits on contributors.

[![Maintainability](https://qlty.sh/badges/f983cb35-d208-4d2f-8872-03fb3e1205de/maintainability.svg)](https://qlty.sh/gh/qltysh/projects/qlty)
[![Code Coverage](https://qlty.sh/badges/f983cb35-d208-4d2f-8872-03fb3e1205de/test_coverage.svg)](https://qlty.sh/gh/qltysh/projects/qlty)
[![Unit Tests](https://github.com/qltysh/qlty/actions/workflows/cli.yml/badge.svg)](https://github.com/qltysh/qlty/actions/workflows/cli.yml)
[![Latest release](https://img.shields.io/github/v/release/qltysh/qlty)](https://github.com/qltysh/qlty/releases)
[![docs.qlty.sh](https://img.shields.io/badge/docs-docs.qlty.sh-08b2b7)](https://docs.qlty.sh)
[![GitHub stars](https://img.shields.io/github/stars/qltysh/qlty)](https://github.com/qltysh/qlty)

---

## 📖 Table of Contents

- [✨ Key Features](#-key-features)
- [🚀 Quick Start](#-quick-start)
  - [📦 Installation](#-quick-start)
  - [Setting up Qlty in a new repository](#setting-up-qlty-in-a-new-repository)
  - [Linting and auto-formatting](#linting-and-auto-formatting)
  - [Maintinability and quality metrics](#maintainability-and-quality-metrics)
  - [Configuration](#configuration)
- [🧹 Available Linters](#-available-linters)
- [📊 Code Quality Metrics](#-code-quality-metrics)
- [🖥️ System Requirements](#%EF%B8%8F-system-requirements)
- [🛟 Help or Feedback](#-help-or-feedback)
- [🧑‍💻 Contributing](#-contributing)
- [⚖️ License](#️-license)

---

## ✨ Key Features

### What We Do

|     | Feature           | Advantage                                            |
| --- | ----------------- | ---------------------------------------------------- |
| 🐛  | Linting           | Comprehensive language support in one tool           |
| 🖌️  | Auto-formatting   | Consistent code style everywhere                     |
| 💩  | Maintainability   | Code smells like copy-paste detection and complexity |
| 🚨  | Security scanning | SAST, SCA, secret detection, IaC analysis, and more  |
| 🚦  | Code coverage     | Total coverage and diff coverage                     |
| 📊  | Quality metrics   | Complexity, duplication, LOC, etc.                   |

### How We Do It

|     | Feature                 | Advantage                                                                   |
| --- | ----------------------- | --------------------------------------------------------------------------- |
| 🌲  | Git-aware               | Focus on newly introduced quality issues                                    |
| ⚡  | Auto-initialization     | Get up and running in two minutes                                           |
| ✅  | Autofixes               | Including tool-generated and AI-generated fixes                             |
| ⚙️  | Config as code          | Version controlled with maximum flexibility                                 |
| 🏎️  | Caching and concurrency | The absolute fastest way to run static analysis                             |
| 🪝  | Git hooks               | Integrate with pre-commit and pre-push hooks                                |
| 🤖  | Pull request reviews    | Automated feedback in comments and statuses                                 |
| 🌐  | Runs anywhere           | Mac, Linux, and Windows with no dependency on Docker                        |
| 🦀  | Written in Rust         | Fast execution and easy to contribute                                       |
| 🎁  | 100% free               | Including for commercial projects, with no contributor limits               |
| ⚖️  | Fair Source             | Public on GitHub with delayed open source publication (DOSP). PRs accepted! |

💡 Learn more in the [Documentation](https://docs.qlty.sh/).

### Qlty Software: Code quality and coverage done right

Qlty CLI is part of Qlty Software's comprehensive platform for code quality. Bring code quality into every step of your software development workflow with:

- [Qlty CLI](https://github.com/qltysh/qlty) -- Polyglot code quality CLI written in Rust
- [Qlty Cloud](https://qlty.sh) -- Automated code review and quality trends
- [Visual Studio Code Extension](https://github.com/qltysh/qlty-vscode) -- Linting and auto-formatting in your IDE
- [GitHub Action](https://github.com/qltysh/qlty-action) -- Run Qlty CLI within your CI workflows
- [Chrome and Firefox Extension](https://github.com/qltysh/qlty-browser) -- Adds code coverage data to GitHub.com

---

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

> [!NOTE]
> The Qlty CLI does _not_ use Docker to run linters. By running linters natively, we achieve maximum performance. The Docker image is provided for situations where running the CLI as a containers is preferred over running it as a native binary.

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

---

## 🧹 Available Linters

Over 20,000 code quality rules are available via the Qlty CLI through its 60+ linter plugins.

To enable new plugins by adding them to your `.qlty/qlty.toml` file run:

```sh
qlty plugins enable <NAME>
```

| Technology     | Available code quality tools                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All files      | [gitleaks](https://gitleaks.io/), [ripgrep](https://github.com/BurntSushi/ripgrep), [semgrep](https://semgrep.dev), [trivy](https://trivy.dev), [trufflehog](https://trufflesecurity.com/trufflehog), [vale](https://vale.sh/)                                                                                                                                                                                                                                                                                                                     |
| Apex           | [pmd](https://pmd.github.io/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| C#             | [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells)                                                                                                                                                                                                                                                                                                                                                                                                            |
| C/C++          | [osv-scanner](https://google.github.io/osv-scanner/), [trivy](https://trivy.dev)                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| CloudFormation | [checkov](https://www.checkov.io/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| CoffeeScript   | [coffeelint](https://github.com/clutchski/coffeelint)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| CSS            | [biome](https://biomejs.dev/), [prettier](https://prettier.io/), [stylelint](https://stylelint.io/)                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| Dart           | [osv-scanner](https://google.github.io/osv-scanner/), [trivy](https://trivy.dev)                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| Docker         | [checkov](https://www.checkov.io/), [dockerfmt](https://github.com/reteps/dockerfmt), [hadolint](https://github.com/hadolint/hadolint), [radarlint](https://github.com/qltysh/radarlint), [trivy](https://trivy.dev)                                                                                                                                                                                                                                                                                                                               |
| Dotenv         | [dotenv-linter](https://dotenv-linter.github.io/#/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| Elixer         | [osv-scanner](https://google.github.io/osv-scanner/), [trivy](https://trivy.dev)                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| Erlang         | [osv-scanner](https://google.github.io/osv-scanner/), [trivy](https://trivy.dev)                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| GitHub Actions | [actionlint](https://rhysd.github.io/actionlint/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Go             | [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [gofmt](https://pkg.go.dev/cmd/gofmt), [golangci-lint](https://golangci-lint.run/), [osv-scanner](https://google.github.io/osv-scanner/), [radarlint](https://github.com/qltysh/radarlint)                                                                                                                                                                                                                |
| GraphQL        | [prettier](https://prettier.io/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| HTML           | [prettier](https://prettier.io/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| Java           | [checkstyle](https://checkstyle.org/), [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [google-java-format](https://github.com/google/google-java-format), [osv-scanner](https://google.github.io/osv-scanner/), [pmd](https://pmd.github.io/), [radarlint](https://github.com/qltysh/radarlint), [trivy](https://trivy.dev)                                                                                                                              |
| JavaScript     | [biome](https://biomejs.dev/), [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [eslint](https://eslint.org/), [knip](https://knip.dev/), [osv-scanner](https://google.github.io/osv-scanner/), [oxc](https://oxc.rs/), [prettier](https://prettier.io/), [radarlint](https://github.com/qltysh/radarlint), [trivy](https://trivy.dev)                                                                                                                     |
| JSON           | [biome](https://biomejs.dev/), [prettier](https://prettier.io/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Kotlin         | [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [osv-scanner](https://google.github.io/osv-scanner/), [radarlint](https://github.com/qltysh/radarlint), [trivy](https://trivy.dev)                                                                                                                                                                                                                                                                        |
| Kubernetes     | [kube-linter](https://docs.kubelinter.io/#/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Markdown       | [markdownlint](https://github.com/DavidAnson/markdownlint), [prettier](https://prettier.io/)                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| OpenAPI        | [redocly](https://redocly.com/docs/cli)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| PHP            | [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [osv-scanner](https://google.github.io/osv-scanner/), [php-codesniffer](https://github.com/squizlabs/PHP_CodeSniffer), [php-cs-fixer](https://cs.symfony.com/), [phpstan](https://phpstan.org/), [radarlint](https://github.com/qltysh/radarlint), [trivy](https://trivy.dev)                                                                                                                             |
| Prisma         | [prisma](https://github.com/prisma/prisma)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Python         | [bandit](https://bandit.readthedocs.io/en/latest/), [black](https://github.com/psf/black), [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [flake8](https://flake8.pycqa.org/en/latest/), [mypy](https://www.mypy-lang.org/), [osv-scanner](https://google.github.io/osv-scanner/), [radarlint](https://github.com/qltysh/radarlint), [ruff](https://docs.astral.sh/ruff/), [trivy](https://trivy.dev)                                                    |
| R              | [osv-scanner](https://google.github.io/osv-scanner/), [trivy](https://trivy.dev)                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| Ruby           | [brakeman](https://brakemanscanner.org/), [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [osv-scanner](https://google.github.io/osv-scanner/), [radarlint](https://github.com/qltysh/radarlint), [reek](https://github.com/troessner/reek), [rubocop](https://docs.rubocop.org/rubocop/1.75/index.html), [ruby-stree](https://github.com/ruby-syntax-tree/syntax_tree), [standardrb](https://github.com/standardrb/standard), [trivy](https://trivy.dev) |
| Rust           | [clippy](https://rust-lang.github.io/rust-clippy/), [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [osv-scanner](https://google.github.io/osv-scanner/), [rustfmt](https://rust-lang.github.io/rustfmt/?version=v1.8.0&search=), [trivy](https://trivy.dev)                                                                                                                                                                                              |
| SASS           | [prettier](https://prettier.io/), [stylelint](https://stylelint.io/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| Scala          | [radarlint](https://github.com/qltysh/radarlint)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| Shell          | [shellcheck](https://www.shellcheck.net/), [shfmt](https://pkg.go.dev/mvdan.cc/sh/v3)                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| SQL            | [sqlfluff](https://sqlfluff.com/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Swift          | [swiftlint](https://realm.github.io/SwiftLint/)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Terraform      | [checkov](https://www.checkov.io/), [osv-scanner](https://google.github.io/osv-scanner/), [radarlint](https://github.com/qltysh/radarlint), [tflint](https://github.com/terraform-linters/tflint), [trivy](https://trivy.dev)                                                                                                                                                                                                                                                                                                                      |
| TypeScript     | [biome](https://biomejs.dev/), [complexity](https://github.com/qltysh/qlty/tree/main/qlty-smells), [duplication](https://github.com/qltysh/qlty/tree/main/qlty-smells), [eslint](https://eslint.org/), [knip](https://knip.dev/), [oxc](https://oxc.rs/), [prettier](https://prettier.io/)                                                                                                                                                                                                                                                         |
| YAML           | [prettier](https://prettier.io/), [trivy](https://trivy.dev), [yamllint](https://github.com/adrienverge/yamllint)                                                                                                                                                                                                                                                                                                                                                                                                                                  |

The [full list of plugins](https://github.com/qltysh/qlty/tree/main/qlty-plugins/plugins/linters) is available on GitHub.

---

## 📊 Code Quality Metrics

The Qlty CLI calculates a variety of code quality metrics which are available through the `qlty metrics` subcommand and as trends on [Qlty Cloud](https://qlty.sh).

| Metric                 | Category        | Definition                                                        |
| ---------------------- | --------------- | ----------------------------------------------------------------- |
| Duplication Density    | Duplication     | Duplicated Lines divided by Code Lines                            |
| Duplicated Lines       | Duplication     | The number of lines that are duplicated                           |
| Duplicated Blocks      | Duplication     | The number of contiguous spans of duplicated lines                |
| Complexity Density     | Complexity      | Complexity divided by Code Lines                                  |
| Complexity             | Complexity      | The count of Cognitive Complexity                                 |
| Cyclomatic Complexity  | Complexity      | The count of Cyclomatic (McCabe's) Complexity                     |
| Technical Debt         | Maintainability | The estimated amount of time needed to resolve the code smells    |
| Technical Debt Ratio   | Maintainability | Technical Debt divided by estimated implementation time           |
| Maintainability Rating | Maintainability | Technical Debt Ratio expressed as a letter rating                 |
| Security Issues        | Security        | Count of security issues                                          |
| Security Rating        | Security        | A letter rating based on security issues and their severity level |
| Covered Lines          | Coverage        | Count of lines covered by automated tests                         |
| Uncovered Lines        | Coverage        | Count of lines that could be covered but are not                  |
| Line Coverage          | Coverage        | Covered Lines divided by Coverd Lines plus Uncovered Lines        |
| Diff Coverage          | Coverage        | The Line Coverage of the new and changes lines of a Git diff      |
| Coverage Rating        | Coverage        | Line Coverage expressed as a letter rating                        |
| Classes                | Size            | Count of classes                                                  |
| Fields                 | Size            | Count of unique fields                                            |
| Functions              | Size            | Count of functions or methods                                     |
| Code Files             | Size            | Count of programming language files                               |
| Lines                  | Size            | Count of all lines including blanks and comments                  |
| Lines of Code          | Size            | Count of lines that are not blank or comments                     |
| Comment Lines          | Size            | Count of comment lines                                            |
| Comments Density       | Size            | Comment Lines divided by Lines                                    |
| Issues Count           | Issues          | Count of static analysis issues                                   |

Quality metrics are available for C#, Go, Java, JavaScript, Kotlin, PHP, Python, Ruby, Rust, and TypeScript.

---

## 🖥️ System Requirements

Qlty CLI is available for MacOS, Linux, and Windows on x86 and ARM platforms.

### Additional requirements for PHP linters

Certain PHP linters require a working installation of PHP available in your `$PATH`. To install PHP, use [Homebrew](https://brew.sh/) or an alternative method.

---

## 🛟 Help or Feedback

- Read the [documentation](https://docs.qlty.sh)
- Join our [Discord](https://qlty.sh/discord) chat
- [Community support](https://github.com/orgs/qltysh/discussions/categories/q-a) via GitHub Discussions
- [Feature requests](https://github.com/orgs/qltysh/discussions/categories/feedback) via GitHub Discussions
- [Bug reports](https://github.com/qltysh/qlty/issues/new/choose) via GitHub Issues
- [Plugin request](https://github.com/qltysh/qlty/issues/new/choose) via GitHub Issues

---

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

---

## ⚖️ License

Qlty CLI is published under a [Fair Source](https://fair.io/) license. As Fair Source, the Qlty CLI is free to use (including in commercial contexts), modify, and distribute in accordance with its license.

This code is made available under the Business Source License 1.1 (BSL) and transitions into Open Source via a Delayed Open Source Publication (DOSP). More details are available in LICENSE.md.

### Acknowledgements

We would like to thank all of the developers of code quality tooling like linters and meta-linters as well as everyone who has contributed to the field of open source static analysis. Qlty CLI stands on the shoulders of decades of this excellent work.

Licenses for code incorporated into Qlty CLI can be found in the docs/licenses folder.

---
