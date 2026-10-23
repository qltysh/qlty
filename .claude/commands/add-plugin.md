# Add a New Linter Plugin to Qlty

Add a new linter plugin for: $ARGUMENTS

## Overview

You are adding a new linter/formatter plugin to the qlty codebase. This requires changes in multiple locations.

## Step 1: Research the Linter

First, research the linter to understand:

1. **What language/files does it lint?** (e.g., Python, JavaScript, Dockerfile)
2. **What is the package manager/runtime?** Options:
   - `runtime = "node"` for npm packages
   - `runtime = "python"` for pip/uv packages
   - `runtime = "ruby"` for gem packages
   - `runtime = "go"` for Go tools
   - Use `releases` with GitHub downloads for standalone binaries
3. **What is the JSON output format?** Run the linter with `--help` or check docs for JSON output flags
4. **What are the config files?** (e.g., `.eslintrc`, `ruff.toml`)
5. **What is the latest version?**

Use web search to find:

- Official documentation for JSON output format
- GitHub repository for releases
- Example JSON output structure

## Step 2: Create the Plugin Directory

Create directory: `qlty-plugins/plugins/linters/<linter-name>/`

### Files to create:

#### 2a. `plugin.toml`

Use existing plugins as reference. Key patterns:

**For runtime-based plugins (npm, pip, gem):**

```toml
config_version = "0"

[plugins.definitions.<linter_name>]
runtime = "<node|python|ruby>"
package = "<package-name>"
file_types = ["<filetype>"]  # See qlty-config/src/config/language.rs for valid types
latest_version = "<version>"
known_good_version = "<version>"
version_command = "<linter> --version"
config_files = ["<config-file>"]
description = "<description>"

[plugins.definitions.<linter_name>.drivers.lint]
script = "<linter> <json-output-flags> ${target}"
success_codes = [0, 1]  # Include exit codes that indicate "issues found but ran successfully"
output = "stdout"  # or "tmpfile" if using ${tmpfile}
output_format = "<format>"  # Use "sarif" if supported, otherwise create custom parser
batch = true
cache_results = true
suggested = "targets"
output_missing = "parse"
```

**For binary releases from GitHub:**

```toml
config_version = "0"

[plugins.releases.<linter_name>]
github = "<owner>/<repo>"
download_type = "executable"  # or "targz" or "zip"

[plugins.definitions.<linter_name>]
releases = ["<linter_name>"]
file_types = ["<filetype>"]
# ... rest same as above
```

#### 2b. `README.md`

```markdown
# <Linter Name>

[<Linter Name>](https://github.com/<owner>/<repo>) <description>.

## Enabling <Linter Name>

Enabling with the `qlty` CLI:

\`\`\`bash
qlty plugins enable <linter-name>
\`\`\`

Or by editing `qlty.toml`:

\`\`\`toml
[plugins.enabled]
<linter-name> = "latest"
\`\`\`

## Configuration files

- [`<config-file>`](link-to-docs)

## Links

- [<Linter Name> on GitHub](https://github.com/<owner>/<repo>)
- [<Linter Name> plugin definition](https://github.com/qltysh/qlty/tree/main/plugins/linters/<linter-name>)

## License

<Linter Name> is licensed under the [<License>](license-url).
```

#### 2c. `<linter-name>.test.ts`

```typescript
import { linterCheckTest } from "tests";

linterCheckTest("<linter-name>", __dirname);
```

#### 2d. `fixtures/` directory

Create `fixtures/basic.in.<ext>` (for single-file fixtures) or `fixtures/basic.in/` (for multi-file fixtures) with:

- A sample file that will produce lint/format issues
- Any required config file for the linter

**For formatters:** Create a file with formatting issues (e.g., inconsistent spacing, wrong indentation).
**For linters:** Create a file with lint violations the tool will detect.

The test framework will automatically generate snapshots in `fixtures/__snapshots__/` when tests run.

## Step 3: Choose Output Format

You have three options for parsing linter output:

### Option A: SARIF (preferred if supported)

If the linter supports SARIF output, use it directly - no custom parser needed:

```toml
output_format = "sarif"
```

### Option B: Custom Parser (for JSON output)

If the linter outputs JSON, create a custom parser (see below).

### Option C: Regex Parser (for simple text output)

If the linter outputs text in a consistent format (e.g., `file:line:col: code message`), use the built-in regex parser. No Rust code needed!

```toml
output_format = "regex"
output_regex = "((?P<path>.*):(?P<line>\\d+):(?P<col>\\d+): (?P<code>\\S+) (?P<message>.+))"
```

**Required named capture groups:**

- `path` - File path
- `line` - Line number
- `code` - Rule/error code
- `message` - Error message

**Optional named capture groups:**

- `col` - Column number
- `severity` - Maps to level (error→high, warning→medium, info/note→low)
- `end_line` - End line for range
- `end_col` - End column for range

**Examples from existing plugins:**

flake8 (`file:line:col: CODE message`):

```toml
output_regex = "((?P<path>.*):(?P<line>-?\\d+):(?P<col>-?\\d+): (?P<code>\\S+) (?P<message>.+))\n"
```

yamllint (`file:line:col: [severity] message (code)`):

```toml
output_regex = "((?P<path>.*):(?P<line>\\d+):(?P<col>\\d+): \\[(?P<severity>.*)\\] (?P<message>.*) \\((?P<code>.*)\\))"
```

GitHub Actions format (`::severity title=code,file=path,line=N,...::message`):

```toml
output_regex = "::(?P<severity>[^ ]+) title=(?P<code>[^,]+),file=(?P<path>[^,]+),line=(?P<line>\\d+),endLine=(?P<end_line>\\d+),col=(?P<col>\\d+),endColumn=(?P<end_col>\\d+)::(?P<message>.+)"
```

See existing regex plugins: `flake8`, `yamllint`, `vale`, `oxc`, `redocly`, `swiftlint`, `stringslint`

---

## Step 3B: Create Custom Parser (for Option B - JSON output)

### 3a. Create parser file: `qlty-check/src/parser/<linter_name>.rs`

```rust
use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct <LinterName> {}

// Define structs matching the linter's JSON output
#[derive(Debug, Deserialize)]
struct <LinterName>Message {
    // ... fields matching JSON structure
}

impl Parser for <LinterName> {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let messages: Vec<<LinterName>Message> = serde_json::from_str(output)?;

        for message in messages {
            let issue = Issue {
                tool: "<linter-name>".into(),
                message: message.message,
                category: Category::Lint.into(),
                level: Level::Medium.into(),
                rule_key: message.code,
                location: Some(Location {
                    path: message.file,
                    range: Some(Range {
                        start_line: message.line,
                        start_column: message.column,
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            };
            issues.push(issue);
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        <sample JSON output>
        "###;

        let issues = <LinterName>::default().parse("<linter-name>", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        "###);
    }
}
```

### 3b. Register parser in `qlty-check/src/parser.rs`

Add:

```rust
pub mod <linter_name>;
```

### 3c. Add OutputFormat variant in `qlty-config/src/config/plugin.rs`

In the `OutputFormat` enum (around line 505), add:

```rust
#[serde(rename = "<linter_name>")]
<LinterName>,
```

In the `Display` impl (around line 571), add:

```rust
OutputFormat::<LinterName> => write!(f, "<linter_name>"),
```

### 3d. Add parser mapping in `qlty-check/src/executor/driver.rs`

In the `parser()` function (around line 417), add:

```rust
OutputFormat::<LinterName> => Box::new(<LinterName> {}),
```

And add the import at the top:

```rust
use crate::parser::<linter_name>::<LinterName>;
```

## Step 4: Build and Test

Run these commands to verify everything works:

```bash
# Type check (for Rust code changes only - skip if no parser was added)
cargo check

# Run plugin tests (this is the main test command)
cd qlty-plugins/plugins && npm test -- --testNamePattern="<linter_name>"

# Format and lint
qlty fmt
qlty check --level=low --fix
```

**Note:** Plugin tests are JavaScript/TypeScript tests run with Jest in `qlty-plugins/plugins/`, NOT Rust tests. The test will automatically install the linter and generate snapshots on first run.

## Reference Examples

Look at these existing plugins for patterns:

- Simple Python formatter: `qlty-plugins/plugins/linters/black/` (Python/pip, formatter)
- Simple Python linter: `qlty-plugins/plugins/linters/ruff/` (Python/pip, linter)
- Markdown linter: `qlty-plugins/plugins/linters/markdownlint/` (Node/npm, both lint and format drivers)
- Complex with versions: `qlty-plugins/plugins/linters/eslint/` (Node/npm)
- Binary release: `qlty-plugins/plugins/linters/hadolint/` (GitHub release)
- Go runtime: `qlty-plugins/plugins/linters/golangci-lint/`
- Security scanner: `qlty-plugins/plugins/linters/trivy/`

## Valid file_types

**IMPORTANT:** Read `qlty-config/default.toml` to see all available file types and their glob patterns.

Look for `[file_types.<name>]` sections in that file. Common examples: `javascript`, `typescript`, `python`, `ruby`, `go`, `rust`, `docker`, `yaml`, `json`, `shell`, `terraform`, `lockfile`, `ALL`.

**To add a new file type**, add it to `qlty-config/default.toml`:

```toml
[file_types.my_new_type]
globs = ["*.ext", "*.other"]
```

## Notes

- If the linter supports SARIF output, prefer that over a custom parser (`output_format = "sarif"`)
- Use `output = "tmpfile"` with `${tmpfile}` in script if the linter writes to a file
- Set `security = true` for security-focused tools
- Use `batch = true` to allow processing multiple files at once
- The `suggested` field controls how the plugin is suggested during `qlty init`

## Formatter vs Linter Plugins

**Formatters** (like `black`, `prettier`, `mdformat`):

- Use `driver_type = "formatter"`
- Use `output = "rewrite"` (tool modifies files in place)
- Typically only need `success_codes = [0]`
- No `output_format` needed - the test framework detects formatting changes

**Linters** (like `eslint`, `ruff`, `markdownlint`):

- Use `driver_type = "linter"`
- Use `output = "stdout"` or `output = "stderr"`
- Need `output_format` to parse the linter's output
- Often need `success_codes = [0, 1]` (1 = issues found but ran successfully)

**Tools with both:** Some tools like `markdownlint` can have both a `lint` and `format` driver defined.
