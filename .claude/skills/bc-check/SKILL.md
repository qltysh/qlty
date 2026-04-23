---
name: bc-check
description: Analyze a qlty pull request (or the current branch) for backwards-incompatible behavior changes. Use when the user asks to review a PR for backwards compatibility, BC breaks, or regression risk — especially before merging changes to the qlty CLI, qlty.toml schema, output formats, or command behavior. Produces a structured findings report.
---

# Backwards-compatibility review for a qlty PR

Your job is to identify whether the target PR (or branch) introduces
behavior changes that would break existing qlty users when they upgrade,
and return a structured findings report in your reply.

The target is whatever the user named (a PR number, PR URL, or branch). If
the user didn't specify anything, default to the current branch vs.
`origin/main`.

## Who gets hurt by a BC break in this repo

Keep these three user segments in mind. The severity of a finding depends
on which segment it affects.

### 1. Developers running qlty locally

Individual developers download a qlty CLI version and often don't run
`qlty upgrade` regularly, so their local version can be months behind main.
They mostly invoke `qlty check` and `qlty fmt` against a `.qlty/qlty.toml`
that's checked into their repo. A BC break here causes friction for one
developer at a time — annoying but recoverable. Severity baseline: **risky**
unless unusually disruptive.

### 2. Customers running qlty in their CI

Customers run `qlty coverage publish` and `qlty coverage complete` inside
their CI pipelines on every build. Pipelines are hard to change in a hurry:
a BC break here fails every build for every customer on the affected
version, often silently (they don't notice until coverage stops showing up,
or a release is blocked by red CI). This is the **highest-stakes** category.
Severity baseline for a confirmed break: **blocker**. Be especially
sensitive to anything that changes exit codes, error vs. warning behavior,
or required inputs on `qlty coverage *` commands.

### 3. Qlty Cloud's hosted builds

Qlty Cloud runs builds on behalf of customers and invokes the qlty CLI
directly. The commands currently relied on include `qlty sources fetch`,
`qlty config validate`, `qlty fmt --skip-source-fetch`, `qlty init`,
`qlty install`, and `qlty build`. A BC break that affects any of these
commands, or the config/source formats they depend on, causes a **Qlty
Cloud outage** — every customer's builds fail at once until we can ship a
coordinated fix. Severity baseline for a confirmed break here: **blocker**,
with extra urgency in the report.

A single finding can hit more than one of these. Call out each affected
segment explicitly.

## Step 1 — Load the diff

Resolve the target from the user input:

- PR number / PR URL → `gh pr view <N> --json title,body,headRefName,baseRefName,author,files` and `gh pr diff <N>`.
- Branch name → `git fetch origin main && git diff origin/main...<branch>` and `git log origin/main..<branch> --oneline`.
- No input → current branch: `git fetch origin main && git diff origin/main...HEAD` and `git log origin/main..HEAD --oneline`.

Read the PR description and commit messages first. Intent matters: a PR
captioned "fix bug where X silently failed" is exactly the #2762 pattern —
that's a red flag unless the fix is gated behind an opt-in flag.

## Step 2 — For each category below, check the diff

For every category the PR touches, produce a finding with **file:line**,
**before behavior**, **after behavior**, **which of the three user segments
breaks**, and **severity** (blocker / risky / note). Don't just point at a
diff — walk through what an existing user actually experiences.

### a. Behavior changes on existing command paths

The #2762 pattern. Silent failures becoming hard errors, added network/IO
on a command's hot path, new validation that rejects previously-accepted
input. Grep signals: new `bail!`, `ensure!`, `return Err`, `?` on
previously-swallowed results, `unwrap_or_default()` being removed,
`fetch_sources`, `load_config`, `validate_` added to a command flow.
Ask: _what does a user with a 6-month-old working setup see now?_

### b. CLI flag surface (qlty-cli/src/commands/\*\*)

Any `#[arg]` / `#[clap]` removed or renamed; `default_value` changed;
optional↔required changes; `value_enum` variant removed; subcommand
removed or renamed; new positional argument shifting existing args;
`hide = true` flags being removed (flag but rarely a blocker).

### c. qlty.toml schema (qlty-config/src/config/\*\*)

serde fields renamed or removed without `#[serde(alias)]`; `Option<T>` →
`T`; type changes; `#[serde(deny_unknown_fields)]` added; default changes;
`config_version` handling that would reject a `config_version = "0"` file;
TOML merge semantics in `qlty-config/src/toml_merge.rs`.

### d. Output formats

`qlty-coverage/src/print.rs`, any `print_*_as_json`, `serde_json::to_*`,
`Serialize` derives on output types. Field renames/removals, numeric
format changes, wire format changes in `qlty-types` protos. Also text
output that scripts parse (CI log patterns, summary lines).

### e. Exit codes

`std::process::exit`, `CommandError` vs `CommandSuccess`, `?` propagation
into `main`. A command starting to return non-zero where it used to return
zero (or vice versa) is a blocker for CI-segment users.

### f. Environment variables

`std::env::var` call sites. `QLTY_*` renamed or removed; precedence order
between env and flag changed; auth token resolution in
`qlty-coverage/src/token.rs`.

### g. Source fetching / cache layout

Changes to `.qlty/sources/` / `.qlty/cache/` / `~/.qlty/` layout;
`GitSource` / `LocalSource` / `DefaultSource` / `SourcesList` changes to
what counts as cached, when fetches happen, credential resolution, source
resolution order.

### h. Plugin behavior

`plugin.toml` schema changes in `qlty-plugins/plugins/*/plugin.toml` or
the loader in `qlty-check`; `drivers`, `prepare_script`, `affects_cache`
semantics; download URL patterns (breaking these kills offline caches).

### i. Telemetry / logging defaults

Log levels changing `debug!` → `warn!` / `error!` (visible in CI logs);
event shape changes for Sentry/analytics (lower severity).

## Step 3 — Trace the worst-case user for each segment

For the riskiest findings, walk through a concrete scenario _per affected
segment_. Write out the `.qlty/qlty.toml` snippet or CLI invocation that
would break, say what the user saw before, and what they see now. This is
the step that catches incidents like #2762 — it forces you past "the tests
pass" into "what does each segment hit?"

For segment 3 (Qlty Cloud), check whether the change would affect any of
the qlty commands listed above that Qlty Cloud relies on. If the answer is
yes, the PR needs a coordinated Qlty Cloud update before it ships — flag
that explicitly in the mitigation suggestion.

## Step 4 — Check for mitigations

For each finding, does the PR:

- Gate the new strict behavior behind an opt-in flag (like
  `--skip-source-fetch`), default preserving old behavior?
- Add `#[serde(alias = "old_name")]` on renamed fields?
- Bump `config_version` and gate breaking checks on the new version?
- Emit a warning (not an error) with a deprecation window?
- Call out a coordinated Qlty Cloud update if segment 3 is affected?
- Update CHANGELOG / migration docs?

No mitigation + confirmed break ⇒ escalate to **blocker**.

## Step 5 — Output the report

Return the findings as the last thing in your reply, using this shape.
Nothing else should come after it — the report is the skill's output.

```
## Backwards-compatibility review

**Target:** <PR #N / branch <name> / current branch>
**Verdict:** <safe / risky — needs mitigation / blocker — do not merge as-is>
**Affected user segments:** <local devs / CI users / Qlty Cloud — segments with findings>

### Findings

**[severity]** `path/to/file.rs:123` — <category>
- Before: ...
- After: ...
- Who breaks: <segment(s) + concrete scenario>
- Suggested mitigation: ...

<repeat, most severe first>

### Categories checked and clear
<list of categories from Step 2 where nothing was found>

### Notes for reviewer
<optional: anything the reviewer should verify manually>
```

If there are zero findings, still emit the report with "Verdict: safe" and
the "Categories checked and clear" list, so the caller knows the check ran.

## Scope / non-goals

- Don't critique code quality, test coverage, or style — only compatibility.
- Don't flag purely additive changes (new optional flag, new optional field,
  new subcommand) unless they shadow or conflict with existing behavior.
- Internal refactors (moving code between private crates, renaming private
  fns) are out of scope unless they change observable behavior.
- Don't post to GitHub, don't edit files, don't commit — just analyze and
  return the report.
- Be specific. "This might break something" is useless. "Customers with
  `[[source]] repository = ...` entries hit exit 1 now because
  `workspace.load_config` at `coverage/publish.rs:185` propagates the error
  where `unwrap_or_default()` used to swallow it — this breaks segment 2
  (CI) and segment 3 (Qlty Cloud, which invokes `qlty sources fetch`)" is
  the bar.
