# VBNet Maintainability Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use trycycle-executing to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class VB.NET maintainability support so `*.vb` files are discovered by the built-in smells and metrics pipeline, emit `LANGUAGE_VBDOTNET`, and preserve VB.NET's case-insensitive member semantics.

**Architecture:** Add a normal built-in `vbnet` language implementation in `qlty-analysis`, register it through the existing language/config/type seams, then add one generic identifier-normalization path in shared metrics code so VB.NET stays case-insensitive without forking smells executors. Prove the feature with direct unit coverage for the parser contracts and shared normalization rules, then with real CLI snapshots for `build`, `metrics`, and `smells`.

**Tech Stack:** Rust workspace, tree-sitter, VB.NET grammar binding compatible with `tree-sitter = 0.22.6`, trycmd CLI snapshots, `cargo test`, `qlty fmt`, `qlty check`

---

## Desired End State

- `qlty build --no-plugins --print` analyzes `.vb` files and emits structure issues, duplication issues, and stats for VB.NET sources.
- `qlty metrics --all --json` emits file and function stats for `.vb` files with `language = "LANGUAGE_VBDOTNET"`.
- `qlty smells --all --no-snippets --json` reports real VB.NET duplication and structure issues, while duplicated `Imports` lines are filtered out like Java `import` and C# `using` lines.
- The internal language name is `vbnet` everywhere in this change. Do not add a `vb` alias and do not emit `LANGUAGE_VB`.
- VB.NET case-insensitive semantics are preserved anywhere shared code tracks identifiers: `Me` and `me` are equivalent, `New` and `new` are equivalent for constructor handling, and mixed-case member references do not fragment recursion, LCOM, or field counts.

## Contracts And Invariants

- The built-in maintainability path is the target. Do not add plugin-only or metrics-only stopgaps.
- `qlty-analysis::lang::from_str("vbnet")` must succeed anywhere the existing built-in languages do.
- `qlty_types::language_enum_from_name("vbnet")` must return `analysis::v1::Language::Vbdotnet`.
- `.vb` discovery must come from `[language.vbnet]` in `qlty-config/default.toml`.
- `Imports` filtering belongs in `qlty-config/default.toml`, not in a VB.NET-specific duplication executor branch.
- Keep `qlty-smells` generic. If VB.NET needs case-insensitive identifier handling, express it through `Language` trait helpers plus shared metrics code.
- Do not stop at `call_identifiers()` and `field_identifiers()`. The implementation must also cover the shared contracts used by the current smells pipeline: `function_name_node()`, `get_parameter_names()`, constructor detection, self-keyword comparisons, and any identifier set membership used by LCOM, cognitive complexity, or field counting.
- Prefer a compatible external grammar crate. If no published binding works with the workspace `tree-sitter = 0.22.6`, create a thin local binding crate instead of changing the public `vbnet` / `LANGUAGE_VBDOTNET` contract.

## File Structure

- Modify `Cargo.toml`
  Add the chosen compatible VB.NET grammar dependency at the workspace level. If no compatible crate exists, add a local workspace member for a thin binding crate instead.
- Modify `Cargo.lock`
  Record the new dependency graph.
- Modify `qlty-analysis/Cargo.toml`
  Consume the VB.NET grammar dependency in `qlty-analysis`.
- Modify `qlty-analysis/src/lang.rs`
  Register `vbnet`, export the module, and add generic identifier helper methods for case-insensitive languages.
- Create `qlty-analysis/src/lang/vbnet.rs`
  Implement the VB.NET `Language` trait: grammar wiring, queries, node buckets, constructor handling, call/field extraction, identifier normalization, parameter extraction, function-name extraction, and focused unit tests.
- Create `vendor/tree-sitter-vbnet/Cargo.toml`
- Create `vendor/tree-sitter-vbnet/build.rs`
- Create `vendor/tree-sitter-vbnet/bindings/rust/lib.rs`
  Create these only if no published crate is compatible with the workspace `tree-sitter` version.
- Modify `qlty-config/default.toml`
  Add `[language.vbnet]` with `globs = ["*.vb"]` and an `Imports` duplication filter that matches the real VB.NET grammar node.
- Modify `qlty-types/src/lib.rs`
  Map `vbnet` to `analysis::v1::Language::Vbdotnet`.
- Create `qlty-types/tests/language_enum.rs`
  Lock down the `vbnet -> Vbdotnet` mapping directly.
- Modify `qlty-smells/src/metrics/metrics/lcom.rs`
  Normalize identifiers before constructor checks, self-receiver checks, and group/set insertion so mixed-case member references still connect the same methods and fields.
- Modify `qlty-smells/src/metrics/metrics/cognitive.rs`
  Normalize function-name tracking and recursive-call comparisons so mixed-case self-recursion still counts.
- Modify `qlty-smells/src/metrics/metrics/fields.rs`
  Normalize deduplicated field names before inserting them into the set.
- Modify `qlty-cli/tests/lang.rs`
  Add `vbnet_tests()` so the existing build snapshot harness executes the new suite.
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
  Keep this fixture repo close to the existing C# maintainability fixture, but make `Members.vb` prove mixed-case `Me` / member / `New` behavior through real emitted stats and issues.
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.toml`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.stdout`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.in/.gitignore`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.in/.qlty/qlty.toml`
- Create `qlty-cli/tests/cmd/metrics/vbnet_json.in/example.vb`
  Add a focused metrics regression that proves the user-facing `metrics` command emits `LANGUAGE_VBDOTNET`.
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.toml`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.stdout`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/.qlty/qlty.toml`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical.vb`
- Create `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/Identical2.vb`
  Add a focused smells regression that proves duplicated `Imports` lines are filtered while duplicated executable code still remains eligible.
- Modify `README.md`
  Add VB.NET where the built-in maintainability and metrics-language capabilities are described, preserving any concurrent language additions already present in the file.

## Strategy Gate

The feature direction is correct, but the earlier task split was wrong. A "skeletal" VB.NET module is not enough to support the shared smells pipeline because the current code depends on more than parser registration: it also depends on `function_name_node()`, `get_parameter_names()`, constructor recognition, and normalized identifier storage inside LCOM/cognitive/field metrics. The direct path is still the right one, but execution must land the real parser/query contract before shared-case-normalization tests, and then normalize identifiers at the points where names are stored as well as compared.

### Task 1: Register VB.NET And Implement The Real Language Contract

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `qlty-analysis/Cargo.toml`
- Modify: `qlty-analysis/src/lang.rs`
- Create: `qlty-analysis/src/lang/vbnet.rs`
- Modify: `qlty-config/default.toml`
- Modify: `qlty-types/src/lib.rs`
- Create: `qlty-types/tests/language_enum.rs`
- Create: `vendor/tree-sitter-vbnet/Cargo.toml`
- Create: `vendor/tree-sitter-vbnet/build.rs`
- Create: `vendor/tree-sitter-vbnet/bindings/rust/lib.rs`

- [ ] **Step 1: Identify or write the failing test**

Write the smallest red tests that prove the registration and parser contract the rest of the pipeline needs:

- Create `qlty-types/tests/language_enum.rs` asserting `language_enum_from_name("vbnet") == analysis::v1::Language::Vbdotnet`.
- Extend `qlty-analysis/src/lang.rs` `language_names()` so `from_str("vbnet")` must resolve.
- Add focused `qlty-analysis/src/lang/vbnet.rs` unit tests for:
  - class query capture on a simple `Class ... End Class`
  - function query capture on `Sub`, `Function`, and `Sub New`
  - `constructor_names()` recognizing `New`
  - `function_name_node()` returning the actual VB.NET member name node
  - `get_parameter_names()` extracting parameter names from real VB.NET parameter syntax
  - `call_identifiers()` for implicit receiver calls and `Me.Member()`
  - `field_identifiers()` for `Me.Member`
  - `mutually_exclusive()` for the node-bucket lists

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty-types --test language_enum`
Expected: FAIL because `"vbnet"` is not mapped yet.

Run: `cargo test -p qlty-analysis language_names`
Expected: FAIL because `from_str("vbnet")` does not resolve yet.

Run: `cargo test -p qlty-analysis vbnet`
Expected: FAIL because the VB.NET language module and grammar wiring do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Implement the real VB.NET language contract, not a placeholder:

- Add a VB.NET grammar source that works with `tree-sitter = 0.22.6`. Prefer a published crate. If none fits, create the thin local binding crate under `vendor/tree-sitter-vbnet/` and wire it into the workspace instead of inventing a new public language name.
- Wire the grammar dependency into `qlty-analysis/Cargo.toml`.
- Register `mod vbnet;`, export it, and add `Box::<vbnet::VBNet>::default()` to `ALL_LANGS`.
- Add `[language.vbnet]` with `globs = ["*.vb"]` and the real `Imports` duplication filter node in `qlty-config/default.toml`.
- Map `"vbnet"` to `analysis::v1::Language::Vbdotnet` in `qlty-types/src/lib.rs`.
- Implement `qlty-analysis/src/lang/vbnet.rs` with the grammar's real node names:
  - `name() == "vbnet"`
  - `self_keyword() == Some("Me")`
  - class/function/field queries with captures `@definition.class`, `@definition.function`, `@field`, `@name`, and `@parameters`
  - node buckets for conditionals, `ElseIf`, `Select Case`, loops, jumps, returns, boolean/binary expressions, fields, calls, functions, comments, strings, and closures only if the grammar exposes them
  - `constructor_names()` including `New`
  - `function_name_node()` if the grammar does not use the default `"name"` field
  - `get_parameter_names()` if VB.NET parameter nodes are not simple direct named children
  - `call_identifiers()` and `field_identifiers()` that handle implicit receiver calls, `Me.Member`, and nested member access correctly enough for metrics

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty-types --test language_enum`
Expected: PASS

Run: `cargo test -p qlty-analysis language_names`
Expected: PASS

Run: `cargo test -p qlty-analysis vbnet`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Do not move on until the parser/query contract is clean and stable:

- Remove duplicated query helpers or node-name constants.
- Confirm any non-obvious node-kind string against the actual grammar before leaving it in the module.
- Keep the implementation aligned with the existing language-module style.

Run: `cargo check`
Run: `cargo test -p qlty-analysis`
Run: `cargo test -p qlty-types`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock qlty-analysis/Cargo.toml qlty-analysis/src/lang.rs qlty-analysis/src/lang/vbnet.rs qlty-config/default.toml qlty-types/src/lib.rs qlty-types/tests/language_enum.rs vendor/tree-sitter-vbnet
git commit -m "feat: register vbnet language support"
```

If no local binding crate was needed, omit `vendor/tree-sitter-vbnet` from `git add`.

### Task 2: Add Generic Case-Insensitive Identifier Handling For VB.NET Metrics

**Files:**
- Modify: `qlty-analysis/src/lang.rs`
- Modify: `qlty-analysis/src/lang/vbnet.rs`
- Modify: `qlty-smells/src/metrics/metrics/lcom.rs`
- Modify: `qlty-smells/src/metrics/metrics/cognitive.rs`
- Modify: `qlty-smells/src/metrics/metrics/fields.rs`

- [ ] **Step 1: Identify or write the failing test**

Add focused red tests around the shared metrics seams that currently assume case-sensitive identifiers:

- Add a red test in `qlty-smells/src/metrics/metrics/lcom.rs` proving a VB.NET `Sub New` or mixed-case `sub new` is still treated as a constructor.
- Add a red test in `qlty-smells/src/metrics/metrics/lcom.rs` proving a method group still connects when one method calls `Me.dothing()` and the declaration is `DoThing`.
- Add a red test in `qlty-smells/src/metrics/metrics/lcom.rs` proving `Me.Value` and `me.value` end up in the same field group.
- Add a red test in `qlty-smells/src/metrics/metrics/cognitive.rs` proving mixed-case self-recursion still counts.
- Add a red test in `qlty-smells/src/metrics/metrics/fields.rs` proving deduplicated field names normalize before insertion into the set.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p qlty-smells vbnet`
Expected: FAIL because shared metrics code still stores and compares identifiers case-sensitively.

- [ ] **Step 3: Write minimal implementation**

Make identifier handling generic and normalize names before they are stored as well as when they are compared:

- Add a default `normalize_identifier()` helper to `Language` in `qlty-analysis/src/lang.rs`.
- Add a small helper in `Language` for equality against language keywords or receivers if that keeps call sites simple.
- Override `normalize_identifier()` in `qlty-analysis/src/lang/vbnet.rs` so VB.NET identifiers normalize case-insensitively.
- Update `sanitize_parameter_name()` to use the helper instead of raw string equality.
- Update `qlty-smells/src/metrics/metrics/lcom.rs` so constructor checks, self-receiver checks, method-name insertion, field-name insertion, and group intersection all use normalized names.
- Update `qlty-smells/src/metrics/metrics/cognitive.rs` so tracked function names and recursive-call comparisons use normalized names.
- Update `qlty-smells/src/metrics/metrics/fields.rs` so deduplicated field names are normalized before insertion.
- Keep the implementation generic. Do not add a VB.NET branch inside the metrics executors.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p qlty-smells vbnet`
Expected: PASS

- [ ] **Step 5: Refactor and verify**

Keep the shared seams minimal and make sure existing case-sensitive languages still behave the same:

Run: `cargo check`
Run: `cargo test -p qlty-smells`
Run: `cargo test -p qlty-analysis`
Run: `cargo test -p qlty-types`
Expected: all PASS

- [ ] **Step 6: Commit**

```bash
git add qlty-analysis/src/lang.rs qlty-analysis/src/lang/vbnet.rs qlty-smells/src/metrics/metrics/lcom.rs qlty-smells/src/metrics/metrics/cognitive.rs qlty-smells/src/metrics/metrics/fields.rs
git commit -m "feat: normalize vbnet identifiers in shared metrics"
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

Add the real product-surface regressions:

- Add `fn vbnet_tests()` in `qlty-cli/tests/lang.rs` and a git-backed VB.NET fixture repo that covers the same core maintainability surfaces as the current C# suite: boolean logic, nested control flow, file complexity, function complexity, many parameters, many returns, duplicated executable code, line/code/comment stats, and one member/constructor case proving mixed-case `Me` / member / `New` behavior.
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
- Adjust node buckets if boolean logic, returns, loops, `ElseIf`, or nested control are undercounted.
- Adjust `function_name_node()` or `get_parameter_names()` if the CLI surfaces still emit wrong names or parameter counts.
- Adjust `call_identifiers()` or `field_identifiers()` if `Members.vb` shows broken mixed-case aggregation.
- Finalize the VB.NET duplication filter pattern in `qlty-config/default.toml` using the real grammar node for an `Imports` statement.
- Update `README.md` to add VB.NET to the built-in maintainability table and the metrics-language list. Merge with current README contents rather than restoring an older snapshot.
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
