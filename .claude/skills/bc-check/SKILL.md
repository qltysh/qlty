---
name: bc-check
description: Analyze a qlty pull request (or the current branch) for backwards-incompatible behavior changes. Use when the user asks to review a PR for backwards compatibility, BC breaks, or regression risk — especially before merging changes to the qlty CLI, qlty.toml schema, output formats, or command behavior. Produces a short findings report.
model: opus
---

# Backwards-compatibility review for a qlty PR

## Qlty runtime contexts

Qlty is run in three distinct runtime contexts, each with different sensitivity to backwards-incompatible changes.

Note that it is possible for a change to impact more than one of these runtime contexts simultaneously.

### 1. Developers running qlty locally

Backwards-incompatible changes here would impact local developers running qlty on their own machines. Individual developers download a qlty CLI version and often don't run `qlty upgrade` regularly, so their local version can be months behind main. Common commands that a developer runs include `qlty check`, `qlty fmt`, and `qlty init`.

They run these against a `.qlty/qlty.toml` checked into their repo. Teammates' CLIs and Qlty Cloud read the same file, each possibly on a different qlty version.

A backwards-incompatible change here hits one developer at a time and is annoying but recoverable, so it is usually medium severity.

### 2. Customers running coverage sub-commands in their CI

Customers run `qlty coverage publish` and `qlty coverage complete` inside
their CI pipelines on every build. Pipelines are hard to change in a hurry:
a backwards-incompatible change here fails every build for every customer on the affected
version, often silently (they don't notice until coverage stops showing up,
or a release is blocked by red CI). A confirmed break here is high severity. Be especially sensitive to anything that changes exit codes, error vs. warning behavior, or required inputs on `qlty coverage *` commands.

### 3. Qlty Cloud's hosted builds

Qlty Cloud runs builds on behalf of customers and invokes the qlty CLI
directly. The commands currently relied on include `qlty sources fetch`,
`qlty config validate`, `qlty fmt --skip-source-fetch`,
`qlty check --install-only --skip-source-fetch` (with `--skip-errored-plugins`
and `--upstream` or `--all`), `qlty init`, `qlty install`, and `qlty build`.
A backwards-incompatible change that affects any of these commands or flags,
or the config/source formats they depend on, causes a **Qlty
Cloud outage** — every customer's builds fail at once until we can ship a
coordinated fix. A confirmed break here is high severity, and the
recommendation must say the PR needs a coordinated Qlty Cloud update before
it ships.

## Steps for Backwards Compatibility Check

### Step 1 — Load the diff and context

Resolve the target from the user's input:

- PR number / PR URL → `gh pr view <N> --json title,body,headRefName,baseRefName,author,files` and `gh pr diff <N>`.
- Branch name → `git fetch origin main && git diff origin/main...<branch>` and `git log origin/main..<branch> --oneline`.
- No input → current branch: `git fetch origin main && git diff origin/main...HEAD` and `git log origin/main..HEAD --oneline`.

Load and understand the changeset, including the context provided by commit messages, PR descriptions, and any relevant discussion. Intent matters: a PR captioned "fix bug where X silently failed" is the #2762 pattern, where a silent failure became a hard error. That's a red flag unless the stricter behavior is behind an opt-in flag.

### Step 2 — Evaluate the impact on runtime contexts

Evaluate the pull request for backwards-incompatible changes. Ask what a user with a setup that worked six months ago sees after upgrading. For each finding, determine:

- **which runtime contexts are affected** (local developers, CI customers, Qlty Cloud)
- **severity level** of the impact (high, medium, low), where high means widespread breakage for one or more runtime contexts, medium means noticeable impact but workarounds exist, and low means minor change or informational. Use the per-context guidance above: a confirmed break for CI customers or Qlty Cloud is high.
- **where** the behavior changes, as `file:line`
- **what breaks**, for whom, and under what circumstances
- **recommendation** for addressing the issue (if applicable)

Only report findings you can tie to a specific changed line.

### Step 3 — Output the report

Don't narrate the investigation. Your reply is the report below and nothing else.

```
## Backwards-compatibility review

**Target:** <PR #N / branch <name> / current branch>
**Verdict:** <safe / risky — needs mitigation / blocker — do not merge as-is>
**Affected runtime contexts:** <local developers / CI customers / Qlty Cloud — contexts with findings, or none>

- **<High / Medium / Low>** · <affected contexts> · `path/to/file.rs:123` — <what breaks, for whom, under what circumstances>. Fix: <recommendation>.

<repeat, most severe first>
```

Keep each finding to a few sentences. The verdict is **blocker** if any finding is high, **risky** if any finding is medium, and **safe** otherwise. If there are zero findings, still emit the report with "Verdict: safe" and "No findings." in place of the list.

A finding at the right level of detail:

- **High** · CI customers, Qlty Cloud · `qlty-cli/src/commands/coverage/publish.rs:185` — Customers with `[[source]] repository = ...` entries now hit exit 1, because `workspace.load_config` propagates the error where `unwrap_or_default()` used to swallow it. Qlty Cloud hits the same error through `qlty sources fetch`. Fix: keep the old fallback and log a warning; ship a coordinated Qlty Cloud update before making it an error.

## Scope / non-goals

- Don't critique code quality, test coverage, or style — only compatibility.
- Don't flag purely additive changes (new optional flag, new optional field,
  new subcommand) unless they shadow or conflict with existing behavior.
- Internal refactors (moving code between private crates, renaming private
  fns) are out of scope unless they change observable behavior.
- Don't post to GitHub, don't edit files, don't commit — just analyze and
  return the report.
