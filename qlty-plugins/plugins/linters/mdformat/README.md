# Mdformat

[Mdformat](https://github.com/hukkin/mdformat) is an opinionated CommonMark compliant Markdown formatter.

## Enabling Mdformat

Enabling with the `qlty` CLI:

```bash
qlty plugins enable mdformat
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[plugins.enabled]
mdformat = "latest"

# OR enable a specific version
[plugins.enabled]
mdformat = "X.Y.Z"
```

## Configuration files

- [`.mdformat.toml`](https://mdformat.readthedocs.io/en/stable/users/configuration_file.html)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Mdformat.

## Links

- [Mdformat on GitHub](https://github.com/hukkin/mdformat)
- [Mdformat documentation](https://mdformat.readthedocs.io/)
- [Mdformat plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/mdformat)
- [Mdformat on PyPI](https://pypi.org/project/mdformat/)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/plugins/linters)

## License

Mdformat is licensed under the [MIT License](https://github.com/hukkin/mdformat/blob/master/LICENSE).
