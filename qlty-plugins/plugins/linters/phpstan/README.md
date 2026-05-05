# PHPStan

[PHPStan](https://github.com/phpstan/phpstan) is a PHP static analysis tool that finds bugs in your code without running it, focusing on type safety and undefined behavior.

## Enabling PHPStan

Enabling with the `qlty` CLI:

```bash
qlty plugins enable phpstan
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "phpstan"

# OR pin to a specific version
[[plugin]]
name = "phpstan"
version = "X.Y.Z"
```

## Auto-enabling

PHPStan will be automatically enabled by `qlty init` if a `phpstan.neon` or `phpstan.neon.dist` configuration file is present.

PHPStan will be automatically enabled by `qlty init` if a `phpstan.neon` configuration file is present.

## Languages and file types

PHPStan analyzes: PHP (`.php`).

## Configuration files

- [`phpstan.neon`](https://phpstan.org/config-reference)

Accepted filenames: `phpstan.neon`, `phpstan.neon.dist`, `phpstan.dist.neon`.

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running PHPStan.

## Troubleshooting

**phpstan fails with "Composer autoloader not found" or "autoload file could not be loaded".**
PHPStan requires the project's Composer autoloader to be available. If `vendor/autoload.php` does not exist (because `composer install` has not been run), PHPStan cannot resolve class names.
Run `composer install` before Qlty runs, or add `vendor/**` to `exclude_patterns` if you prefer to skip it.

**phpstan reports many false positives on code that uses dynamic method calls or magic properties.**
PHPStan's strict mode can flag dynamic calls and magic `__get`/`__set` methods as errors. Legacy PHP codebases heavily using these patterns will produce many false positives at higher levels.
Lower the PHPStan `level` in `phpstan.neon` (for example to `level: 3`) to reduce noise, or add `@phpstan-ignore` annotations to suppress specific known-safe calls.

## Links

- [PHPStan on GitHub](https://github.com/phpstan/phpstan)
- [PHPStan plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/phpstan)
- [PHPStan releases](https://github.com/phpstan/phpstan/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

PHPStan is licensed under the [MIT License](https://github.com/phpstan/phpstan/blob/2.x/LICENSE).
