# Qlty Plugins Guide

This is a loose guide on how to add new plugins.

This plugins repo basically contains definitions and tests for various plugins supported by the main Qlty application.

## Toml file

The code to add support for a given plugin (`MY_PLUGIN`) is placed under `linters/MY_PLUGIN`.
The definitions for a given plugin can be found in `plugin.toml` file, which contains instructions on how to install the plugin, as well as how to execute it.

### Plugin Installation

For plugins which have a usable executable release on Github, those can be installed via Github releases using `[plugins.releases.MY_PLUGIN]`.
Example: `linters/gitleaks/plugin.toml`.

For plugins which require a runtime, such as python, they require the package and runtime option in their definitions `[plugins.definitions.MY_PLUGIN]`.
Example: `linters/black/plugin.toml`.

### Plugin Run

The script and the various options to run the plugin correctly can also be found in the `plugin.toml` file. `[plugins.definitions.MY_PLUGIN.drivers.lint]` contains the script, success codes, output, output format, batch and a few other options.

## Tests

The snapshot tests are under the `linters/MY_PLUGIN/fixtures` directory. The my `linters/MY_PLUGIN/MY_PLUGIN.test.ts` file contains the call to the test function `linterCheckTest` with the name of the plugin as argument.

The `.qlty/qlty.toml` file in the mian `qlty` repo needs to be updated when adding new plugins.

```
[sources.default]
directory = "/PATH_TO_PLUGINS/plugins"
```

Once that is setup you can run `npm test` to run the test suite or `npm test MY_PLUGIN` to run tests for a specific plugin.

## Debug

You can use `DEBUG=qlty:MY_PLUGIN` env variable to get debug logs which include a path to the sandbox for a given plugin or `DEBUG=qlty:*` for all of them.

The tests are run in sandboxes and they contain plenty of information required to debug a plugin, including but not limited to plugin scripts, outputs, qlty logs and invocation details.

`QLTY_PLUGINS_SANDBOX_DEBUG` can be passed as an enviornment variable to prevent teardown of sandboxes after a test run.

## Updating Versions

You can update the versions of plugins easily by updating `latest_version` and `known_good_version` attributes in their plugin definitions.

Also to test and add snapshots for the updated versions of the plugins, you can run the test suite with `QLTY_PLUGINS_LINTER_VERSION=KnownGoodVersion` flag, i-e `QLTY_PLUGINS_LINTER_VERSION=KnownGoodVersion npm test` and that will test with the updated `known_good_version` and add snapshots for it. (This currently does not fully support plugin drivers with multiple versions such as ESLint, as it will run all the drivers with `known_good_version`, so expect that test to fail.)

NOTE: It will add snapshots in .shot files, but will not validate with older snapshot files.

### Some gachas and solutions

- You can use the logs in `.qlty` folder to ensure plugin is installing correctly. Make sure the runtime version and plugin version are correctly defined in `.qlty/qlty.toml` in case of installation issues.

- If the logs show error during invocation, most likely the `plugin.toml` file for the plugin has the issue.

- You can check the output from invocation in the `sandbox/tmp/invocation-FINGERPRINT.txt` file and verify the raw output from the plugin against your test case.

- If the output in invocations txt file is correct and something is still broken, you may want to check the CLI and parser of the plugin's output.

- Some plugins such as `Trivy` download their own specific databases independent of their versions which can cause conflict in local tests (which may be using an older db) and github action tests (which doesn't cache them).
