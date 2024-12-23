# Changelog

## Week of December 15th, 2024

### New

- Add ability to skip Git hooks executon by pressing enter key

### Improved

- Stop running plugins when we exceed the total maximum issue limit of 10k
- Disable plugins by adding `mode = "disabled"` instead of removing them from `qlty.toml`
- Reduce output when running without `--verbose`
- Minor improvements to output content

### Fixed

- Fix panic when running `qlty fmt` with a non-existent path argument

## Week of December 8th, 2024

### New

- Major change: Compile official plugin definitions into CLI binaries
- Add kube-linter plugin
- Automtically run `qlty fmt` if `qlty check --fix` applies autofixes

### Improved

- Detect and use Biome version from `package.json` if present
- Target more file types for Biome. Thank you @lamualfa for this contribution!
- Add operating system, CPU architecture, and build date to `qlty version` output
- Limit amount of results data processed to 100 MB
- Limit the maximum number of issues per file to 100
- Improve documentation for writing a new plugin

### Fixed

- Add targetting for `\*.mts`, `\*.cts`, `\*.mtsx`, and `\*.ctsx` extensions
- Prevent stack overflow panics when analyzing deeply nested ASTs

## Week of December 1st, 2024

### New

- Add `qlty githooks install` command to install Git pre-commit and pre-push hooks (alpha)
- Add `qlty githooks uninstall` command to remove Git hooks
- Add Reek plugin. Thank you @noahd1 for this contribution!

### Improved

- Support initializing running StandardRB run via Rubocop

### Fixed

- Don't apply unsafe fixes when running `qlty check --fix` unless `--unsafe` is specified
- When ESLint <= v5 crashes, avoid throwing errors about non-existant tempfiles
- Avoid panic when attempting to install shell completions for unknown shell
- Fix analysis of Go binary expressions and paramaters counts
- Fixed a bug with `qlty init` set up for ESLint
- Fixed a bug parsing Ruff output when it finds syntax errors

## Week of November 24th, 2024

### New

- Add `qlty plugins upgrade NAME` subcommand which upgrades to the highest, known good version of the plugin
- Add `qlty plugins disable NAME` subcommand

### Improved

- Add new Dependency Alert and Secret issue categories

### Fixed

- Don't modify `qlty.toml` when running `qlty plugins enable` with a plugin that is already enabled
- Do not automatically enable Hadolint because the latest release crashed on MacOS v15 Sequoia

## Week of November 17th, 2024

### New

- Publish the Qlty CLI as a public repository on GitHub (Fair Source License)
- Start of CHANGELOG
