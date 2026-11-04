# Rubocop

[Rubocop](https://github.com/rubocop/rubocop) is a Ruby static code analyzer (a.k.a. linter) and code formatter.

## Enabling Rubocop

Enabling with the `qlty` CLI:

```bash
qlty plugins enable rubocop
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "rubocop"

# OR pin to a specific version
[[plugin]]
name = "rubocop"
version = "X.Y.Z"
```

## Auto-enabling

Rubocop will be automatically enabled by `qlty init` if a `.rubocop.yml` configuration file is present.

## Installing from a Gemfile with custom sources

To use your Gemfile's original gem sources, including private source blocks:

```toml
[[plugin]]
name = "rubocop"
package_file = "Gemfile"
preserve_gemfile_sources = true

[[plugins.definitions.rubocop.environment]]
name = "BUNDLE_GEMS__EXAMPLE__COM"
value = "${env.BUNDLE_GEMS__EXAMPLE__COM}"
```

Set the credential through the environment, or through a Build Secret on Qlty
Cloud, using the Bundler variable for your source's hostname. Bundler handles
authentication; missing credentials produce an installation error.

`preserve_gemfile_sources` defaults to `false`, so existing configurations keep
their current source rewriting. The option applies to Ruby plugins with a
`package_file` and cannot be combined with `package_filters`. When enabled, Qlty
preserves the sources and their gems without adding a rubygems.org source. The
Gemfile must declare the sources it needs. Qlty still removes `ruby` directive
lines for its managed runtime and stages the Gemfile and `Gemfile.lock` in the
tool's installation directory.

## Configuration files

- [`.rubocop.yml`](https://docs.rubocop.org/rubocop/1.63/configuration.html)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running Rubocop.

## Languages and file types

Rubocop analyzes: Ruby (`.rb`, `.gemspec`).

## Troubleshooting

**Issues are not shown for some files even though violations exist.**
Qlty drops all issues from a file when Rubocop produces more than 500 results for it. Large model or controller files with accumulated tech debt commonly hit this limit.
Add those files to `exclude_patterns` in `qlty.toml`, or run `rubocop <file>` locally to fix violations in bulk.

**"Unrecognized cop" or "cop has been renamed" errors appear in logs.**
Your `.rubocop.yml` references cop names that have been renamed or removed in the version of Rubocop Qlty is running.
Update your config to use the new names (the warning message includes the replacement), or pin `rubocop` to the version your config targets.

**Config files inside subdirectories (e.g. `db/`, `config/`, `spec/`) are not being used.**
Files matching `exclude_patterns` in `qlty.toml` are skipped, even if they are valid Rubocop configs. `.rubocop.yml` files inside `db/` or `config/` fall into this category by default.
Move the relevant config to `.qlty/configs/` or narrow your `exclude_patterns` to un-exclude those directories.

**"Extension supports plugin — specify `plugins:` instead of `require:`" warning.**
Newer Rubocop versions require extension gems to be declared under `plugins:` rather than `require:` in `.rubocop.yml`.
Replace `require: rubocop-capybara` (or the relevant gem) with `plugins: rubocop-capybara` in your `.rubocop.yml` and consult the [plugin migration guide](https://docs.rubocop.org/rubocop/plugin_migration_guide.html).

## Links

- [Rubocop on GitHub](https://github.com/rubocop/rubocop)
- [Rubocop plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/rubocop)
- [Rubocop releases](https://github.com/rubocop/rubocop/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

Rubocop is licensed under the [MIT License](https://github.com/rubocop/rubocop/blob/master/LICENSE.txt).
