# CoffeeLint

[CoffeeLint](https://github.com/coffeelint/coffeelint) is a style checker for CoffeeScript that helps keep CoffeeScript code clean and consistent.

## Enabling CoffeeLint

Enabling with the `qlty` CLI:

```bash
qlty plugins enable coffeelint
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "coffeelint"

# OR pin to a specific version
[[plugin]]
name = "coffeelint"
version = "X.Y.Z"
```

## Auto-enabling

CoffeeLint will be automatically enabled by `qlty init` if a `coffeelint.json` configuration file is present.

CoffeeLint will be automatically enabled by `qlty init` if a `coffeelint.json` configuration file is present.

## Languages and file types

CoffeeLint analyzes: CoffeeScript (`.coffee`).

## Configuration files

- [`coffeelint.json`](https://coffeelint.github.io/#options)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running CoffeeLint.

## Troubleshooting

**coffeelint reports no issues on CoffeeScript files that have obvious style violations.**
coffeelint requires a `coffeelint.json` configuration file to define which rules are active. Without it, the default rule set may not include the rules you expect.
Create a `coffeelint.json` (or add a `coffeelint` key to `package.json`) to explicitly enable the rules your project needs.

**coffeelint does not run on `.coffee` files even though they are present.**
If CoffeeScript is not listed in the `file_types` for the project, qlty may not pass `.coffee` files to coffeelint.
Verify that the project has `.coffee` files and that they are not excluded by `exclude_patterns` in `qlty.toml`.

## Links

- [CoffeeLint on GitHub](https://github.com/coffeelint/coffeelint)
- [CoffeeLint plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/coffeelint)
- [CoffeeLint releases](https://github.com/coffeelint/coffeelint/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

CoffeeLint is licensed under the [MIT License](https://github.com/coffeelint/coffeelint/blob/master/LICENSE).
