# Qlty Plugins Documentation

Qlty supports a comprehensive plugin system that allows you to configure and use various linters, formatters, and security tools for your projects. This guide explains how to configure and use these plugins through your `qlty.toml` file.

## Table of Contents

- [Overview](#overview)
- [Plugin Configuration](#plugin-configuration)
- [Available Plugins](#available-plugins)
- [Configuration Options](#configuration-options)
- [Excluding Files and Patterns](#excluding-files-and-patterns)
- [Ignoring Specific Rules](#ignoring-specific-rules)
- [Plugin Triggers](#plugin-triggers)
- [Examples](#examples)

## Overview

Qlty plugins are configured in the `.qlty/qlty.toml` file in your project root. Each plugin represents a specific tool (linter, formatter, or security scanner) that can analyze your code.

## Plugin Configuration

Plugins are defined using the `[[plugin]]` array syntax in your `qlty.toml` file:

```toml
# Basic plugin configuration
[[plugin]]
name = "eslint"

# Plugin with specific version
[[plugin]]
name = "prettier"
version = "3.3.3"

# Plugin with custom configuration
[[plugin]]
name = "eslint"
version = "8.57.0"
package_file = ".qlty/configs/package.json"
enabled = true
```

## Available Plugins

Qlty includes 66 built-in plugins across various languages and tools:

### JavaScript/TypeScript
- **eslint** - JavaScript linter
- **prettier** - Code formatter
- **biome** - Fast formatter and linter
- **oxc** - JavaScript oxidation compiler
- **knip** - Find unused files, dependencies and exports
- **stylelint** - CSS/SCSS/Less linter
- **coffeelint** - CoffeeScript linter
- **tsc** - TypeScript compiler checks

### Python
- **flake8** - Python linter
- **black** - Python code formatter
- **ruff** - Fast Python linter
- **mypy** - Static type checker
- **bandit** - Security linter
- **radarlint-python** - Python code quality tool

### Ruby
- **rubocop** - Ruby static code analyzer
- **standardrb** - Ruby style guide
- **reek** - Code smell detector
- **ruby-stree** - Ruby formatter
- **radarlint-ruby** - Ruby code quality tool

### Go
- **gofmt** - Go formatter
- **golangci-lint** - Go linters aggregator
- **radarlint-go** - Go code quality tool

### Java
- **checkstyle** - Java code style checker
- **pmd** - Java source code analyzer
- **google-java-format** - Java formatter
- **radarlint-java** - Java code quality tool

### PHP
- **php-codesniffer** - PHP coding standards
- **php-cs-fixer** - PHP code formatter
- **phpstan** - PHP static analysis
- **radarlint-php** - PHP code quality tool

### Rust
- **clippy** - Rust linter
- **rustfmt** - Rust code formatter

### Swift
- **swiftformat** - Swift code formatter
- **swiftlint** - Swift style and conventions
- **stringslint** - Swift localization linter

### Security Tools
- **gitleaks** - Detect secrets in git repos
- **trufflehog** - Find credentials
- **osv-scanner** - Vulnerability scanner
- **trivy** - Security scanner
- **checkov** - Infrastructure as code scanner
- **semgrep** - Static analysis tool

### Infrastructure/DevOps
- **terraform** - Terraform formatter
- **tflint** - Terraform linter
- **hadolint** - Dockerfile linter
- **dockerfmt** - Dockerfile formatter
- **kube-linter** - Kubernetes YAML linter
- **actionlint** - GitHub Actions linter

### Other Tools
- **shellcheck** - Shell script analyzer
- **shfmt** - Shell formatter
- **yamllint** - YAML linter
- **sqlfluff** - SQL linter
- **markdownlint** - Markdown linter
- **vale** - Prose linter
- **ripgrep** - Pattern search tool
- **ast-grep** - AST-based search tool
- **redocly** - OpenAPI linter
- **prisma** - Database schema linter

## Configuration Options

### Basic Options

```toml
[[plugin]]
name = "eslint"              # Required: Plugin identifier
version = "9.0.0"           # Optional: Specific version (defaults to latest)
enabled = true              # Optional: Enable/disable plugin (default: true)
```

### Runtime Options

For Node.js-based plugins:
```toml
[[plugin]]
name = "eslint"
package_file = ".qlty/configs/package.json"  # Custom package.json location
runtime_version = "20.0.0"                   # Specific Node.js version
```

### Execution Control

```toml
[[plugin]]
name = "prettier"
triggers = ["save", "commit"]  # When to run this plugin
targets = ["**/*.js", "**/*.ts"]  # File patterns to analyze
config_file = ".prettierrc"       # Custom config file path
```

### Plugin-Specific Options

Many plugins support additional configuration options. Check the plugin's `plugin.toml` file for available options:

```toml
[[plugin]]
name = "eslint"
fix = true                    # Auto-fix issues
cache = true                  # Enable caching
max_warnings = 10            # Maximum warnings allowed
```

## Excluding Files and Patterns

You can exclude files from specific plugins:

```toml
# Exclude test files from security scanners
[[exclude]]
plugins = ["osv-scanner", "trufflehog", "gitleaks"]
file_patterns = ["**/*.test.js", "tests/**", "__tests__/**"]

# Exclude generated files from all plugins
[[exclude]]
file_patterns = ["dist/**", "build/**", "*.min.js"]
```

## Ignoring Specific Rules

To ignore specific rules from plugins:

```toml
# Ignore specific markdownlint rule in CHANGELOG
[[ignore]]
rules = ["markdownlint:MD024"]  # No duplicate headings
file_patterns = ["CHANGELOG.md"]

# Ignore multiple rules
[[ignore]]
rules = ["eslint:no-console", "eslint:no-debugger"]
file_patterns = ["scripts/**/*.js"]

# Ignore all rules from a plugin in specific files
[[ignore]]
plugins = ["rubocop"]
file_patterns = ["db/migrate/**/*.rb"]
```

## Plugin Triggers

Control when plugins run using triggers:

```toml
[[plugin]]
name = "osv-scanner"
triggers = ["manual", "build"]  # Only run manually or during CI builds
```

Available triggers:
- `save` - Run when files are saved (if IDE integration supports it)
- `commit` - Run during git pre-commit hook
- `push` - Run during git pre-push hook
- `manual` - Only run when explicitly invoked
- `build` - Run during CI/CD builds

## Examples

### Basic JavaScript/TypeScript Setup

```toml
config_version = "0"

[[plugin]]
name = "eslint"
version = "8.57.0"

[[plugin]]
name = "prettier"
version = "3.3.3"

[[plugin]]
name = "tsc"
triggers = ["commit", "build"]  # Type-check on commit and in CI
```

### Python Project with Security Scanning

```toml
config_version = "0"

[[plugin]]
name = "ruff"

[[plugin]]
name = "black"

[[plugin]]
name = "mypy"

[[plugin]]
name = "bandit"
triggers = ["push", "build"]  # Security scan before push and in CI

# Exclude test files from security scanning
[[exclude]]
plugins = ["bandit"]
file_patterns = ["tests/**", "test_*.py"]
```

### Multi-language Project

```toml
config_version = "0"

# JavaScript/TypeScript
[[plugin]]
name = "eslint"
targets = ["**/*.js", "**/*.jsx", "**/*.ts", "**/*.tsx"]

[[plugin]]
name = "prettier"

# Python
[[plugin]]
name = "ruff"
targets = ["**/*.py"]

# Go
[[plugin]]
name = "golangci-lint"
targets = ["**/*.go"]

# Infrastructure
[[plugin]]
name = "terraform"
targets = ["**/*.tf"]

[[plugin]]
name = "hadolint"
targets = ["**/Dockerfile*"]

# Security scanning (manual only to avoid slowing down development)
[[plugin]]
name = "trufflehog"
triggers = ["manual"]

[[plugin]]
name = "gitleaks"
triggers = ["manual"]
```

### Custom Configuration with Exclusions

```toml
config_version = "0"

# Define sources
[[source]]
name = "default"
default = true

# Global exclusions
exclude_patterns = [
    "vendor/**",
    "node_modules/**",
    "*.min.js",
    "coverage/**",
]

# Test file patterns
test_patterns = [
    "**/*.test.js",
    "**/*.spec.ts",
    "tests/**",
]

# Plugins
[[plugin]]
name = "eslint"
config_file = ".eslintrc.custom.js"

[[plugin]]
name = "prettier"
version = "3.3.3"

# Exclude specific files from prettier
[[exclude]]
plugins = ["prettier"]
file_patterns = ["*.config.js", "legacy/**"]

# Ignore specific ESLint rules in test files
[[ignore]]
rules = ["eslint:no-unused-expressions", "eslint:no-undefined"]
file_patterns = ["**/*.test.js", "**/*.spec.js"]
```

## Advanced Configuration

### Using Custom Package Files

For Node.js plugins, you can specify a custom `package.json`:

```toml
[[plugin]]
name = "eslint"
package_file = ".qlty/configs/package.json"
```

This allows you to:
- Use specific plugin versions
- Add ESLint plugins and configurations
- Manage dependencies separately from your project

### Configuring Code Smells Detection

```toml
[smells]
mode = "comment"  # How to handle code smells: "comment", "error", or "disabled"
```

### Runtime Configuration

```toml
[runtimes.enabled]
node = "20"
python = "3.11"
ruby = "3.2"
rust = "stable"
```

### Plugin Overrides and Custom Definitions

Qlty allows you to override existing plugins or define entirely new plugins directly in your `qlty.toml` file. This is useful for:
- Customizing how built-in plugins are invoked
- Adding support for proprietary or internal tools
- Modifying plugin behavior for your specific needs

#### Overriding Existing Plugins

You can override specific aspects of built-in plugins. For example, to increase PHP memory limit for PHPStan:

```toml
[plugins.definitions.phpstan.drivers.lint]
script = "php -d memory_limit=-1 ${linter}/phpstan analyze ${target} --error-format=json --level=9 ${autoload_script} --configuration=${config_file}"
copy_configs_into_tool_install = true
```

Or to customize PHP CS Fixer:

```toml
[plugins.definitions.php-cs-fixer.drivers.formatter]
script = "php -d memory_limit=-1 ${linter}/vendor/bin/php-cs-fixer fix --using-cache=no --show-progress=none --config=${config_file} ${target}"
batch = true
```

#### Defining Custom Plugins Inline

You can define entirely new plugins in your `qlty.toml`. Here's a complete example:

```toml
# Define a custom Kotlin linter
[plugins.definitions.ktlint]
runnable_archive_url = "https://github.com/pinterest/ktlint/releases/download/${version}/ktlint"
runtime = "java"
file_types = ["kotlin"]
suggested = "targets"
affects_cache = [".editorconfig"]

# Define the lint driver
[plugins.definitions.ktlint.drivers.lint]
script = "java -jar ${linter}/ktlint -l none --reporter=sarif ${target}"
success_codes = [0, 1]
output = "stdout"
output_format = "sarif"
cache_results = true
batch = true
suggested = "targets"

# Define the format driver
[plugins.definitions.ktlint.drivers.format]
script = "java -jar ${linter}/ktlint --format ${target}"
success_codes = [0]
output = "rewrite"
cache_results = true
batch = true
driver_type = "formatter"

# Enable the plugin
[[plugin]]
name = "ktlint"
version = "1.6.0"
```

#### Plugin Definition Structure

When defining a plugin, you can specify:

**Basic Configuration:**
```toml
[plugins.definitions.my-custom-tool]
runtime = "node"              # Runtime: node, python, ruby, rust, java, php, go
file_types = ["js", "ts"]     # File types this plugin handles
config_files = ["config.yml"] # Config files that affect the plugin
affects_cache = ["package.json"] # Files that invalidate cache when changed
```

**Driver Configuration:**
```toml
[plugins.definitions.my-custom-tool.drivers.lint]
script = "my-tool analyze ${target}"  # Command to run
output = "stdout"                     # Where output is written
output_format = "json"                # Format of the output
success_codes = [0]                   # Exit codes indicating success
batch = true                          # Can process multiple files at once
cache_results = true                  # Cache results for unchanged files
```

#### Available Variables in Scripts

- `${target}` - The file or directory being analyzed
- `${linter}` - Directory where the tool is installed
- `${config_file}` - Path to the configuration file
- `${tmpfile}` - Temporary file for output (when `output="tmpfile"`)
- `${version}` - Version of the tool

#### Driver Output Types

- `stdout` - Tool writes to standard output
- `stderr` - Tool writes to standard error
- `tmpfile` - Tool writes to a temporary file
- `rewrite` - Tool modifies files in place
- `pass_fail` - Tool only indicates success/failure

#### Advanced Features

**Version-Specific Behavior:**
```toml
[[plugins.definitions.my-tool.drivers.lint.version]]
version_matcher = ">=2.0.0"
script = "my-tool-v2 ${target} --new-format"

[[plugins.definitions.my-tool.drivers.lint.version]]
version_matcher = "<2.0.0"
script = "my-tool-v1 ${target} --legacy-format"
```

**Environment Variables:**
```toml
[[plugins.definitions.my-tool.environment]]
name = "TOOL_CONFIG_PATH"
value = "/custom/config/path"
```

**Custom Target Handling:**
```toml
[plugins.definitions.my-tool.drivers.lint]
# Pass parent directory instead of individual files
target = { type = "parent" }

# Or pass a literal path
target = { type = "literal", path = "." }

# Or pass parent directory containing a specific file
target = { type = "parent_with", path = "package.json" }
```

**Preparation Scripts:**
```toml
[plugins.definitions.my-tool.drivers.lint]
prepare_script = "mkdir -p ${linter}/cache"
script = "my-tool --cache-dir=${linter}/cache ${target}"
```

## Getting Help

- Run `qlty plugins list` to see all available plugins
- Run `qlty plugins info <plugin-name>` to see detailed information about a specific plugin
- Check the plugin's `plugin.toml` file in the `qlty-plugins` repository for all configuration options
- Visit the [Qlty documentation](https://docs.qlty.sh) for more information

## Troubleshooting

### Plugin Not Found

If a plugin is not found, ensure:
1. The plugin name is spelled correctly
2. The plugin is available in your configured sources
3. Your `.qlty/qlty.toml` file is in the project root

### Plugin Not Running

If a plugin is not running:
1. Check that it's enabled (`enabled = true` or not set to `false`)
2. Verify the triggers match your workflow
3. Ensure the target file patterns match your files
4. Check for exclusion rules that might be preventing it from running

### Version Conflicts

If you encounter version conflicts:
1. Specify exact versions in your configuration
2. Use a custom `package_file` for Node.js plugins
3. Check compatibility between the plugin and its runtime version