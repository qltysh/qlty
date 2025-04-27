# Changelog

## Week of April 20, 2025

### New

- Significantly improve output from `qlty coverage publish` to help setup and debugging (#1960)
- Print warnings when files in a coverage report are missing from disk (#1957)

### Fixed

- Fix Ruby download URLs to no longer depend on EOL Ubuntu 20.04 packages (#1966)

## Week of April 13, 2025

### New

- Enhance CI with Buildkite Variable Support (#1933)
- Capture metadata about coverage uploader tools into coverage metadata (#1947)

### Improved

- Auto-detect `*.lcov` files as LCOV format (#1948)
- Automatically migrate Code Climate exclude_patterns (#1936)
- Adjust auto-generated fmt issues to `comment` mode (#1945)
- Exit early with good errors when there is a fatal error with coverage upload (#1950)

### Fixed

- Fix missing code smells when ignore rules are used in qlty.toml (#1938)
- Improve Clover XML parsing compatibility (#1944)
- Fix detection of GitHub releases with `application/x-msdownload` MIME type on Windows (#1935)

## Week of April 6, 2025

### New

- Add support for `--total-parts-count` to `qlty coverage publish` (#1913)
- Add support for installing using package lockfiles (#1658)
- Add dockerfmt plugin (#1865)

### Improved

- Suggest known good versions of plugins (#1684)
- Update SARIF parser to support missing `uri` in `originalUriBaseIds` (#1884)

### Fixed

- Fix bug where some plugins were inaccurately auto-initialized (#1881)

## Week of March 30, 2025

### New

- Compile binaries for Linux Arm64 with Musl (#1626)
- Add SARIF output format for `qlty check` (#1595)
- `qlty init` will now suggest issue modes (#1628)

### Improved

- Fix panic crashes in certain edge case conditions and improve error messages (#1587)
- Improve plugin suggestions during `qlty init` (#1654)

### Fixed

- Fix running `qlty check PATH` in a subdirectory (#1594)

## Week of March 23, 2025

### New

- Update supported RadarLint version (#1579)
- Add php-cs-fixer plugin (#1580)

### Fixed

- Fix load path initialization bug affecting Ruby on Windows (#1585)
- Fix loading of RUBYLIB in a Gem-compatible way (#1583)

## Week of March 16, 2025

### Improved

- Support `tool:ruleKey` matching in `[[override]]` blocks (#1564)
- Improve error handling for reading extra package data (#1566)
- Improve error handling when stylelint does not generate output (#1562)
- Output installation debug files in additional cases (#1554)
- Improve reliability of Composer package installs (#1555)

### Fixed

- Fix handling of `[[ignore]]` blocks with `rules = ["tool:rule"]` (#1563)

## Week of March 9, 2025

### New

- Add checkstyle plugin (#1551)
- Add tool install debug files (#1543)

### Improved

- Add stylelint 16 support (#1553)

### Fixed

- Fix Git diff calculation in certain sub-path cases (#1550)
- Improve path resolution in auto-fixer (#1549)
- Fix y/n prompting bugs with lower/upper case combinations (#1548)

## Week of March 2, 2025

### New

- Skip install errors when `--skip-errored-plugins` is enabled (#1534)

## Week of February 23, 2025

### New

- Use Composer to install PHP plugins (#1517)

### Fixed

- Fix fmt path issues with prefixes (#1533)
- Fix plugin prefixes during `qlty init` (#1531)

## Week of February 16, 2025

### BREAKING CHANGES

- Change warning about deprecated sources to an error (#1515)

### New

- Add `--skip-missing-files` option to `coverage publish` (#1491)
- Prompt users with Code Climate configs to run "config migrate" during init (#1409)
- Add support for negated exclusion patterns (#1472)
- Support SimpleCov "merged" coverage format (#1493)

### Improved

- Add phpcs.xml as a known config file for PHPCS (#1503)
- Reduce noisy logging into CLI log from dependencies (#1508)
- Log details when a fatal, unexpected error occurs while running a linter (#1513)
- Use Git metadata to improve issue cache accuracy (#1518)
- Treat missing phpstan output as an error (#1504)
- Auto-enable PHPStan only when a PHPStan config is found (#1512)
- Update ruby-build definitions (#1502)

### Fixed

- Fix bug in cache key calculation creating inaccurate results in some cases (#1509)
- Fix Ruby downloads on Linux arm64 (#1505)
- Fix coverage parser when files have no hits (#1516)
- Only prompt the user to auto-format in correct mode (#1510)
- Fix Ruby installs in edge cases where version paths on disk do not match version string (#1497)
- Fix panic in package.json handling (#1492)
- Fix format loop when run with `--fix` (#1514)
- Retain settings during find-and-fix loops (#1511)
- Fix Ruby install for MacOS identification of arch with non-standard version numbering (#1500)

## Week of February 2, 2025

### New

- Allow customizing plugin behavior when they do not generate any output (#1466)
- Detect SemaphoreCI environment variables for code coverage metadata (#1479)
- Add support for simplecov-json gem format (#1475)

### Improved

- Skip formatters when sampling issues during init (#1474)
- Always log stdout and stderr from prepare_script commands (#1480)

### Fixed

- Fix bug where `[[ignore]]` without `file_patterns` would ignore everything (#1483)
- Fix applying formatting fixes from subdirectories (#1476)
- Copy phpstan config files into sandbox (#1482)
- Fix Jacoco parser to not throw error when there are no lines for a source file (#1473)

## Week of January 26, 2025

### Improved

- Improve error messaging when coverage upload token is invalid (#1462)
- Truncate very large strings when generating invocation data (#1464)

### Fixed

- Fix panic when processing empty LCOV files with no coverage data (#1458)
- Fix panics editing code when a byte offset is not on a UTF-8 character boundry (#1460)

## Week of January 19, 2025

### New

- Add support for Workspace-level code coverage upload tokens (#1445)

### Improved

- Add CLI version to code coverage upload metadata (#1451)
- Upgrade Ruby runtime to v3.3.7 (#1449)
- Experimental: Add support for generating qlty.toml validation schema using Taplo (#1454)

### Fixed

- Throw an error if a prepare_script fails (#1448)
- Fix support for qlty-ignore directive on source files of unknown file types (#1447)

## Week of January 12, 2025

### Improved

- Render column headers in verbose text output (#1437)
- Adjust kube-linter plugin to lint entire directory at once (#1438)
- Improve auto-generated .gitignore rules in .qlty/ folder (#1423)
- Support `%SRCROOT%` as a root path in SARIF parser (#1435)
- Default code coverage prefix stripping to workspace root (#1428)

### Fixed

- Fix "Duplicate filename" bug when multiple coverage files had the same name (#1424)
- Fix panic when path for `qlty check <path>` is outside qlty directory (#1426)
- Fix off-by-1 offset in ripgrep plugin column info (#1427)

## Week of January 5, 2025

### New

- Add `cache prune` subcommand to prune cached results, logs, and debug files (#1408)
- Add support for C# maintainability analysis (smells and metrics) (#1388)
- Upload raw coverage data with processed data to improve debugging experience (#1415)

### Improved

- Add `**/fixtures/**` to default exclusion patterns (#1384)
- Auto-initialize a new project when running "qlty plugins enable" (#1401)
- `qlty install` now installs all enabled plugins (#1397)
- Fix a concurrency deadlock during filesystem walking (#1412)
- Ignore Java `import` statements and C# `using` directives when calculating duplication (#1413)
- Improve error message when a plugin version is missing (#1394)

### Fixed

- Fix bug with code coverage paths processing (#1411)

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
