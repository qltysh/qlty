# Changelog

## Week of December 29th, 2024

### New

- Add browser-based flow to authenticate the CLI as a Qlty Cloud user

## Week of December 22nd, 2024

### New

- Prompt to format unformatted files when running `qlty check`
- Re-run analysis when fixes are applied to ensure fresh results

### Improved

- Add a warning when a deprecated repository-based default source is detected

### Fixed

- Fix cognitive complexity calculation for Ruby conditionals
- Don't print paths to non-existent install logs in error output
- When running as a Git pre-push hook, fallback to comparing against upstream if the remote head is not present locally
- Fix built-in duplication AST filters

## Week of December 15th, 2024

### New

- Interactively prompt to apply fixes by default (override with `--fix` or `--no-fix` arguments)
- Print fix suggestions diffs
- Print list of any unformatted files before issues
- Add ability to skip Git hooks executon by pressing enter key
- Add new `note` level for issues and use it with ripgrep
- Add `--summary` argument to `qlty check` to limit output to only counts
- Add `--no-fix` argument to `qlty check` to skip applying fixes
- Add plugin platform targetting via a new `supported_platforms` key
- Print a count of securty issues at the end of the output
- Experimental: Publish Docker images to GitHub Container Registry

### Improved

- In newly-generated `qlty.toml` files from `qlty init`, emit smell issues in `comment` mode instead of `block`
- Stop running plugins when we exceed the total maximum issue limit of 10k
- Disable plugins by adding `mode = "disabled"` instead of removing them from `qlty.toml`
- Reduce output when running without `--verbose`
- Moved invocation files to be stored in `out/`
- Automatically enable rustfmt if a rustfmt config file is found
- Minor improvements and cleanups to command output
- Shorted formatting of thread IDs printed to log files

### Fixed

- Improve data qualty of output Markdownlint issues
- Emit Hadolint style issues at `low` level (not `fmt`, which is reserved)
- Fix panic when running `qlty fmt` with a non-existent path argument

### Breaking

- Removed support for configuring sources as a hash in `qlty.toml` in favor of array notation

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
