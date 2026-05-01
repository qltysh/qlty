# Black

[Black](https://github.com/psf/black) is a Python code formatter.

## Enabling Black

Enabling with the `qlty` CLI:

```bash
qlty plugins enable black
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "black"

# OR pin to a specific version
[[plugin]]
name = "black"
version = "X.Y.Z"
```

## Languages and file types

Black formats: Python (`.py`, `.pyi`).

## Troubleshooting

**black reports "cannot format" on a file that looks syntactically correct.**
black refuses to format files that contain syntax errors. Even a single mismatched parenthesis or invalid f-string will cause black to skip the file with a "cannot format" message.
Fix the syntax error first (running `python -m py_compile <file.py>` will surface it), then re-run `qlty fmt`.

**black and ruff/flake8 conflict on trailing commas or string quotes.**
black enforces double-quoted strings and magic-trailing-comma. If ruff or flake8 is also enabled with rules that conflict (for example `Q000` for single quotes, or `COM812` for trailing commas), the two tools will fight over the same lines.
Disable the conflicting ruff rules: add `"COM812"` and `"ISC001"` to ruff's `ignore` list, and remove any quote-style rules that contradict black's output.

## Links

- [Black on GitHub](https://github.com/psf/black)
- [Black plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/black)
- [Black releases](https://github.com/psf/black/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Black is licensed under the [The MIT License](https://github.com/psf/black/blob/main/LICENSE).
