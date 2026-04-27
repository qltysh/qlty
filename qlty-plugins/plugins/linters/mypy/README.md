# mypy

[mypy](https://github.com/python/mypy) is a static type checker for Python that finds type errors before runtime by analyzing type annotations.

## Enabling mypy

Enabling with the `qlty` CLI:

```bash
qlty plugins enable mypy
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "mypy"

# OR pin to a specific version
[[plugin]]
name = "mypy"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

mypy will be automatically enabled by `qlty init` when Python files are present.

## Languages and file types

mypy analyzes: Python (`.py`, `.pyi`).

## Configuration files

- [`mypy.ini`](https://mypy.readthedocs.io/en/stable/config_file.html)

Accepted filenames: `mypy.ini`, `.mypy.ini`. mypy also reads `[mypy]` sections in `pyproject.toml` and `setup.cfg`.

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running mypy.

## Troubleshooting

**mypy reports "Cannot find implementation or library stub for module named X".**
mypy requires type stub packages for third-party libraries. If a library does not include inline type information and no stub package is installed, mypy reports this error for every import from that module.
Install the relevant stubs (for example `pip install types-requests`) or add `ignore_missing_imports = true` to `mypy.ini` / `[tool.mypy]` in `pyproject.toml` to suppress errors for third-party libraries without stubs.

**mypy reports no errors on a file that has obvious type mismatches.**
mypy may be running in lenient mode (`--ignore-missing-imports`) or the file may not be in the `files` / `packages` list that mypy is configured to check. Files outside the configured scope are skipped silently.
Verify your `mypy.ini` configuration and run `mypy <file.py>` directly to confirm mypy sees the errors you expect.

## Links

- [mypy on GitHub](https://github.com/python/mypy)
- [mypy plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/mypy)
- [mypy releases](https://github.com/python/mypy/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

mypy is licensed under the [MIT License](https://github.com/python/mypy/blob/master/LICENSE).
