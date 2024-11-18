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

Qlty CLI is licensed under the Functional Source License (FSL). More details are available in LICENSE.md.

## Acknowledgments

We would like to thank all of the developers of code quality tooling like linters and meta-linters as well as everyone who has contributed to the field of open source static analysis. Qlty CLI stands on the shoulders of decades of this excellent work.

Development of Qlty CLI is sponsored by [Qlty Software](https://qlty.sh).
