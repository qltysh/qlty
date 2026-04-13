# VBNet Maintainability Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use trycycle-executing to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class VB.NET maintainability support so `*.vb` files are discovered by the built-in smells and metrics pipeline and all emitted issues/stats use `LANGUAGE_VBDOTNET`.

**Architecture:** Add a normal built-in `vbnet` language implementation in `qlty-analysis`, wire that one language key through config and proto mapping, and prove behavior primarily through real CLI snapshot tests. Use a tree-sitter VB.NET grammar that is compatible with this workspace's `tree-sitter` version; keep the public product contract (`vbnet` in config, `LANGUAGE_VBDOTNET` in output) fixed even if the dependency choice changes.

**Tech Stack:** Rust workspace, tree-sitter, VB.NET grammar crate or binding, trycmd CLI snapshots, `cargo test`, `qlty fmt`, `qlty check`

---

## Desired End State

- `qlty build --no-plugins --print` analyzes `.vb` files and emits structure issues, duplication issues, and stats for VB.NET sources.
- `qlty metrics --all --json` emits file and function stats for `.vb` files with `language = "LANGUAGE_VBDOTNET"`.
- `qlty smells --all --no-snippets --json` reports real VB.NET duplication/structure issues, but duplicated `Imports` lines are filtered out the same way Java `import` and C# `using` lines are today.
- The internal language name is `vbnet` everywhere in this change. Do not alias to `vb`, and do not emit `LANGUAGE_VB`.
- Constructors named `New` are treated as constructors for language semantics, so they are not accidentally counted as ordinary instance methods in LCOM-related logic.

## Contracts And Invariants

- The built-in maintainability path is the target here. Do not add plugin-only or metrics-only stopgaps.
- `qlty-analysis::lang::from_str("vbnet")` must succeed anywhere the existing built-in languages do.
- `qlty_types::language_enum_from_name("vbnet")` must return `analysis::v1::Language::Vbdotnet`.
- `Imports` filtering belongs in `qlty-config/default.toml`, not in custom duplication executor branches.
- Keep `qlty-smells` generic. If the `Language` trait can express the requirement, use the trait rather than adding VB.NET-specific executor code.
- Do not add `[file_types.vbnet]` in this task unless an actual consumer in this change needs it. The user asked for built-in maintainability/language support, and `config.language` already drives Qlty file discovery for this pipeline.
- Prefer `arborium-vb` if it is compatible with the workspace's `tree-sitter = 0.22.6`. If it is not, choose a compatible VB.NET tree-sitter binding or vendor a thin local binding without changing the public `vbnet`/`LANGUAGE_VBDOTNET` contract.

## File Structure

- Modify `Cargo.toml`
  Add the chosen compatible VB.NET grammar dependency at the workspace level.
- Modify `Cargo.lock`
  Record the new grammar dependency and its transitive dependencies.
- Modify `qlty-analysis/Cargo.toml`
  Consume the new grammar dependency in the analysis crate.
- Create `qlty-analysis/src/lang/vbnet.rs`
  Implement the VB.NET `Language` trait, query definitions, identifier extraction, constructor handling, and focused unit tests.
- Modify `qlty-analysis/src/lang.rs`
  Register `vbnet`, export the module, add it to `ALL_LANGS`, and extend the registry tests.
- Modify `qlty-config/default.toml`
  Add `[language.vbnet]` with `globs = ["*.vb"]` and a duplication filter for `Imports`.
- Modify `qlty-types/src/lib.rs`
  Map `vbnet` to `analysis::v1::Language::Vbdotnet`.
- Create `qlty-types/tests/language_enum.rs`
  Lock down the `vbnet -> Vbdotnet` mapping directly.
- Modify `qlty-cli/tests/lang.rs`
  Add `vbnet_tests()` so the new language suite runs with the existing integration harness.
- Create `qlty-cli/tests/lang/vbnet/basic.toml`
- Create `qlty-cli/tests/lang/vbnet/basic.stdout`
- Create `qlty-cli/tests/lang/vbnet/basic.in/.gitignore`
- Create `qlty-cli/tests/lang/vbnet/basic.in/BooleanLogic.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/FileComplexity.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/FunctionComplexity.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Identical.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Lines.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Members.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/NestedControl.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Parameters.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Returns.vb`
  Keep this fixture repo close to the existing C# maintainability fixture, with one extra file that exercises `Me` and member access so LCOM and field extraction are not accidental.
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
  Add VB.NET to the built-in maintainability language table and to the metrics language list without dropping any concurrent language additions.

## Strategy Gate

The plan should stay on the direct path: built-in VB.NET support, not a compatibility alias and not a partial implementation. The main implementation risks are shared registration seams and VB-specific language semantics that existing languages do not have, especially constructor naming (`New`) and member access via `Me` or implicit receiver calls. The safest execution pattern is to lock down the registry/enum seams first, then implement the VB.NET language module with focused unit coverage, then prove the whole product path through real CLI snapshots.

### Task 1: Register VB.NET In The Shared Seams

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `qlty-analysis/Cargo.toml`
- Modify: `qlty-analysis/src/lang.rs`
- Modify: `qlty-config/default.toml`
- Modify: `qlty-types/src/lib.rs`
- Create: `qlty-types/tests/language_enum.rs`
- Create: `qlty-analysis/src/lang/vbnet.rs`

- [ ] **Step 1: Identify or write the failing test**

Add the smallest seam tests before doing any behavior work:

- Create `qlty-types/tests/language_enum.rs` with a direct assertion that `language_enum_from_name("vbnet")` returns `analysis::v1::Language::Vbdotnet`.
- Extend `qlty-analysis/src/lang.rs` `language_names()` so `from_str("vbnet")` must succeed.
- Create `qlty-analysis/src/lang/vbnet.rs` as a compileable skeleton module so the registry can reference it, even before the full query logic is finished.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty-types --test language_enum`
Expected: FAIL because `"vbnet"` is not mapped yet.

Run: `cargo test -p qlty-analysis language_names`
Expected: FAIL because `from_str("vbnet")` does not resolve yet.

- [ ] **Step 3: Write minimal implementation**

Make the shared seams compile and resolve cleanly:

- Add a VB.NET grammar dependency that is compatible with the workspace's `tree-sitter` version. Prefer `arborium-vb`; if it is incompatible, choose a compatible binding without changing public behavior.
- Wire that dependency into `qlty-analysis/Cargo.toml`.
- Register `mod vbnet;`, export it, and add `Box::<vbnet::VBNet>::default()` to `ALL_LANGS`.
- Add `[language.vbnet]` with `globs = ["*.vb"]` in `qlty-config/default.toml`.
- Map `"vbnet"` to `analysis::v1::Language::Vbdotnet` in `qlty-types/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty-types --test language_enum`
Expected: PASS

Run: `cargo test -p qlty-analysis language_names`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Keep this task scoped to registration only. Do not guess VB.NET node kinds yet beyond what is needed for the module to compile.

Run: `cargo check`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock qlty-analysis/Cargo.toml qlty-analysis/src/lang.rs qlty-analysis/src/lang/vbnet.rs qlty-config/default.toml qlty-types/src/lib.rs qlty-types/tests/language_enum.rs
git commit -m "feat: register vbnet language support"
```

### Task 2: Implement VB.NET Language Semantics With Direct Unit Coverage

**Files:**
- Modify: `qlty-analysis/src/lang/vbnet.rs`

- [ ] **Step 1: Identify or write the failing test**

Add focused unit tests in `qlty-analysis/src/lang/vbnet.rs` that lock down the language-specific seams most likely to regress:

- A query capture test proving the class query finds a VB.NET type container and the function query finds `Sub`, `Function`, and constructor declarations.
- A constructor contract test proving `constructor_names()` includes `New`.
- A `call_identifiers` test for a simple implicit receiver call like `DoThing()`.
- A `call_identifiers` test for `Me.DoThing()`.
- A `field_identifiers` test for `Me.Value`.
- A `mutually_exclusive()` bucket test like the existing language modules.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty-analysis vbnet`
Expected: FAIL because the skeleton module does not implement the real VB.NET queries and identifier logic yet.

- [ ] **Step 3: Write minimal implementation**

Implement the real VB.NET language module using the grammar's actual node kinds:

- Set `name()` to `"vbnet"` and `self_keyword()` to `Some("Me")`.
- Define class/function/field queries from the real parse tree, not from guessed C# names.
- Make the function query include constructors and then override `constructor_names()` so `New` is treated as a constructor by the metrics code.
- Implement node buckets for the maintainability surfaces already used elsewhere: conditionals, `ElseIf`, loops, `Select Case`, jump/return nodes, boolean operators, comments, strings, fields, calls, functions, and closures only if the grammar exposes them.
- Implement `call_identifiers()` and `field_identifiers()` so `Me.Member`, `Me.Method()`, and implicit receiver calls inside a type are handled consistently enough for LCOM and cognitive metrics.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty-analysis vbnet`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Tighten the module before moving to CLI snapshots:

- Collapse duplicated constants/helpers.
- Re-check the parse tree before finalizing any node-kind string that is not covered by a test.
- Keep all behavior inside the `Language` trait implementation; do not add VB.NET branches in `qlty-smells`.

Run: `cargo test -p qlty-analysis`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add qlty-analysis/src/lang/vbnet.rs
git commit -m "feat: implement vbnet language semantics"
```

### Task 3: Prove End-To-End VB.NET Maintainability Through `qlty build`

**Files:**
- Modify: `qlty-cli/tests/lang.rs`
- Create: `qlty-cli/tests/lang/vbnet/basic.toml`
- Create: `qlty-cli/tests/lang/vbnet/basic.stdout`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/.gitignore`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/BooleanLogic.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/FileComplexity.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/FunctionComplexity.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Identical.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Lines.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Members.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/NestedControl.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Parameters.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Returns.vb`

- [ ] **Step 1: Identify or write the failing test**

Add `fn vbnet_tests()` in `qlty-cli/tests/lang.rs` and build a git-backed VB.NET fixture repo that covers the same core maintainability surfaces as the existing C# suite:

- boolean logic
- nested control flow
- file complexity
- function complexity
- many parameters
- many returns
- duplicated executable code
- line/code/comment stats
- one member-access case that exercises `Me` or implicit receiver behavior

Write the expected snapshot so the output must contain VB.NET issues and stats labeled `LANGUAGE_VBDOTNET`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty --test integration vbnet_tests`
Expected: FAIL until the real CLI output matches the new VB.NET snapshot.

- [ ] **Step 3: Write minimal implementation**

Fix product behavior until the end-to-end snapshot is faithful:

- Adjust VB.NET queries if file stats, function stats, or structure issues are missing.
- Adjust node buckets if boolean logic, returns, loops, or nested control are undercounted.
- Adjust `call_identifiers()` or `field_identifiers()` if the `Members.vb` stats are clearly wrong.
- Do not weaken the snapshot to accept `LANGUAGE_VB` or missing VB.NET behavior.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty --test integration vbnet_tests`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Keep the fixture set compact, but do not remove any file that is carrying unique coverage signal.

Run: `cargo test -p qlty --test integration vbnet_tests`
Run: `cargo test -p qlty --test integration csharp_tests`
Run: `cargo test -p qlty --test integration swift_tests`

If `c_tests` and `cpp_tests` exist in `qlty-cli/tests/lang.rs` when this task is executed, run them here too because those changes overlap the same registration files.

Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add qlty-cli/tests/lang.rs qlty-cli/tests/lang/vbnet
git commit -m "feat: add vbnet build snapshots"
```

### Task 4: Lock Down `metrics`, `smells`, And Documentation

**Files:**
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

Add two focused command-surface regressions:

- `qlty metrics --all --json` on a `.vb` file must emit `LANGUAGE_VBDOTNET`.
- `qlty smells --all --no-snippets --json` on files that only duplicate `Imports` lines must emit `[]`.

Keep these fixtures single-purpose so failures point directly at the broken contract.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty --test integration metrics_tests`
Expected: FAIL because the new VB.NET metrics snapshot is not green yet.

Run: `cargo test -p qlty --test integration smells_tests`
Expected: FAIL because the VB.NET `Imports` filter case is not green yet.

- [ ] **Step 3: Write minimal implementation**

Implement only the remaining behavior these command-surface tests reveal:

- Finalize the VB.NET duplication filter pattern in `qlty-config/default.toml` using the real grammar node for an `Imports` statement.
- Correct any remaining stats/query issues that keep `metrics` from emitting `LANGUAGE_VBDOTNET`.
- Update `README.md` to add VB.NET to the built-in maintainability table and to the metrics-language list. Merge with current README contents rather than restoring an older table snapshot.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty --test integration metrics_tests`
Expected: PASS

Run: `cargo test -p qlty --test integration smells_tests`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

This is the completion gate for the whole feature. Before the final commit, run the repo-required checks:

Run: `cargo test -p qlty --test integration vbnet_tests`
Run: `cargo test -p qlty --test integration metrics_tests`
Run: `cargo test -p qlty --test integration smells_tests`
Run: `cargo check`
Run: `qlty fmt`
Run: `qlty check --level=low --fix`
Run: `cargo test`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add qlty-cli/tests/cmd/metrics/vbnet_json.toml qlty-cli/tests/cmd/metrics/vbnet_json.stdout qlty-cli/tests/cmd/metrics/vbnet_json.in qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.toml qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.stdout qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in README.md qlty-config/default.toml qlty-analysis/src/lang/vbnet.rs
git commit -m "feat: finalize vbnet maintainability support"
```
