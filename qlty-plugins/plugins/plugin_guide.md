# Qlty Plugin Development Guide

The plugins directory contains definitions and tests for plugins supported by the Qlty CLI.

Creating a plugin in Qlty typically the following steps:

1. Creating the folder structure and a valid plugin.toml file
2. (Sometimes) Creating a parser in Rust which understands the tool's output format
3. Creating at least 1 test target file in the fixtures directory with its expected output as a snapshot

## Plugin Structure / Definition

(N.B.: Copying and editing an existing plugin is a good way to get started, but this section helps put these files into context.)

A plugin consists of:

1. A top-level folder for `MY_PLUGIN` at `linters/${MY_PLUGIN}`
2. The plugin definition file `linters/${MY_PLUGIN}/plugin.toml` located at the top level of this folder, along with a simple test runner
3. A "fixtures" directory under plugins containing the test target and snapshot for the target.

## The Plugin Definition File ("plugin.toml")

The plugin definition file (`plugin.toml`) is the heart of the plugin; it contains instructions on how to install the plugin; its capabilities; and how to run it.

You'll typically see at least a section for the definition of the plugin (`[plugins.definitions.${MY_PLUGIN}]`) as well as one or more sections for each "driver" the plugin supports. For a plugin that supports both formatting and linting, you'd see two a section for each one of these drivers: `[plugins.definitions.${MY_PLUGIN}.drivers.lint]` and `[plugins.definitions.${MY_PLUGIN}.drivers.format]`

### Plugin Installation

Every plugin.toml needs to define how to find/install the plugin. There are a variety of options, listed below:

### Plugin Installation: GitHub Release

If the plugin lives on GitHub, as many do, and stores its releases on GitHub, the plugin can define (and later reference) a "releases" section.

```
[plugins.releases.${MY_PLUGIN}]
github = "plugin-owner/plugin-repo"
download_type = "executable"
```

`download_type` can also be defined as a `zip` or `targz`

The plugin definition references this release as follows:

```
[plugins.definitions.${MY_PLUGIN}]
releases = ["${MY_PLUGIN}"]
```

NB: The GitHub Release installation type uses a heuristic to determine the appropriate download URL for the platform/architecture in question. If the release doesn't follow this heuristic, you may find you may need to define each architecture/platform individually

### Plugin Installation: Downloads

If the plugin binaries can be downloaded, you can also define download sections for each known platform / architecture. This is also appropriate for some GitHub Releases which do not follow a URL standard we recognize for architecture/platform.

Define a download section for each architecture/platform the plugin supports. E.g. and substitute a variable ${version} if the version is in the URL.

```
[[plugins.downloads.${MY_PLUGIN}.system]]
cpu = "x86_64"
os = "macos"
url = "https://github.com/myplugin-owner/myplugin-repo/releases/download/v${version}/my-plugin-darwin"
```

```
[[plugins.downloads.${MY_PLUGIN}.system]]
cpu = "aarch64"
os = "windows"
url = "https://github.com/myplugin-owner/myplugin-repo/releases/download/v${version}/my-plugin_arm64.exe"
```

In the main plugin definition section, reference these downloads as follows:

```
[plugins.definitions.${MY_PLUGIN}]
downloads = ["${MY_PLUGIN}"]
```

### Plugin Installation: With a Runtime

For plugins which require a runtime, such as python, define the runtime and package in the main plugin defintion. For example:

```
[plugins.definitions.${MY_PLUGIN}]
runtime = "python"
package = "${MY_PLUGIN}"
```

### Plugin Driver(s)

Drivers are defined under `[plugins.definitions.${MY_PLUGIN}.drivers.${driver}]` and contain the script, success codes, output, output format, batch and a few other options.

- `success_codes`: An array of exit codes that denote that the plugin run finished successfully. Ordinarily you might expect this to just be 0 to denote a binary exit successfully, but because many plugins use a 0 vs 1 (or 2) to differentiate between successful runs that did not find any issues vs successful runs that did find issues, Qlty accepts an array of valid exit codes.

## Creating a Parser

If the plugin supports a standard format, like SARIF, you are set, and do not need to create a parser. Many tools output structured (like JSON), but not standardized, output, which requires a Rust parser to translate that structure into Qlty issues.

Parsers can be found in `qlty_check/source/parser/*` and are the best source for learning how to write a parser. Writing a parser involves:

1. Writing the Rust code to translate the output format
2. Writing an inline test within the parser file for this parser
3. Adding new references to the parser within the codebase

### Parser tests

Each parser contains an inline test which typically can be run within VSCode by clicking "Run tests". VSCode simply runs (`cargo test --package qlty-check --lib -- parser::reek::test::parse --exact --show-output` e.g.) behind the scenes.

Typically, you'll update the input of the test by hand to match the tool's output.

And you can use `insta` to write the test output for you automatically, which helps prevents test failures from small character deltas in manually written output. Install `insta` with:

```
cargo install cargo-insta
```

Run `cargo insta review` to display and accept/reject test output.

## Plugin Tests

### Test setup

The first time you run any tests you'll need to run `npm install` at the top level of the `qlty-plugin` directory.

The snapshot tests are under the `linters/MY_PLUGIN/fixtures` directory. The my `linters/${MY_PLUGIN}/${MY_PLUGIN}.test.ts` file contains the call to the test function `linterCheckTest` with the name of the plugin as argument -- copying and adjusting a runner from an existing plugin is easiest.

Create the "fixtures" directory structure if one does not exist as well as:

- an example "target" typically named something like basic.in.py (extension dependent on plugin). This needs to be a file with issues the plugin identifies
- a \_\_snapshots\_\_ directory containing snaphotted output from a plugin run against the fixture file created above.
- in the snaphost directory, a snapshot file for each version / file named e.g. `basic_v0.2.2.shot` where the version matches the plugin version you're testing against.

### Running tests

Once that is setup you can run `npm test` to run the test suite or `npm test MY_PLUGIN` to run tests for a specific plugin.

```
npm test ${PLUGIN_NAME}
```

e.g. `npm test reek`

### Using Local Plugin Definitions

Qlty's default source for plugin definitions is the github.com/qltysh/qlty remote repository, which is fine for most cases, but is not always appropriate for plugin development where developers expect that their local plugin definitions to be used.

To ensure a run of Qlty is using your local plugin definitions, adjust the relevant `.qlty/qlty.toml` definition file to instead reference a local source:

```
[[source]]
name = "default"
directory = "/PATH_TO_QLTY_REPO/qlty-plugins/plugins"
```

## Debug

You can use `DEBUG=qlty:MY_PLUGIN` env variable to get debug logs which include a path to the sandbox for a given plugin or `DEBUG=qlty:*` for all of them.

The tests are run in sandboxes and they contain plenty of information required to debug a plugin, including but not limited to plugin scripts, outputs, qlty logs and invocation details.

`QLTY_PLUGINS_SANDBOX_DEBUG` can be passed as an enviornment variable to prevent teardown of sandboxes after a test run.

## Updating Versions

You can update the versions of plugins easily by updating `latest_version` and `known_good_version` attributes in their plugin definitions.

Also to test and add snapshots for the updated versions of the plugins, you can run the test suite with `QLTY_PLUGINS_LINTER_VERSION=KnownGoodVersion` flag, i-e `QLTY_PLUGINS_LINTER_VERSION=KnownGoodVersion npm test` and that will test with the updated `known_good_version` and add snapshots for it. (This currently does not fully support plugin drivers with multiple versions such as ESLint, as it will run all the drivers with `known_good_version`, so expect that test to fail.)

NOTE: It will add snapshots in .shot files, but will not validate with older snapshot files.

### Some gotchas and solutions

- You can use the logs in `.qlty` folder to ensure plugin is installing correctly. Make sure the runtime version and plugin version are correctly defined in `.qlty/qlty.toml` in case of installation issues.

- If the logs show error during invocation, most likely the `plugin.toml` file for the plugin has the issue.

- You can check the output from invocation in the `sandbox/tmp/invocation-FINGERPRINT.txt` file and verify the raw output from the plugin against your test case.

- If the output in invocations txt file is correct and something is still broken, you may want to check the CLI and parser of the plugin's output.

- Some plugins such as `Trivy` download their own specific databases independent of their versions which can cause conflict in local tests (which may be using an older db) and github action tests (which doesn't cache them).
