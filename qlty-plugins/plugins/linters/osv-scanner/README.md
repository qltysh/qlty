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
[plugins.enabled]
osv-scanner = "latest"

# OR enable a specific version
[plugins.enabled]
osv-scanner = "X.Y.Z"
```

## Auto-enabling

Osv-scanner will be automatically enabled by `qlty init` if a `osv-scanner.toml` configuration file is present.

## Configuration files

- [`osv-scanner.toml`](https://google.github.io/osv-scanner/configuration/#configure-osv-scanner)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Osv-scanner.

## Links

- [Osv-scanner on GitHub](https://github.com/google/osv-scanner)
- [Osv-scanner plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/osv-scanner)
- [Osv-scanner releases](https://github.com/google/osv-scanner/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Osv-scanner is licensed under the [Apache License 2.0](https://github.com/google/osv-scanner/blob/main/LICENSE).
