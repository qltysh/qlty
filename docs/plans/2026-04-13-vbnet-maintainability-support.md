# VBNet Maintainability Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use trycycle-executing to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class VB.NET maintainability support so `*.vb` files are discovered by the built-in smells and metrics pipeline, emit `LANGUAGE_VBDOTNET`, and behave correctly for VB.NET's case-insensitive identifier semantics.

**Architecture:** Add a normal built-in `vbnet` language implementation in `qlty-analysis`, then add one generic identifier-normalization hook to the shared language/metrics path so VB.NET can stay case-insensitive without introducing VB-specific branches in the smells executors. Prove the feature primarily through real CLI snapshot tests, with targeted unit coverage for the shared enum/normalization seams and the VB.NET parser/query contract.

**Tech Stack:** Rust workspace, tree-sitter, VB.NET grammar crate or binding compatible with `tree-sitter = 0.22.6`, trycmd CLI snapshots, `cargo test`, `qlty fmt`, `qlty check`

---

## Desired End State

- `qlty build --no-plugins --print` analyzes `.vb` files and emits structure issues, duplication issues, and stats for VB.NET sources.
- `qlty metrics --all --json` emits file and function stats for `.vb` files with `language = "LANGUAGE_VBDOTNET"`.
- `qlty smells --all --no-snippets --json` reports real VB.NET duplication and structure issues, but duplicated `Imports` lines are filtered out the same way Java `import` and C# `using` lines are today.
- The internal language name is `vbnet` everywhere in this change. Do not alias to `vb`, and do not emit `LANGUAGE_VB`.
- Case-insensitive VB.NET semantics are preserved in the shared metrics path: `Me` and `me` are treated identically, `New` and `new` are treated identically for constructor handling, and member references with case variants do not fragment LCOM or field counts.

## Contracts And Invariants

- The built-in maintainability path is the target here. Do not add plugin-only or metrics-only stopgaps.
- `qlty-analysis::lang::from_str("vbnet")` must succeed anywhere the existing built-in languages do.
- `qlty_types::language_enum_from_name("vbnet")` must return `analysis::v1::Language::Vbdotnet`.
- `.vb` discovery must come from `[language.vbnet]` in `qlty-config/default.toml`.
- `Imports` filtering belongs in `qlty-config/default.toml`, not in custom duplication executor branches.
- Keep `qlty-smells` generic. If VB.NET needs special identifier comparison rules, express them through the `Language` trait and shared metrics helpers rather than a VB.NET-only executor fork.
- Do not add `[file_types.vbnet]` in this task unless an actual consumer in this change needs it. The user asked for built-in maintainability/language support for the smells/metrics pipeline.
- Prefer `arborium-vb` if it is compatible with the workspace's `tree-sitter = 0.22.6`. If it is not, choose a compatible VB.NET tree-sitter binding or vendor a thin local binding without changing the public `vbnet` / `LANGUAGE_VBDOTNET` contract.

## File Structure

- Modify `Cargo.toml`
  Add the chosen compatible VB.NET grammar dependency at the workspace level.
- Modify `Cargo.lock`
  Record the new grammar dependency and its transitive dependencies.
- Modify `qlty-analysis/Cargo.toml`
  Consume the new grammar dependency in the analysis crate.
- Modify `qlty-analysis/src/lang.rs`
  Register `vbnet`, export the module, and add a shared identifier-normalization hook for case-insensitive languages.
- Create `qlty-analysis/src/lang/vbnet.rs`
  Implement the VB.NET `Language` trait, query definitions, identifier extraction, constructor handling, identifier normalization, and focused unit tests.
- Modify `qlty-config/default.toml`
  Add `[language.vbnet]` with `globs = ["*.vb"]` and a duplication filter for `Imports`.
- Modify `qlty-types/src/lib.rs`
  Map `vbnet` to `analysis::v1::Language::Vbdotnet`.
- Create `qlty-types/tests/language_enum.rs`
  Lock down the `vbnet -> Vbdotnet` mapping directly.
- Modify `qlty-smells/src/metrics/metrics/lcom.rs`
  Normalize identifier comparisons so constructors, `self`, method groups, and field groups work for case-insensitive languages.
- Modify `qlty-smells/src/metrics/metrics/cognitive.rs`
  Normalize recursive-call comparisons so VB.NET self-recursion is not case-sensitive.
- Modify `qlty-smells/src/metrics/metrics/fields.rs`
  Normalize deduplicated field names so VB.NET member casing does not inflate field counts.
- Modify `qlty-cli/tests/lang.rs`
  Add `vbnet_tests()` so the new language suite runs with the existing integration harness.
- Create `qlty-cli/tests/lang/vbnet/basic.toml`
- Create `qlty-cli/tests/lang/vbnet/basic.stdout`
- Create `qlty-cli/tests/lang/vbnet/basic.in/.gitignore`
- Create `qlty-cli/tests/lang/vbnet/basic.in/.qlty/qlty.toml`
- Create `qlty-cli/tests/lang/vbnet/basic.in/BooleanLogic.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/FileComplexity.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/FunctionComplexity.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Identical.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Lines.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Members.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/NestedControl.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Parameters.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Returns.vb`
  Keep this fixture repo close to the existing C# maintainability fixture, but make `Members.vb` prove the mixed-case `Me` / member / constructor semantics that the shared metrics path must preserve.
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.toml`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.stdout`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.in/.gitignore`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.in/.qlty/qlty.toml`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.in/example.vb`
  Add a focused metrics regression that proves the enum and basic stats contract on the user-facing `metrics` command.
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.toml`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.stdout`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/.qlty/qlty.toml`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical.vb`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical2.vb`
  Add a focused smells regression that proves duplicated `Imports` lines are filtered while executable duplication still remains eligible.
- Modify `README.md`
  Add VB.NET to the built-in maintainability table and to the metrics language list without dropping any concurrent language additions.

## Strategy Gate

The original direction was close, but it missed one architectural requirement: VB.NET is case-insensitive while the shared smells/metrics path currently compares `self`, constructors, field names, and recursive-call names case-sensitively. If execution followed that plan literally, a polished implementation could still ship incorrect LCOM, field, and recursion behavior for valid VB.NET code. The direct path is still the right path, but it needs one generic trait-level identifier-normalization seam in addition to the new language module. Once that seam is in place, the rest of the feature should stay idiomatic: register `vbnet`, implement the parser/query contract, and prove the whole pipeline through real CLI snapshots.

### Task 1: Register VB.NET And Add Shared Identifier Normalization

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `qlty-analysis/Cargo.toml`
- Modify: `qlty-analysis/src/lang.rs`
- Create: `qlty-analysis/src/lang/vbnet.rs`
- Modify: `qlty-config/default.toml`
- Modify: `qlty-types/src/lib.rs`
- Create: `qlty-types/tests/language_enum.rs`
- Modify: `qlty-smells/src/metrics/metrics/lcom.rs`
- Modify: `qlty-smells/src/metrics/metrics/cognitive.rs`
- Modify: `qlty-smells/src/metrics/metrics/fields.rs`

- [ ] **Step 1: Identify or write the failing test**

Add the smallest seam tests before doing the full VB.NET query work:

- Create `qlty-types/tests/language_enum.rs` with a direct assertion that `language_enum_from_name("vbnet")` returns `analysis::v1::Language::Vbdotnet`.
- Extend `qlty-analysis/src/lang.rs` `language_names()` so `from_str("vbnet")` must succeed.
- Create `qlty-analysis/src/lang/vbnet.rs` as a compileable skeleton module so the registry can reference it.
- Add one focused red test in `qlty-smells/src/metrics/metrics/lcom.rs` proving a VB.NET constructor named `new` is still treated as a constructor.
- Add one focused red test in `qlty-smells/src/metrics/metrics/lcom.rs` proving mixed-case `me.Value` and `Me.value` participate in the same LCOM group.
- Add one focused red test in `qlty-smells/src/metrics/metrics/cognitive.rs` proving a mixed-case self-recursive call still counts as recursion.
- Add one focused red test in `qlty-smells/src/metrics/metrics/fields.rs` proving mixed-case references to the same VB.NET field deduplicate to one field.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty-types --test language_enum`
Expected: FAIL because `"vbnet"` is not mapped yet.

Run: `cargo test -p qlty-analysis language_names`
Expected: FAIL because `from_str("vbnet")` does not resolve yet.

Run: `cargo test -p qlty-smells vbnet`
Expected: FAIL because the VB.NET skeleton and shared identifier comparisons are not implemented yet.

- [ ] **Step 3: Write minimal implementation**

Make the shared seams compile and compare correctly:

- Add a VB.NET grammar dependency that is compatible with the workspace's `tree-sitter` version. Prefer `arborium-vb`; if it is incompatible, choose a compatible binding without changing public behavior.
- Wire that dependency into `qlty-analysis/Cargo.toml`.
- Register `mod vbnet;`, export it, and add `Box::<vbnet::VBNet>::default()` to `ALL_LANGS`.
- Add a default `normalize_identifier()` hook to `Language` in `qlty-analysis/src/lang.rs`, then use it anywhere the shared path compares identifiers for constructors, self-references, recursion, or deduplicated field names.
- Add `[language.vbnet]` with `globs = ["*.vb"]` and an `Imports` duplication filter in `qlty-config/default.toml`.
- Map `"vbnet"` to `analysis::v1::Language::Vbdotnet` in `qlty-types/src/lib.rs`.
- Keep the VB.NET module skeletal here: enough to parse and override `normalize_identifier()`, but do not guess the real VB.NET query/node set yet.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty-types --test language_enum`
Expected: PASS

Run: `cargo test -p qlty-analysis language_names`
Expected: PASS

Run: `cargo test -p qlty-smells vbnet`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Keep this task scoped to registration and the generic normalization seam. Do not guess VB.NET node kinds yet beyond what is needed for the module to compile.

Run: `cargo check`
Run: `qlty fmt`
Run: `qlty check --level=low --fix`
Run: `cargo test`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock qlty-analysis/Cargo.toml qlty-analysis/src/lang.rs qlty-analysis/src/lang/vbnet.rs qlty-config/default.toml qlty-types/src/lib.rs qlty-types/tests/language_enum.rs qlty-smells/src/metrics/metrics/lcom.rs qlty-smells/src/metrics/metrics/cognitive.rs qlty-smells/src/metrics/metrics/fields.rs
git commit -m "feat: register vbnet seams and identifier normalization"
```

### Task 2: Implement The VB.NET Language Module With Direct Unit Coverage

**Files:**
- Modify: `qlty-analysis/src/lang/vbnet.rs`

- [ ] **Step 1: Identify or write the failing test**

Add focused unit tests in `qlty-analysis/src/lang/vbnet.rs` that lock down the VB.NET parser/query seams most likely to regress:

- A query capture test proving the class query finds a VB.NET type container and the function query finds `Sub`, `Function`, and constructor declarations.
- A constructor contract test proving `constructor_names()` includes `New`.
- A `call_identifiers` test for an implicit receiver call like `DoThing()`.
- A `call_identifiers` test for `Me.DoThing()`.
- A `field_identifiers` test for `Me.Value`.
- A `field_identifiers` or `call_identifiers` test for mixed-case source text so the extractor is proven against real VB.NET syntax rather than only canonical casing.
- A `mutually_exclusive()` bucket test like the existing language modules.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty-analysis vbnet`
Expected: FAIL because the skeleton module does not implement the real VB.NET queries and identifier logic yet.

- [ ] **Step 3: Write minimal implementation**

Implement the real VB.NET language module using the grammar's actual node kinds:

- Set `name()` to `"vbnet"` and `self_keyword()` to `Some("Me")`.
- Override `normalize_identifier()` so VB.NET identifier comparisons are case-insensitive.
- Define class/function/field queries from the real parse tree, not from guessed C# names.
- Make the function query include constructors and then override `constructor_names()` so `New` is treated as a constructor by the shared metrics code.
- Implement node buckets for the maintainability surfaces already used elsewhere: conditionals, `ElseIf`, loops, `Select Case`, jump/return nodes, boolean operators, comments, strings, fields, calls, functions, and closures only if the grammar exposes them.
- Implement `call_identifiers()` and `field_identifiers()` so `Me.Member`, `Me.Method()`, and implicit receiver calls inside a type are handled consistently enough for LCOM and cognitive metrics.
- Keep behavior inside the `Language` trait implementation; do not add VB.NET branches in `qlty-smells`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty-analysis vbnet`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Tighten the module before moving to CLI snapshots:

- Collapse duplicated constants/helpers.
- Re-check the parse tree before finalizing any node-kind string that is not covered by a test.
- Keep the module consistent with the existing language implementations rather than inventing a new pattern.

Run: `cargo check`
Run: `cargo test -p qlty-analysis`
Run: `qlty fmt`
Run: `qlty check --level=low --fix`
Run: `cargo test`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add qlty-analysis/src/lang/vbnet.rs
git commit -m "feat: implement vbnet language semantics"
```

### Task 3: Prove End-To-End VB.NET Maintainability Through The CLI And Update Docs

**Files:**
- Modify: `qlty-cli/tests/lang.rs`
- Create: `qlty-cli/tests/lang/vbnet/basic.toml`
- Create: `qlty-cli/tests/lang/vbnet/basic.stdout`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/.gitignore`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/.qlty/qlty.toml`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/BooleanLogic.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/FileComplexity.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/FunctionComplexity.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Identical.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Lines.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Members.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/NestedControl.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Parameters.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Returns.vb`
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.toml`
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.stdout`
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.in/.gitignore`
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.in/.qlty/qlty.toml`
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.in/example.vb`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.toml`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.stdout`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/.qlty/qlty.toml`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical.vb`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical2.vb`
- Modify: `README.md`
- Modify: `qlty-config/default.toml`
- Modify: `qlty-analysis/src/lang/vbnet.rs`

- [ ] **Step 1: Identify or write the failing test**

Add three focused CLI regressions and the corresponding documentation update:

- Add `fn vbnet_tests()` in `qlty-cli/tests/lang.rs` and build a git-backed VB.NET fixture repo that covers the same core maintainability surfaces as the existing C# suite: boolean logic, nested control flow, file complexity, function complexity, many parameters, many returns, duplicated executable code, line/code/comment stats, and one member/constructor case that proves mixed-case `Me` / member / `New` behavior still aggregates correctly.
- Add a focused `qlty metrics --all --json` fixture on a `.vb` file that must emit `LANGUAGE_VBDOTNET`.
- Add a focused `qlty smells --all --no-snippets --json` fixture where duplicated `Imports` lines are ignored but duplicated executable code still remains eligible.
- Update `README.md` expectations so VB.NET is listed wherever the built-in maintainability and metrics capabilities are described.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty --test integration vbnet_tests`
Expected: FAIL until the real CLI output matches the new VB.NET snapshot.

Run: `cargo test -p qlty --test integration metrics_tests`
Expected: FAIL because the new VB.NET metrics snapshot is not green yet.

Run: `cargo test -p qlty --test integration smells_tests`
Expected: FAIL because the VB.NET `Imports` filter case is not green yet.

- [ ] **Step 3: Write minimal implementation**

Fix product behavior until the end-to-end snapshots are faithful:

- Adjust VB.NET queries if file stats, function stats, or structure issues are missing.
- Adjust node buckets if boolean logic, returns, loops, or nested control are undercounted.
- Adjust `call_identifiers()` or `field_identifiers()` if the `Members.vb` stats are clearly wrong.
- Finalize the VB.NET duplication filter pattern in `qlty-config/default.toml` using the real grammar node for an `Imports` statement.
- Update `README.md` to add VB.NET to the built-in maintainability table and to the metrics-language list. Merge with current README contents rather than restoring an older table snapshot.
- Do not weaken the snapshots to accept `LANGUAGE_VB`, missing VB.NET behavior, or case-sensitive semantics that are invalid in VB.NET.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty --test integration vbnet_tests`
Expected: PASS

Run: `cargo test -p qlty --test integration metrics_tests`
Expected: PASS

Run: `cargo test -p qlty --test integration smells_tests`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

This is the completion gate for the whole feature. Before the final commit, run the repo-required checks and the shared regression surfaces:

Run: `cargo test -p qlty --test integration vbnet_tests`
Run: `cargo test -p qlty --test integration csharp_tests`
Run: `cargo test -p qlty --test integration swift_tests`

If `c_tests` and `cpp_tests` exist in `qlty-cli/tests/lang.rs` when this task is executed, run them here too because those changes overlap the same registration files.

Run: `cargo check`
Run: `qlty fmt`
Run: `qlty check --level=low --fix`
Run: `cargo test`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add qlty-cli/tests/lang.rs qlty-cli/tests/lang/vbnet qlty-cli/tests/cmd/metrics/vbnet_json.toml qlty-cli/tests/cmd/metrics/vbnet_json.stdout qlty-cli/tests/cmd/metrics/vbnet_json.in qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.toml qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.stdout qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in README.md qlty-config/default.toml qlty-analysis/src/lang/vbnet.rs
git commit -m "feat: add vbnet maintainability support"
```
