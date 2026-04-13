# VBNet Maintainability Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use trycycle-executing to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class VB.NET maintainability and metrics support so `*.vb` files are discovered, analyzed by the built-in smells/metrics pipeline, and emitted as `LANGUAGE_VBDOTNET` in all CLI outputs.

**Architecture:** Add a new built-in `vbnet` language implementation in `qlty-analysis` backed by a tree-sitter VB.NET grammar crate, then wire that single language name through the shared discovery/config/proto seams. Keep the steady-state shape identical to C# and Swift support: `qlty-analysis` owns grammar queries and node classification, `qlty-config` owns `.vb` discovery and duplication filtering, `qlty-types` owns the protocol enum mapping, and the acceptance proof lives primarily in real CLI snapshot tests.

**Tech Stack:** Rust workspace, tree-sitter, VB.NET grammar crate (`arborium-vb`), trycmd CLI snapshots, `cargo test`, `qlty fmt`, `qlty check`

---

## Desired End State

- `qlty build --no-plugins --print` analyzes `.vb` files and emits VB.NET structure and duplication issues plus file/function stats.
- `qlty smells --all --json` reports real VB.NET duplication/structure issues, but duplicated `Imports` statements are filtered out the same way C# `using` and Java `import` lines are today.
- `qlty metrics --all --json` emits `LANGUAGE_VBDOTNET` for VB.NET file and function stats.
- The implementation uses one internal language key, `vbnet`, end-to-end. It does not alias to `vb`, and it does not emit `LANGUAGE_VB`.
- The new language behaves like existing built-in maintainability languages: no plugin dependency, default thresholds unchanged, and no special-case plumbing in `qlty-smells`.

## Contracts And Invariants

- The only supported source extension for this feature is `*.vb`, and it maps to the internal language name `vbnet`.
- The emitted protocol enum must be `analysis::v1::Language::Vbdotnet` for all VB.NET issues and stats. Emitting `Vb` would be a product bug.
- `qlty-analysis::lang::from_str("vbnet")` must succeed everywhere that existing languages do; any path that still panics on `"vbnet"` is incomplete.
- `Imports` filtering belongs in config, not in custom duplication executor code. Keep VB.NET aligned with the existing language-specific `smells.duplication.filter_patterns` pattern.
- VB.NET should plug into the existing `Language` trait cleanly. Do not add language-specific branches in `qlty-smells` unless the trait truly cannot express the requirement.
- The C/C++ work in PR `#2746` touches the same shared registration seams (`Cargo.toml`, `qlty-analysis/src/lang.rs`, `qlty-config/default.toml`, `qlty-types/src/lib.rs`, `qlty-cli/tests/lang.rs`). Any implementation must merge with the current contents of those files rather than restoring older ordering or dropping in-flight language entries.

## Tricky Boundaries And Risks

- `.vb` is historically ambiguous, but this repo already distinguishes `LANGUAGE_VB` and `LANGUAGE_VBDOTNET` in the proto. This plan deliberately treats `.vb` as VB.NET and uses `vbnet` as the internal key to avoid muddying modern VB.NET support with legacy VB semantics.
- The highest-risk area is not the smell algorithms themselves; it is the shared registration seam. A missing enum mapping or missing `ALL_LANGS` entry will either panic or silently skip VB.NET files.
- The grammar node names are the second highest-risk area. Do not cargo-cult C# names. Inspect the actual parsed VB.NET tree before finalizing node constants and duplication filter query strings.
- LCOM and member-based metrics depend on `self_keyword`, `call_identifiers`, `field_identifiers`, and the class/function/field queries being coherent. Treat unqualified member calls and `Me.`-qualified accesses as first-class cases in the VB.NET module.
- Keep the first cut scoped to normal VB.NET constructs used in maintainability analysis: `Imports`, classes/modules/structures/interfaces, properties/fields, `If`/`ElseIf`/`Else`, `Select Case`, loops, lambdas, returns, and duplicated executable statements. XML literals, query comprehensions, and explicit line continuation can remain residual risk if the main pipeline is correct.

## File Structure

- Modify `Cargo.toml:129-140`
  Add the workspace-level VB.NET grammar dependency alongside the existing tree-sitter dependencies.
- Modify `Cargo.lock`
  Record the new grammar crate and its transitive dependencies.
- Modify `qlty-analysis/Cargo.toml:16-49`
  Consume the new workspace grammar dependency in the analysis crate.
- Create `qlty-analysis/src/lang/vbnet.rs`
  Implement the VB.NET `Language` trait, queries, node classifications, identifier extraction, and focused unit tests.
- Modify `qlty-analysis/src/lang.rs:6-48,219-229`
  Register the new module, re-export it, add it to `ALL_LANGS`, and extend the registry tests to include `vbnet`.
- Modify `qlty-config/default.toml:464-466`
  Add `[language.vbnet]` with `globs = ["*.vb"]` and the VB.NET duplication filter for `Imports`.
- Modify `qlty-types/src/lib.rs:302-318`
  Map `"vbnet"` to `analysis::v1::Language::Vbdotnet`.
- Create `qlty-types/tests/language_enum.rs`
  Lock down the `vbnet -> Vbdotnet` mapping with a direct test.
- Modify `qlty-cli/tests/lang.rs:53-60`
  Add `vbnet_tests()` so the new CLI snapshot suite runs like the other built-in languages.
- Create `qlty-cli/tests/lang/vbnet/basic.toml`
  Define the `qlty build --no-plugins --print` snapshot case.
- Create `qlty-cli/tests/lang/vbnet/basic.stdout`
  Snapshot the real end-to-end issues and stats, including `LANGUAGE_VBDOTNET`.
- Create `qlty-cli/tests/lang/vbnet/basic.in/.gitignore`
  Enable the git-backed fixture harness.
- Create `qlty-cli/tests/lang/vbnet/basic.in/.qlty/qlty.toml`
  Minimal config for the fixture repo.
- Create `qlty-cli/tests/lang/vbnet/basic.in/BooleanLogic.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/FileComplexity.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/FunctionComplexity.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Identical.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Lines.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/NestedControl.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Parameters.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Returns.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Members.vb`
- Create `qlty-cli/tests/lang/vbnet/basic.in/Closures.vb`
  Compact but representative VB.NET fixtures covering the full maintainability surface without copying the entire C# corpus.
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.toml`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.stdout`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.in/.gitignore`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.in/example.vb`
  Focused `qlty metrics --all --json` regression for the VB.NET language enum and basic stats.
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.toml`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.stdout`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical.vb`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical2.vb`
  Focused duplication regression proving `Imports` are filtered out.
- Modify `README.md:143-174,337`
  Advertise VB.NET maintainability support and add VB.NET to the metrics language list without undoing any concurrent C/C++ doc changes.

## Strategy Gate

The direct path is the right one here. VB.NET should be added as a normal built-in maintainability language, not as a plugin, alias, or partial “metrics-only” stopgap. The codebase already has a clean extension point for this in the `Language` trait plus config and enum registration, and the CLI snapshot harness is already the highest-fidelity way to prove the product behavior. The main implementation risk is shared-seam churn with the in-flight C/C++ work, so the executor should merge into the current versions of those files and let the real CLI tests, not assumptions, drive the final shape.

### Task 1: Build The VB.NET Language Core

**Files:**
- Modify: `Cargo.toml:129-140`
- Modify: `Cargo.lock`
- Modify: `qlty-analysis/Cargo.toml:16-49`
- Create: `qlty-analysis/src/lang/vbnet.rs`
- Modify: `qlty-analysis/src/lang.rs:6-48,219-229`
- Modify: `qlty-config/default.toml:464-466`
- Modify: `qlty-types/src/lib.rs:302-318`
- Create: `qlty-types/tests/language_enum.rs`

- [ ] **Step 1: Identify or write the failing test**

Add the smallest direct tests that lock down the shared seams before any CLI snapshot work:

- `qlty-types/tests/language_enum.rs` should assert `language_enum_from_name("vbnet") == analysis::v1::Language::Vbdotnet`.
- Extend the `language_names` test in `qlty-analysis/src/lang.rs` so `from_str("vbnet")` succeeds.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty-types --test language_enum`
Expected: FAIL because `language_enum_from_name("vbnet")` currently panics.

Run: `cargo test -p qlty-analysis language_names`
Expected: FAIL because `from_str("vbnet")` currently returns `None`.

- [ ] **Step 3: Write minimal implementation**

Implement the full language core cleanly in one pass:

- Add the VB.NET grammar crate to the workspace and `qlty-analysis`.
- Create `qlty-analysis/src/lang/vbnet.rs` and implement the `Language` trait using the actual node kinds from the grammar, not guessed C# names.
- Set the internal language name to `"vbnet"` and the self keyword to `Me`.
- Make the class query cover the VB.NET containers that should participate in metrics and LCOM: classes, structures, modules, and interfaces.
- Make the function query capture VB.NET callable declarations that users expect in metrics and structure analysis: `Function`, `Sub`, and constructors (`New`) where the grammar exposes them.
- Make the field query count fields and properties in the same spirit as C#.
- Implement `call_identifiers` and `field_identifiers` so `Me.Member`, `Me.Method()`, and simple unqualified member calls inside a type can contribute to LCOM instead of being dropped on the floor.
- Register the language in `qlty-analysis/src/lang.rs`.
- Add `[language.vbnet] globs = ["*.vb"]` in `qlty-config/default.toml`.
- Map `"vbnet"` to `analysis::v1::Language::Vbdotnet` in `qlty-types/src/lib.rs`.
- Add focused unit tests inside `vbnet.rs` for query capture sanity and mutually exclusive node bucket classification.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty-types --test language_enum`
Expected: PASS

Run: `cargo test -p qlty-analysis language_names`
Expected: PASS

Run: `cargo test -p qlty-analysis vbnet`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Tighten the new module before moving to the CLI layer. In particular:

- Collapse duplicated query helpers/constants inside `vbnet.rs`.
- Keep the implementation trait-driven; do not add VB.NET branches in `qlty-smells`.
- Re-check the grammar-derived node names used by the duplication filter and tests so they match the real parse tree.

Run: `cargo test -p qlty-analysis vbnet`
Run: `cargo check`
Run: `qlty fmt`
Run: `qlty check --level=low --fix`
Run: `cargo test`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock qlty-analysis/Cargo.toml qlty-analysis/src/lang.rs qlty-analysis/src/lang/vbnet.rs qlty-config/default.toml qlty-types/src/lib.rs qlty-types/tests/language_enum.rs
git commit -m "feat: add vbnet language core"
```

### Task 2: Prove End-To-End VB.NET Maintainability Through The Real CLI

**Files:**
- Modify: `qlty-cli/tests/lang.rs:53-60`
- Create: `qlty-cli/tests/lang/vbnet/basic.toml`
- Create: `qlty-cli/tests/lang/vbnet/basic.stdout`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/.gitignore`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/.qlty/qlty.toml`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/BooleanLogic.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/FileComplexity.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/FunctionComplexity.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Identical.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Lines.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/NestedControl.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Parameters.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Returns.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Members.vb`
- Create: `qlty-cli/tests/lang/vbnet/basic.in/Closures.vb`

- [ ] **Step 1: Identify or write the failing test**

Add `fn vbnet_tests()` in `qlty-cli/tests/lang.rs` and create a compact VB.NET fixture repo that exercises the user-visible behavior:

- Boolean logic thresholds via `AndAlso` / `OrElse`
- Nested control flow
- High file and function complexity
- Too many parameters
- Too many returns
- Duplicated executable code
- File/function/class/field/line stats
- A class or module that exercises field/property references for LCOM
- A lambda / closure example so closure nodes are not silently ignored

Author `basic.stdout` from the desired end state: the snapshot should contain VB.NET issues and stats labeled `LANGUAGE_VBDOTNET`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty --test integration vbnet_tests`
Expected: FAIL because the brand-new VB.NET snapshot will not match until the analysis output is correct.

- [ ] **Step 3: Write minimal implementation**

Fix the real product behavior until the CLI snapshot is correct. Typical remaining work here should be inside `qlty-analysis/src/lang/vbnet.rs` and `qlty-config/default.toml`, not the tests:

- Adjust query captures if classes/functions/properties are missing from stats.
- Adjust node buckets if boolean logic, returns, loops, or nested control are undercounted.
- Adjust `call_identifiers` / `field_identifiers` if LCOM or field counts are obviously wrong.
- Ensure the emitted issues and stats consistently use `LANGUAGE_VBDOTNET`.

Do not weaken the snapshot to accept missing behavior. Fix the implementation until the real CLI output is faithful.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty --test integration vbnet_tests`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Keep the fixture set representative but compact. Remove redundant files if they add no new coverage signal, but do not delete any scenario that catches a real VB.NET contract.

Run: `cargo test -p qlty --test integration vbnet_tests`
Run: `cargo test -p qlty --test integration csharp_tests`
Run: `cargo test -p qlty --test integration swift_tests`

If `c_tests` and `cpp_tests` exist in `qlty-cli/tests/lang.rs` by the time this task is executed, run them here too because PR `#2746` overlaps the same registration seam.

Run: `cargo check`
Run: `qlty fmt`
Run: `qlty check --level=low --fix`
Run: `cargo test`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add qlty-cli/tests/lang.rs qlty-cli/tests/lang/vbnet
git commit -m "feat: add vbnet maintainability snapshots"
```

### Task 3: Lock Down Command Surfaces And Documentation

**Files:**
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.toml`
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.stdout`
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.in/.gitignore`
- Create: `qlty-cli/tests/cmd/metrics/vbnet_json.in/example.vb`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.toml`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.stdout`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical.vb`
- Create: `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical2.vb`
- Modify: `README.md:143-174,337`

- [ ] **Step 1: Identify or write the failing test**

Add focused CLI regressions for the two easiest-to-miss product contracts:

- `qlty metrics --all --json` on a `.vb` file must emit `LANGUAGE_VBDOTNET`.
- `qlty smells --all --no-snippets --json` on files that only duplicate `Imports` lines must emit `[]`.

Keep these cases small and single-purpose so failures point directly at the broken contract.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty --test integration metrics_tests`
Expected: FAIL because the new VB.NET metrics snapshot is not green yet.

Run: `cargo test -p qlty --test integration smells_tests`
Expected: FAIL because the new VB.NET `Imports` filter case is not green yet.

- [ ] **Step 3: Write minimal implementation**

Implement only the behavior those command-surface tests reveal:

- Finalize the VB.NET duplication filter query string in `qlty-config/default.toml` using the real grammar node for an `Imports` statement.
- Correct any remaining stats/query issues that keep `metrics` from emitting `LANGUAGE_VBDOTNET`.
- Update `README.md` to add VB.NET maintainability support in the language table and include VB.NET in the metrics language list. Preserve any C/C++ wording that may have landed while the work was in progress; this doc edit must merge with current reality, not overwrite it.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty --test integration metrics_tests`
Expected: PASS

Run: `cargo test -p qlty --test integration smells_tests`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Re-run the focused VB.NET surfaces plus the repo gate:

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
git add qlty-cli/tests/cmd/metrics/vbnet_json.toml qlty-cli/tests/cmd/metrics/vbnet_json.stdout qlty-cli/tests/cmd/metrics/vbnet_json.in qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.toml qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.stdout qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in README.md
git commit -m "feat: finalize vbnet maintainability support"
```
