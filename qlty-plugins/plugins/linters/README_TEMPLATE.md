# {{PluginName}}

[{{PluginName}}]({{upstream_github_url}}) is {{one_line_description}}.

## Enabling {{PluginName}}

Enabling with the `qlty` CLI:

```bash
qlty plugins enable {{plugin_id}}
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "{{plugin_id}}"

# OR pin to a specific version
[[plugin]]
name = "{{plugin_id}}"
version = "X.Y.Z"
```

## Auto-enabling

<!-- Remove this section if the plugin is never auto-enabled. -->

{{PluginName}} will be automatically enabled by `qlty init` if {{auto_enable_condition}}.

Examples:

- "a `.rubocop.yml` configuration file is present"
- "Python files are present"

## Languages and file types

<!-- List the languages or file extensions this linter targets. -->

{{PluginName}} analyzes: {{languages_or_file_types}}.

Examples:

- "Ruby (`.rb`)"
- "JavaScript and TypeScript (`.js`, `.ts`, `.jsx`, `.tsx`)"
- "Shell scripts (`.sh`, `.bash`)"
- "All files (secret scanning)"

## Configuration files

<!-- Remove this section if the plugin has no user-facing config file. -->

- [`{{config_filename}}`]({{config_docs_url}})

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running {{PluginName}}.

## Extra dependencies

<!-- Remove this section if no extra packages or runtimes are needed. -->

{{PluginName}} requires {{extra_dependencies}}.

Examples:

- "Node.js (installed separately or via a package manager)"
- "A `Gemfile` with the relevant gems for Rubocop extensions"

## Troubleshooting

**{{PluginName}} is not running on my files.**
Check that the file extensions match what {{PluginName}} supports (see _Languages and file types_ above). If you use a non-standard extension, you may need to configure `{{plugin_id}}` in your `qlty.toml`.

**I see "plugin not found" or version errors.**
Run `qlty plugins list` to confirm {{PluginName}} is enabled, then `qlty upgrade {{plugin_id}}` to fetch the latest version.

**My existing `{{config_filename}}` is being ignored.**
Move it into `.qlty/configs/` — Qlty looks there first. Alternatively, verify the file is not listed in `exclude_patterns` in `qlty.toml`.

<!-- Add any plugin-specific troubleshooting notes here. -->

## Links

- [{{PluginName}} on GitHub]({{upstream_github_url}})
- [{{PluginName}} plugin definition](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters/{{plugin_id}})
- [{{PluginName}} releases]({{upstream_github_url}}/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty-plugins/tree/main/plugins/linters)

## License

{{PluginName}} is licensed under the [{{license_name}}]({{license_url}}).
