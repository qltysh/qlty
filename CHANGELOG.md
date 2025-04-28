# Changelog

## v0.518.0 (2025-04-25)

### New

- Significantly improve output from `qlty coverage publish` to help setup and debugging (#1960)
- Print warnings when files in a coverage report are missing from disk (#1957)

### Fixed

- Fix Ruby download URLs to no longer depend on EOL Ubuntu 20.04 packages (#1966)

## v0.517.0 (2025-04-24)

### New

- Enhance CI with Buildkite Variable Support (#1933)

## v0.516.0 (2025-04-20)

### New

- Capture metadata about coverage uploader tools into coverage metadata (#1947)

### Improved

- Auto-detect `*.lcov` files as LCOV format (#1948)
- Automatically migrate Code Climate exclude_patterns (#1936)

## v0.515.0 (2025-04-20)

### Improved

- Adjust auto-generated fmt issues to `comment` mode (#1945)

### Fixed

- Fix missing code smells when ignore rules are used in qlty.toml (#1938)

## v0.514.0 (2025-04-20)

### Improved

- Exit early with good errors when there is a fatal error with coverage upload (#1950)

### Fixed

- Improve Clover XML parsing compatibility (#1944)

## v0.513.0 (2025-04-19)

### Fixed

- Fix detection of GitHub releases with `application/x-msdownload` MIME type on Windows (#1935)

## v0.512.0 (2025-04-19)

### New

- Add support for `--total-parts-count` to `qlty coverage publish` (#1913)

## v0.511.0 (2025-04-17)

### New

- Add support for installing using package lockfiles (#1658)

## v0.510.0 (2025-04-16)

### New

- Add dockerfmt plugin (#1865)

## v0.509.0 (2025-04-12)

### Improved

- Suggest known good versions of plugins (#1684)

## v0.508.0 (2025-04-09)

### Improved

- Update SARIF parser to support missing `uri` in `originalUriBaseIds` (#1884)

## v0.507.0 (2025-04-04)

### Fixed

- Fix bug where some plugins were inaccurately auto-initialized (#1881)

## v0.506.0 (2025-04-04)

### New

- Compile binaries for Linux Arm64 with Musl (#1626)

## v0.505.0 (2025-04-03)

### New

- Add SARIF output format for `qlty check` (#1595)

## v0.504.0 (2025-04-02)

### New

- `qlty init` will now suggest issue modes (#1628)

### Improved

- Fix panic crashes in certain edge case conditions and improve error messages (#1587)

## v0.503.0 (2025-03-28)

### Improved

- Improve plugin suggestions during `qlty init` (#1654)

## v0.502.0 (2025-03-27)

### Fixed

- Fix running `qlty check PATH` in a subdirectory (#1594)

## v0.501.0 (2025-03-27)

### New

- Update supported RadarLint version (#1579)

## v0.500.0 (2025-03-18)

### New

- Add php-cs-fixer plugin (#1580)

### Fixed

- Fix load path initialization bug affecting Ruby on Windows (#1585)
- Fix loading of RUBYLIB in a Gem-compatible way (#1583)

## v0.499.0 (2025-03-13)

### Improved

- Support `tool:ruleKey` matching in `[[override]]` blocks (#1564)
- Improve error handling for reading extra package data (#1566)
- Improve error handling when stylelint does not generate output (#1562)
- Output installation debug files in additional cases (#1554)
- Improve reliability of Composer package installs (#1555)

### Fixed

- Fix handling of `[[ignore]]` blocks with `rules = ["tool:rule"]` (#1563)

## v0.498.0 (2025-03-05)

### New

- Add checkstyle plugin (#1551)
- Add tool install debug files (#1543)

## v0.497.0 (2025-02-28)

### Improved

- Add stylelint 16 support (#1553)

## v0.496.0 (2025-02-27)

### Fixed

- Fix Git diff calculation in certain sub-path cases (#1550)
- Improve path resolution in auto-fixer (#1549)
- Fix y/n prompting bugs with lower/upper case combinations (#1548)

## v0.495.0 (2025-02-21)

### New

- Skip install errors when `--skip-errored-plugins` is enabled (#1534)

## v0.494.0 (2025-02-21)

### New

- Use Composer to install PHP plugins (#1517)

## v0.493.0 (2025-02-21)

### Fixed

- Fix fmt path issues with prefixes (#1533)

## v0.492.0 (2025-02-20)

### Fixed

- Fix plugin prefixes during `qlty init` (#1531)

## v0.491.0 (2025-02-20)

### BREAKING CHANGES

- Change warning about deprecated sources to an error (#1515)

## v0.490.0 (2025-02-18)

### New

- Add `--skip-missing-files` option to `coverage publish` (#1491)

### Improved

- Add phpcs.xml as a known config file for PHPCS (#1503)

## v0.489.0 (2025-02-18)

### Improved

- Reduce noisy logging into CLI log from dependencies (#1508)

## v0.488.0 (2025-02-18)

### Improved

- Log details when a fatal, unexpected error occurs while running a linter (#1513)
- Use Git metadata to improve issue cache accuracy (#1518)
- Treat missing phpstan output as an error (#1504)

### Fixed

- Fix bug in cache key calculation creating inaccurate results in some cases (#1509)

## v0.487.0 (2025-02-17)

### New

- Prompt users with Code Climate configs to run "config migrate" during init (#1409)

### Improved

- Auto-enable PHPStan only when a PHPStan config is found (#1512)

### Fixed

- Fix coverage parser when files have no hits (#1516)
- Only prompt the user to auto-format in correct mode (#1510)
- Fix format loop when run with `--fix` (#1514)
- Retain settings during find-and-fix loops (#1511)

## v0.486.0 (2025-02-17)

### New

- Add support for negated exclusion patterns (#1472)
- Support SimpleCov "merged" coverage format (#1493)

### Improved

- Update ruby-build definitions (#1502)

### Fixed

- Fix Ruby downloads on Linux arm64 (#1505)
- Fix Ruby installs in edge cases where version paths on disk do not match version string (#1497)
- Fix panic in package.json handling (#1492)
- Fix Ruby install for MacOS identification of arch with non-standard version numbering (#1500)

## v0.485.0 (2025-02-07)

### New

- Allow customizing plugin behavior when they do not generate any output (#1466)

## v0.484.0 (2025-02-07)

### New

- Detect SemaphoreCI environment variables for code coverage metadata (#1479)
- Add support for simplecov-json gem format (#1475)

### Improved

- Skip formatters when sampling issues during init (#1474)
- Always log stdout and stderr from prepare_script commands (#1480)

## v0.483.0 (2025-02-05)

### Fixed

- Fix bug where `[[ignore]]` without `file_patterns` would ignore everything (#1483)
- Fix applying formatting fixes from subdirectories (#1476)

## v0.482.0 (2025-02-05)

### Fixed

- Copy phpstan config files into sandbox (#1482)
- Fix Jacoco parser to not throw error when there are no lines for a source file (#1473)

## v0.481.0 (2025-02-04)

### Improved

- Improve error messaging when coverage upload token is invalid (#1462)
- Truncate very large strings when generating invocation data (#1464)

## v0.480.0 (2025-01-31)

### Fixed

- Fix panic when processing empty LCOV files with no coverage data (#1458)
- Fix panics editing code when a byte offset is not on a UTF-8 character boundry (#1460)

## v0.479.0 (2025-01-29)

### New

- Add support for Workspace-level code coverage upload tokens (#1445)

## v0.478.0 (2025-01-28)

### Improved

- Add CLI version to code coverage upload metadata (#1451)

## v0.477.0 (2025-01-28)

### Improved

- Upgrade Ruby runtime to v3.3.7 (#1449)

## v0.476.0 (2025-01-27)

### Improved

- Experimental: Add support for generating qlty.toml validation schema using Taplo (#1454)

### Fixed

- Throw an error if a prepare_script fails (#1448)
- Fix support for qlty-ignore directive on source files of unknown file types (#1447)

## v0.475.0 (2025-01-23)

### Improved

- Render column headers in verbose text output (#1437)

## v0.474.0 (2025-01-17)

### Improved

- Adjust kube-linter plugin to lint entire directory at once (#1438)

## v0.473.0 (2025-01-16)

### Improved

- Improve auto-generated .gitignore rules in .qlty/ folder (#1423)
- Support `%SRCROOT%` as a root path in SARIF parser (#1435)

## v0.472.0 (2025-01-14)

### Improved

- Default code coverage prefix stripping to workspace root (#1428)

## v0.471.0 (2025-01-14)

### Fixed

- Fix "Duplicate filename" bug when multiple coverage files had the same name (#1424)
- Fix panic when path for `qlty check <path>` is outside qlty directory (#1426)
- Fix off-by-1 offset in ripgrep plugin column info (#1427)

## v0.470.0 (2025-01-12)

### New

- Add `cache prune` subcommand to prune cached results, logs, and debug files (#1408)

## v0.469.0 (2025-01-11)

### New

- Add support for C# maintainability analysis (smells and metrics) (#1388)

## v0.468.0 (2025-01-09)

### New

- Upload raw coverage data with processed data to improve debugging experience (#1415)

### Improved

- Add `**/fixtures/**` to default exclusion patterns (#1384)

## v0.467.0 (2025-01-07)

### Improved

- Auto-initialize a new project when running "qlty plugins enable" (#1401)
- `qlty install` now installs all enabled plugins (#1397)
- Fix a concurrency deadlock during filesystem walking (#1412)
- Ignore Java `import` statements and C# `using` directives when calculating duplication (#1413)
- Improve error message when a plugin version is missing (#1394)

### Fixed

- Fix bug with code coverage paths processing (#1411)

## v0.466.0 (2025-01-02)

### New

- Add browser-based flow to authenticate the CLI as a Qlty Cloud user

## v0.465.0 (2024-12-19)

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

## v0.464.0 (2024-12-19)

### New

- Interactively prompt to apply fixes by default (override with `--fix` or `--no-fix` arguments)
- Print fix suggestions diffs

## v0.463.0 (2024-12-10)

### New

- Print list of any unformatted files before issues
- Add ability to skip Git hooks executon by pressing enter key

## v0.462.0 (2024-12-09)

### New

- Add new `note` level for issues and use it with ripgrep
- Add `--summary` argument to `qlty check` to limit output to only counts

## v0.461.0 (2024-12-09)

### New

- Add `--no-fix` argument to `qlty check` to skip applying fixes
- Add plugin platform targetting via a new `supported_platforms` key

## v0.460.0 (2024-12-05)

### New

- Print a count of securty issues at the end of the output
- Experimental: Publish Docker images to GitHub Container Registry

### Improved

- In newly-generated `qlty.toml` files from `qlty init`, emit smell issues in `comment` mode instead of `block`
- Stop running plugins when we exceed the total maximum issue limit of 10k

## v0.459.0 (2024-12-05)

### Improved

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

## v0.458.0 (2024-12-02)

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

## v0.457.0 (2024-11-27)

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

## v0.456.0 (2024-11-23)

### New

- Add `qlty plugins upgrade NAME` subcommand which upgrades to the highest, known good version of the plugin
- Add `qlty plugins disable NAME` subcommand

### Improved

- Add new Dependency Alert and Secret issue categories

### Fixed

- Don't modify `qlty.toml` when running `qlty plugins enable` with a plugin that is already enabled
- Do not automatically enable Hadolint because the latest release crashed on MacOS v15 Sequoia

## v0.455.0 (2024-11-17)

### New

- Publish the Qlty CLI as a public repository on GitHub (Fair Source License)
- Start of CHANGELOG
