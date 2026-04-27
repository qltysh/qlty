# PHP-CS-Fixer

[PHP-CS-Fixer](https://github.com/PHP-CS-Fixer/PHP-CS-Fixer) is a tool that automatically fixes PHP coding standards issues, rewriting files to conform to a configured style.

## Enabling PHP-CS-Fixer

Enabling with the `qlty` CLI:

```bash
qlty plugins enable php-cs-fixer
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "php-cs-fixer"

# OR pin to a specific version
[[plugin]]
name = "php-cs-fixer"
version = "X.Y.Z"
```

## Auto-enabling

<!-- REVIEW: confirm auto-enabling condition -->

PHP-CS-Fixer will be automatically enabled by `qlty init` if a `.php-cs-fixer.dist.php` configuration file is present.

## Languages and file types

PHP-CS-Fixer formats: PHP (`.php`).

## Configuration files

- [`.php-cs-fixer.dist.php`](https://cs.symfony.com/doc/config.html)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running PHP-CS-Fixer.

## Troubleshooting

**"PHP needs to be a minimum version" error on PHP 8.4+.**
PHP-CS-Fixer enforces a strict PHP version range. Qlty sets `PHP_CS_FIXER_IGNORE_ENV=true` automatically to bypass this check, but on hosts where that environment variable is not inherited you may still see the error.
Verify that `PHP_CS_FIXER_IGNORE_ENV=true` is present in the environment where Qlty runs.

## Links

- [PHP-CS-Fixer on GitHub](https://github.com/PHP-CS-Fixer/PHP-CS-Fixer)
- [PHP-CS-Fixer plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/php-cs-fixer)
- [PHP-CS-Fixer releases](https://github.com/PHP-CS-Fixer/PHP-CS-Fixer/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

PHP-CS-Fixer is licensed under the [MIT License](https://github.com/PHP-CS-Fixer/PHP-CS-Fixer/blob/master/LICENSE).
