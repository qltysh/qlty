# PHP_CodeSniffer

[PHP_CodeSniffer](https://github.com/PHPCSStandards/PHP_CodeSniffer) is a PHP linter that detects violations of a defined set of coding standards in PHP files.

## Enabling PHP_CodeSniffer

Enabling with the `qlty` CLI:

```bash
qlty plugins enable php-codesniffer
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "php-codesniffer"

# OR pin to a specific version
[[plugin]]
name = "php-codesniffer"
version = "X.Y.Z"
```

## Auto-enabling

PHP_CodeSniffer will be automatically enabled by `qlty init` if a `phpcs.xml` configuration file is present.

PHP_CodeSniffer will be automatically enabled by `qlty init` if a `phpcs.xml` configuration file is present.

## Languages and file types

PHP_CodeSniffer analyzes: PHP (`.php`).

## Configuration files

- [`phpcs.xml`](https://github.com/PHPCSStandards/PHP_CodeSniffer/wiki/Advanced-Usage#using-a-default-configuration-file)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running PHP_CodeSniffer.

## Troubleshooting

**php-codesniffer reports violations on auto-generated or vendor files.**
PHP_CodeSniffer scans all PHP files unless explicitly excluded. Generated files (for example migration stubs, ORM proxies) and `vendor/` often fail many rules.
Add `<exclude-pattern>vendor/*</exclude-pattern>` and any generated directories to your `phpcs.xml` configuration file to skip them.

**php-codesniffer reports "ERROR: Referenced sniff does not exist" on a custom standard.**
When the configured coding standard name cannot be resolved (for example because a custom standard package is not installed), PHP_CodeSniffer exits with an error.
Ensure the package providing the standard is installed via `composer require --dev <package>` and that `composer install` has been run so the standard is in `vendor/`.

## Links

- [PHP_CodeSniffer on GitHub](https://github.com/PHPCSStandards/PHP_CodeSniffer)
- [PHP_CodeSniffer plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/php-codesniffer)
- [PHP_CodeSniffer releases](https://github.com/PHPCSStandards/PHP_CodeSniffer/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

PHP_CodeSniffer is licensed under the [BSD 3-Clause License](https://github.com/PHPCSStandards/PHP_CodeSniffer/blob/master/licence.txt).
