# Qlty CLI

Qlty CLI is a multi-language code linter, auto-formatter, and security scanner.

Engineering teams use Qlty CLI for static analysis and auto-formatting of all of their code using a single tool with fast, consistent results.

As a Git-aware tool, Qlty CLI makes adopting linting into the development workflow easy by limiting results to only _new_ issues.

Qlty CLI is implemented in Rust, supported by [Qlty Software](https://qlty.sh), free to use, and is published under a [Fair Source](https://fair.io/) license.

## Features

- 🐞 Linting (for every programming language)
- 🖌️ Auto-formatting
- 🚨 Security scanning (IaC, SAST, SCA, and more)
- 📊 Complexity metrics and duplication
- 💩 Maintainability smells

## Installation

Qlty CLI is available for MacOS, Linux, and Windows.

### Install on MacOS or Linux

```bash
curl https://qlty.sh | bash
```

### Install on Windows

```bash
powershell -c "iwr https://qlty.sh | iex"
```

## Usage

Setup Qlty within a Git repository:

```bash
cd my_repo/
qlty init
```

View a sample of lint issues:

```bash
qlty check --sample=5
```

Auto-format the codebase:

```bash
qlty fmt --all
```

Scan for code smells like duplication:

```bash
qlty smells --all
```

Review a summary of code quality metrics:

```bash
qlty metrics --all --max-depth=2 --sort complexity --limit 10
```

## Plugins

Qlty CLI is powered by a set of 40+ plugins for static analysis tools like linters, auto-formatters, and security scanners. Plugin definitions can be found in the [`plugins/linters` directory](https://github.com/qltysh/qlty/tree/main/plugins/linters).

Creating a plugin can be as easy as writing a small plugin definition TOML file. If the tool has a custom output format (instead of a standard like [SARIF](https://sarifweb.azurewebsites.net/)), then writing a simple output parser in Rust is also needed.

We also happily accept requests for new plugins via [GitHub issues](https://github.com/qltysh/qlty/issues/new/choose).

## Configuration

Qlty CLI is configured using a `.qlty/qlty.toml` file in your Git repository. You can generate a default configuration with `qlty init` and then customize it.

Read our documentation about [configuration](https://docs.qlty.sh/analysis-configuration) for more information.

## Development

Developing on Qlty CLI requires a working [Rust toolchain](https://rustup.rs/).

```bash
cargo build
cargo test
```

## Contributing

1. Read the Guide to Contributing in CONTRIBUTING.md
2. Fork the repository and
3. Submit a pull request.

Contributions require agreeing to our [Contributor License Agreement](https://gist.github.com/brynary/00d59e41ffd852636a2f8a8f5f5aa69b) (CLA).

## Support

- Read the [documentation](https://docs.qlty.sh)
- Join our [Discord](https://qlty.sh/discord) chat
- [Community support](https://github.com/orgs/qltysh/discussions/categories/q-a) via GitHub Discussions
- [Feature requests](https://github.com/orgs/qltysh/discussions/categories/feedback) via GitHub Discussions
- [Bug reports](https://github.com/qltysh/qlty/issues/new/choose) via GitHub Issues
- [Plugin request](https://github.com/qltysh/qlty/issues/new/choose) via GitHub Issues

## License

Qlty CLI is licensed under the Qlty Source License, which is a Fair Source license that is a fork of the version of the Functional Source License (FSL). Qlty CLI is free to use, modify, and distribute in accordance with the FSL.

This codebase transitions into Open Source via a Delayed Open Source Publication (DOSP). More details are available in LICENSE.md.

Licenses for code incorporated into Qlty CLI can be found in the docs/licenses folder.

## Acknowledgments

We would like to thank all of the developers of code quality tooling like linters and meta-linters as well as everyone who has contributed to the field of open source static analysis. Qlty CLI stands on the shoulders of decades of this excellent work.

Development of Qlty CLI is sponsored by [Qlty Software](https://qlty.sh).
