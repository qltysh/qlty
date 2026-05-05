# Osv-scanner

[Osv-scanner](https://github.com/google/osv-scanner) is a vulnerability scanner for your project.

## Enabling Osv-scanner

Enabling with the `qlty` CLI:

```bash
qlty plugins enable osv-scanner
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "osv-scanner"

# OR pin to a specific version
[[plugin]]
name = "osv-scanner"
version = "X.Y.Z"
```

## Auto-enabling

Osv-scanner will be automatically enabled by `qlty init` if a `osv-scanner.toml` configuration file is present.

## Configuration files

- [`osv-scanner.toml`](https://google.github.io/osv-scanner/configuration/#configure-osv-scanner)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Osv-scanner.

## Languages and file types

OSV-Scanner analyzes: dependency lockfiles such as `package-lock.json`, `Gemfile.lock`, `go.sum`, `requirements.txt`, and other package manifests.

## Troubleshooting

**Issues are not shown for a lockfile even though it contains vulnerable dependencies.**
Qlty drops all issues from a file when OSV-Scanner produces more than 100 results for it. Large monorepo lockfiles with many transitive dependencies (for example a root `package-lock.json` with hundreds of packages) can hit this limit.
Split large lockfiles across subdirectory packages, or add the affected file to `exclude_patterns` in `qlty.toml` and run OSV-Scanner directly on the command line instead.

**OSV-Scanner reports no vulnerabilities on a project you know has outdated packages.**
OSV-Scanner only checks lockfiles and SBOM files against the [OSV database](https://osv.dev/). If no lockfile is present (for example a project using `npm` without committing `package-lock.json`) it has nothing to scan.
Ensure your lockfile is committed to version control and is not listed in `.gitignore`.

## Links

- [Osv-scanner on GitHub](https://github.com/google/osv-scanner)
- [Osv-scanner plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/osv-scanner)
- [Osv-scanner releases](https://github.com/google/osv-scanner/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Osv-scanner is licensed under the [Apache License 2.0](https://github.com/google/osv-scanner/blob/main/LICENSE).
