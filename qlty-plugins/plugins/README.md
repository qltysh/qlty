# Qlty Plugins

This is the official repository containing Qlty's integrations with linters, formatters, and other static analysis tools. By default, Qlty users import this repository as a source via the following snippet in `.qlty/qlty.toml`:

```toml
[sources.default]
repository = "https://github.com/qltysh/qlty"
tag = "v0.13.0"
```

Plugins marked as `recommended = true` will be automatically enabled when setting up Qlty using the init command:

```bash
qlty init
```

Additional plugins can be enabled by editing `.qlty/qlty.toml` or by using the CLI:

```bash
qlty plugins enable shellcheck@0.9.0
```

## Development

### Dependencies

- Node.js v21+
- Qlty CLI available as `qlty` in `$PATH`

### Set up

```bash
npm install
npm run test
```

### Release

```bash
export TAG_NAME="v0.1.0"
git tag $TAG_NAME
git push --tags
```

## Acknowledgements

Thank you to [trunk-io/plugins](https://github.com/trunk-io/plugins) for inspiration.
