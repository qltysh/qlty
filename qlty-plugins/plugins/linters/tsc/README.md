# tsc

[tsc](https://github.com/microsoft/TypeScript) is the TypeScript compiler. Under Qlty, the `tsc` plugin runs the compiler in type-check mode to surface type errors without emitting output files.

## Enabling tsc

Enabling with the `qlty` CLI:

```bash
qlty plugins enable tsc
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "tsc"

# OR pin to a specific version
[[plugin]]
name = "tsc"
version = "X.Y.Z"
```

## Auto-enabling

tsc will be automatically enabled by `qlty init` if a `tsconfig.json` configuration file is present.

tsc will be automatically enabled by `qlty init` if a `tsconfig.json` configuration file is present.

## Languages and file types

tsc analyzes: TypeScript (`.ts`, `.tsx`). It requires a `tsconfig.json` and a `package.json` in the project root.

## Configuration files

- [`tsconfig.json`](https://www.typescriptlang.org/tsconfig)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running tsc.

## Troubleshooting

**"The enabled plugin version is 'known_good', but the known good version is unknown: tsc" error.**
Unlike most plugins, tsc does not have a bundled known-good version because it should match the TypeScript version in your project. You must specify an explicit version in `qlty.toml`.
Add `version = "<your-typescript-version>"` to the `[[plugin]]` block for tsc (for example, `version = "5.9.3"`), matching the TypeScript version in your `package.json`.

**tsc finds no errors even though the project has type errors.**
tsc runs from the directory containing `package.json` and uses the project's `tsconfig.json`. If the tsconfig excludes the files with errors, or if `noEmit` is not set, tsc may not report what you expect.
Verify that `tsconfig.json` includes the files you want to check and contains `"noEmit": true`. Run `npx tsc --noEmit` directly in the project root to confirm type errors are visible before Qlty runs.

## Links

- [TypeScript on GitHub](https://github.com/microsoft/TypeScript)
- [tsc plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/tsc)
- [TypeScript releases](https://github.com/microsoft/TypeScript/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

TypeScript is licensed under the [Apache License 2.0](https://github.com/microsoft/TypeScript/blob/main/LICENSE.txt).
