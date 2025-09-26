# Changelog

## v0.578.0 (2025-09-26)

### New

- Ensure read important env vars for BuildKite for coverage publishing (#2415)

### Improved

- Reduce snippet size limits for storage and performance optimization (#2409)

## v0.577.0 (2025-09-25)

### Fixed

- Fix interaction of exported_config_paths and copy_configs_into_tool_install directives (#2402)

#### Notes

NB: There was no release versions: 0.575.0, 0.576.0

## v0.574.0 (2025-09-12)

### Improved

- Improve coverage publish output (#2378)

## v0.573.0 (2025-09-11)

### Improved

- Adjust plugin suggestions to prioritize commonly used tools (#2373)

### Fixed

- Remove `./` from beginning of `--add-prefix` in coverage command (#2376)

## v0.572.0 (2025-09-08)

### Fixed

- Improve error handling when retrieving commit metadata (#2371)

## v0.571.0 (2025-09-04)

### New

- Add `skip_errored_plugins` option to install command (#2366)

## v0.570.0 (2025-08-29)

### Fixed

- Escape special characters in file paths for glob matching (#2337)

## v0.569.0 (2025-08-27)

### New

- Add ktlint plugin (#2352)

## v0.568.0 (2025-08-22)

### Improved

- Add debug statements to jacoco parser (#2339)

## v0.567.0 (2025-08-20)

### New

- Support JACOCO_SOURCE_PATH environment variable for coverage resolution (#2332)

### Fixed

- Never check rustc version to avoid unneeded breakages (#2317)
- Ensure ZDOTDIR environment variable is properly set (#2315)

Thank you @mgnisia for your contribution!

## v0.566.0 (2025-08-15)

### New

- Support merge group detection on any CI provider listening to GitHub push events (#2320)

## v0.565.0 (2025-08-13)

### New

- Print formatter name when formatting (#2313)

### Fixed

- Prioritize explicitly stated version during merge (#2312)
- Merge enabled plugins by name and prefix (#2310)

## v0.564.0 (2025-08-11)

### New

- Initial support for GitHub Merge Queues
- Updates for Rust 1.89

## v0.563.0 (2025-08-06)

### New

- Implement GitHub SHA resolution for pull requests in coverage publish instead of relying on the sha being supplied via --override-commit-sha (#2293)

### Fixed

- Do not require a reference (branch, tag or pr) or commit time for the `coverage complete` subcommand (#2291)

## v0.562.0 (2025-08-05)

### Fixed

- Fix installation of libssl on Linux when installing Ruby (#2288)

## v0.561.0 (2025-08-04)

This release contains a breaking change for the `coverage publish` command.

Now the --validate flag will be enabled by default. This means the Qlty CLI will automatically validate your coverage reports before attempting to upload them. This allows you to identify and fix issues path matching issues much earlier in your dev process.

What This Means for You:

- Potential CI Build Failures: Once this change is implemented, if your current CI/CD pipeline uploads a report with mismatched paths, your builds will begin to fail when executing qlty coverage publish. To avoid disruption, you can begin validating your reports now (by using --validate ) to address any issues before this change takes effect.
- Quick Fix for Build Failures: If your builds start failing on Monday and you need to get them passing immediately, you can temporarily add the new --no-validate flag to your qlty coverage publish command. This will disable validation and allow your CI build to pass (though your coverage data will remain broken until you've uploaded a valid report).

We believe this change will significantly improve the accuracy and usability of your coverage data within Qlty. If you have any questions or require assistance, please don't hesitate to contact our support team.

See our [path fixing docs](https://docs.qlty.sh/coverage/path-fixing) for more information.

## v0.560.0 (2025-08-04)

### New

- Add no-op option '--no-validate' to support GitHub Action

## v0.559.0 (2025-08-04)

### New

- Add zizmor plugin (#2265)

### Fixed

- Remove branch requirement for coverage: instead, require branch, tag OR pull request (#2273)
- Filter broken pipe errors from Sentry reporting (#2255)
- Fix fmt prompt path targeting (#2230)

## v0.558.0 (2025-07-27)

### Improved

- Add installation error to stderr during tool installation error (#2231)
- Add output after installing githooks (#2217)

### Fixed

- Incorrect parameter counting for Kotlin functions with annotations (#2160)
- Honor ZDOTDIR environment variable in install.sh (#2240)

Thank you @voltechs for your contribution!

## v0.557.0 (2025-07-22)

### Fixed

- Respect Git `insteadOf` when fetching sources (#2235)

## v0.556.0 (2025-07-17)

### New

- Bubble installation log into build logs on error (#2215)
- Point to debug logs when lint errors occur (#2222)
- Add Coverprofile parser (#2225)

## v0.555.0 (2025-07-10)

No changes. (Release was triggered by CI workflow upgrades.)

## v0.552.0 (2025-07-09)

### New

- Batch file_coverages data in JSONL files (#2182)

## v0.548.0 (2025-06-27)

### Fixed

- Treat exceeding 50,000 total issues as a fatal error (#2178)

### Improved

- Lower issue JSONL file batch size (#2179)

## v0.547.0 (2025-06-27)

### Fixed

- Show issues for plugins with no end lines in diff mode (#2176)

### Improved

- Truncate issue snippets for maintainability issues (#2180)

## v0.546.0 (2025-06-24)

### New

- Add timestamps to coverage output (#2170)

## v0.545.0 (2025-06-24)

### New

- Make strip prefix coverage transformer optional (#2172)

## v0.544.0 (2025-06-24)

### New

- Support `coverage complete` command without Git (#2168)
- Allow `coverage publish` to run without Git when using `--override-commit-time` (#2167)

## v0.543.0 (2025-06-23)

### New

- Batch build messages and stats into smaller files (#2165)
- Batch build issues records into multiple JSONL files (#2157)
- Add `--override-commit-time` flag to `coverage publish` command (#2155)

## v0.542.0 (2025-06-19)

### New

- Batch build invocation records into multiple JSONL files (#2149)

## v0.541.0 (2025-06-11)

### New

- Add dotCover format support for coverage reports (#2143)

### Fixed

- Add eslint.config.ts as a known ESLint config file (#2137)

Thank you @kolarski for your contribution!

## v0.540.0 (2025-06-05)

### Fixed

- Truncate issue snippets to prevent large Issue records (#2135)

## v0.539.0 (2025-06-02)

### New

- Add support for `--sarif` flag to `qlty smells` command (#2037)

Thank you @ujlbu4 for your contribution!

### Fixed

- Reject coverage reports when branch name is missing (#2119)
- Handle terraform parser to handle missing location (#2129)
- Update shellcheck and tab column width (#2127)
- Only print identical warnings once (#2125)
- Pass proxy env variables to installations and invocations (#2126)
- Add ca-certificates to Docker image (#2124)
- Support head diff mode for `qlty smells` (#2039)

Thank you @relu and @ujlbu4 for your contributions!

## v0.538.0 (2025-05-31)

### New

- Add haml-lint plugin (#2112)
- Add ruff auto-formatting support (#2080)

### Improved

- Reject invalid configuration combinations in qlty.toml (#2077)
- Support Git fetches through HTTPS proxies (#2081)

### Fixed

- Include AWS response body in coverage upload error messages (#2085)
- Warn instead of error in `config migrate` if we can't find a plugin for a fetch item (#2096)
- Use system TLS certificate roots for downloads (#2075)
- Improve accuracy of field counting for Java (#2082)

## v0.537.0 (2025-05-30)

### New

- Add support for auto-detection of coverage variables for Travis CI users (#2108)

### Fixed

- Add more support for SimpleCov coverage formats (#2111)
- Fix shellcheck tab column handling in location ranges (#2099)

## v0.536.0 (2025-05-29)

### Improved

- Filter plugins in `qlty config migrate` based on .codeclimate.yml (#2094)
- Improve warning messages for deprecated configuration syntax (#2101)
- Report the number of files excluded for code coverage up to Qlty Cloud (#2098)

## v0.535.0 (2025-05-28)

### New

- Add support for `[[triage]]` blocks (replaces `[[override]]`) (#2005)
- Add support for `[[exclude]]` blocks (replaces `[[ignore]]`) (#1954)
- Support linux/arm64 platform in Docker image (#1990)

Thank you @ujlbu4 for your contribution of linux/arm64 Docker support!

### Improved

- Update many linter versions (#2029)
- Add output to indicate the number of excluded code coverage paths (#2078)
- Ignore note issues in Git pre-push hook (#2079)

### Fixed

- Eliminate panic from telemetry when network is disabled (#2086)

## v0.534.0 (2025-05-22)

### New

- Add terraform plugin with support for formatting and validation (#2067)

### Fixed

- Fix formatter invocation directory when runs from target_directory (#2066)

## v0.533.0 (2025-05-22)

### Improved

- Improve Git authentication to support credential configuration from gitconfig (#2068)

## v0.532.0 (2025-05-21)

### Improved

- Improve behavior of `qlty install` to better align with `qlty check` (#2064)

## v0.531.0 (2025-05-20)

### New

- Add extra trigger options: `agent`, `ide` (#2063)

### Improved

- Detect `biome.jsonc` as a config file for Biome plugin (#2059)
- Tighten tool install retry policy (#2060)
- Better error messages for errored downloads (#2055)
- Support HTTP proxies from env vars for tool downloads (#2056)

### Fixed

- Fix loading deep config files into staging (#2058)

Thank you @raybrownco for your contribution!

## v0.530.0 (2025-05-20)

This release failed and this version does not exist.

## v0.529.0 (2025-05-16)

### Fixed

- Fixed providing runtime env vars to tool execution (#2051)

## v0.528.0 (2025-05-16)

### New

- Add new `exported_config_paths` directive to allow sources to provide linter configs (#2044)

### Improved

- Only enable radarlint-ruby when a RadarLint config is present (#2047)

### Fixed

- Fix `suggested_mode` configuration for RadarLint (#2047)

## v0.527.0 (2025-05-15)

### New

- Add Ruby preflight dependency checks to Ruby binary install (#2028)

### Improved

- Treat hitting maximum total issues of 50k as a fatal error (#2027)
- Skip version check for Rust when using a named channel (#2046)
- Print coverage report URL at the end of `coverage complete` (#2036)
- Display version command on failure for user debuggability (#2031)

## v0.526.0 (2025-05-05)

### Improved

- Increase maximum total issues count from 10k to 50k (#2024)

## v0.525.0 (2025-05-05)

### New

- Add `--validate` option for `qlty coverage publish` (#1915)

### Improved

- Increase maximum issues per file limit from 100 to 500 (#2023)
- Automatically install sandboxes libyaml when installing Ruby on Linux (#2009)

### Fixed

- Disable binary installs of Ruby on Linux Musl platforms in favor of compilation (#2010)
- Allow coverage publish `--dry-run` to work without token (#2017)

## v0.524.0 (2025-05-03)

### New

- Add a `qlty coverage complete` sub-command (#2011)
- Add a new `--incomplete` argument to `qlty coverage publish` (#2006)
- Add optional `--name` argument to `qlty coverage publish` (#2015)

### Improved

- Auto-detect `.rubcoop-*.yml` files as RuboCop config files (#2016)
- Print coverage upload ID in `qlty coverage publish` output (#2014)

### Fixed

- Fix coverage format specification for absolute paths on Windows (#2013)
- Fix bubbling of stderr to tool install log files (#2008)

## v0.523.0 (2025-05-01)

### Improved

- Increase detail in error messages from `qlty coverage publish` (#2004)

## v0.522.0 (2025-05-01)

### New

- Add stringlint linter (<https://github.com/dral3x/StringsLint>) for Swift (#1987)
- Add SwiftFormat auto-formatter (<https://github.com/swiftlang/swift-format>) (#1986)
- Add support for a `source.toml` file in custom sources (#2000)

### Improved

- Improve subcommand descriptions in `--help` output (#1992)
- `qlty plugins enable` will now install linter configs like `qlty init` (#1997)

### Fixed

- Prevent installing linter configurations during init if another config is present (#1998)
- Fix TOML generated by `qlty init --source=...` (#1996)
- Fix TOML merging of arrays of tables when applying cascading configuration (#1995)
- Prevent `.qlty/.gitignore` from ignoring itself (#1988)
- Specify `suppported_platforms = ["macos"]` for SwiftLint (#1985)

## v0.521.0 (2025-04-29)

### Improved

- Print installation errors to CLI output (#1556)

### Fixed

- Fix golanglint-ci incorrectly downloading 32-bit binaries on Windows (#1989)

## v0.520.0 (2025-04-28)

### Improved

- Provide a default `.yamllint.yaml` when initializing yamllint (#1973)

### Fixed

- Fix a bug where yamllint would report errors on a hadolint config file (#1973)

## v0.519.0 (2025-04-28)

### New

- Add ast-grep plugin (#1972)
- Add Swift support for maintainability (complexity, duplication, smells, metrics) (#1971)

### Improved

- Add `**/templates/**` to default exclude patterns (#1967)
- Include Invocation ID in diagnostic messages for improved debugging (#1968)
- Simplify a few arguments for `qlty coverage publish` (#1965)
- Provide a default `.shellcheckrc` when initializing shellcheck (#1973)

### Fixed

- Fix parsing of Clover coverage data with repeated, non-adjacent XML elements (#1964)
- Fix compilation bug preventing default plugin configs from being installed during `qlty init` (#1983)
- Update hadolint to version 2.12.1-beta to fix on MacOS (#1981)

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
