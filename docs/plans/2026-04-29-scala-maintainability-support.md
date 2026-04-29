# Scala Maintainability Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use trycycle-executing to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class Scala maintainability support so `*.scala` (and `*.sc`) files are discovered by Qlty's built-in smells and metrics pipeline, parsed by tree-sitter, and emit `LANGUAGE_SCALA` — at parity with Ruby, Java, Kotlin, Rust, and the in-flight VB.NET work.

**Architecture:** Add a normal built-in `scala` language implementation in `qlty-analysis` backed by the published `tree-sitter-scala` crate, register it through the existing language/config/type seams (`ALL_LANGS`, `default.toml`, `language_enum_from_name`), and add Scala-flavored test mods plus a CLI fixture suite that exercises every metric and structure smell. Scala is case-sensitive, JVM-idiomatic, and expression-oriented — so the closest implementation analog is `qlty-analysis/src/lang/kotlin.rs`. No changes to shared metrics executors are required (the case-insensitive normalization seam landed for VB.NET works for Scala as a no-op). The cutover is single-shot; there is no interim phase.

**Tech Stack:** Rust workspace, tree-sitter `0.22.6`, `tree-sitter-scala = 0.22.1` (compatible with workspace tree-sitter; published on crates.io), trycmd CLI snapshots, `cargo test`, `qlty fmt`, `qlty check`.

---

## Desired End State

- `qlty build --no-plugins --print` analyzes `.scala` and `.sc` files and emits structure issues, duplication issues, and stats for Scala sources.
- `qlty metrics --all --json` emits file and function stats for `.scala`/`.sc` files with `language = "LANGUAGE_SCALA"`.
- `qlty smells --all --no-snippets --json` reports real Scala duplication and structure issues, with duplicated `import` statements filtered out (mirroring Java/Kotlin/Rust).
- The internal language name is `scala` everywhere. Globs are `*.scala` and `*.sc` (Scala scripts and worksheets share grammar; gating them behind a separate name would diverge from how every other language registers a single canonical name).
- `from_str("scala")` resolves; `language_enum_from_name("scala") == analysis::v1::Language::Scala`; `LANGUAGE_SCALA` already exists in the proto (`qlty-types/src/protos/qlty.analysis.v1.rs:771`), so no proto regeneration is required.
- Per-language unit test mods (`mod scala`) exist in every smells/metrics file that already has a `mod java`/`mod kotlin`/`mod ruby` block, plus the additional smells/metrics modules called out in the testing strategy. Each mod exercises one Scala-specific scenario (match-with-guards, for-comprehension, expression-bodied def, case class) on top of mirroring the existing per-language scenarios.
- A malformed Scala snippet through `File::from_string("scala", ...)` plus a metric counter does not panic.

## Contracts And Invariants

- The built-in maintainability path is the target. No external linter plugin (no scalafix, no scalastyle, no metals).
- `qlty-analysis::lang::from_str("scala")` must succeed anywhere existing built-in languages do.
- `qlty_types::language_enum_from_name("scala")` must return `analysis::v1::Language::Scala`.
- `.scala` and `.sc` discovery must come from `[language.scala]` in `qlty-config/default.toml`.
- Scala `import` statement filtering belongs in `qlty-config/default.toml` via `smells.duplication.filter_patterns`, not in any Scala-specific executor branch. Use the real `tree-sitter-scala` node kind (`import_declaration`).
- Keep `qlty-smells` generic. Scala is case-sensitive and reuses the case-sensitive defaults of the `Language` trait — do not override `normalize_identifier` for Scala.
- Do not stop at `call_identifiers()` and `field_identifiers()`. The implementation must also cover the shared contracts the smells pipeline depends on: `function_name_node()`, `get_parameter_names()`, `constructor_names()`, `self_keyword()`, and the node-bucket lists used by cyclomatic, cognitive, LCOM, fields, function complexity, file complexity, parameters, returns, and boolean logic. Verify each against the actual `tree-sitter-scala` grammar before leaving it in the module.
- Prefer the published external grammar crate. `tree-sitter-scala = 0.22.1` declares `tree-sitter ^0.22.6` (verified via crates.io API) and is therefore compatible with the workspace tree-sitter pin. Newer 0.23.x/0.24.x/0.25.x/0.26.x releases require tree-sitter `^0.24` or `^0.25` and are incompatible with this workspace until tree-sitter itself is bumped — that bump is out of scope for this change. Pin to `0.22.1`.
- The plan lands the requested end state in a single cutover. There is no interim "skeleton then integration" phase.
- Conventional Commits for every commit. Auto-format and lint before each commit. Never commit to `main`. Open the PR in draft mode with a Conventional Commits title.

## Strategy Gate

The direct path is correct. Three framing decisions worth pinning down explicitly so a reviewer doesn't second-guess them:

1. **Single language name `scala` covering both `*.scala` and `*.sc`.** The user explicitly asked for parity with Ruby/Rust/Java/Kotlin — none of which split into multiple internal language names by extension. `tree-sitter-scala` parses both `.scala` source files and `.sc` script/worksheet files using the same grammar (script files are a strict superset that allow top-level definitions, which the grammar already accepts). Splitting `*.sc` into a second language name would create cosmetic divergence with no analytical benefit and would also collide with Scala's own ecosystem norms (sbt, mill, IntelliJ, and Metals all treat `.sc` as Scala). **Decision:** include both globs under `[language.scala]`. If a real-world `.sc` parsing failure surfaces during fixture construction, downgrade to `*.scala`-only and record the residual risk — but start with both.

2. **Use the published `tree-sitter-scala 0.22.1` crate, not a vendored fork.** The VB.NET branch ended up vendoring its grammar because no compatible published crate existed. Scala does not have that problem — `tree-sitter-scala 0.22.1` is on crates.io with a `tree-sitter ^0.22.6` dependency that exactly matches the workspace pin. Vendoring would add maintenance burden for no benefit. The grammar is mature (Scala 2 + partial Scala 3 support).

3. **Scala 3 (Dotty) syntax coverage is best-effort, not a blocker.** `tree-sitter-scala 0.22.1` parses most Scala 2 syntax correctly and a meaningful subset of Scala 3 (significant indentation, `given`/`using`, enums). Where Scala 3-only syntax fails to parse, the file will produce zero metrics rather than panic — that's the same behavior every other language has on malformed input, and it's the right behavior. Document this as a residual risk in the plan but do not gate the cutover on it. Add a small Scala 3 file to the fixture suite to surface the current ceiling.

4. **No changes to shared metrics executors.** VB.NET's `normalize_identifier` seam is sufficient for Scala because Scala is case-sensitive — the default no-op normalization is correct. Do not override the helper for Scala. This keeps the change purely additive.

Architectural direction is stable. Proceed to file structure and task decomposition.

## File Structure

- Modify `Cargo.toml`
  Add `tree-sitter-scala = "0.22.1"` to the `[workspace.dependencies]` block alongside the other tree-sitter language crates.
- Modify `Cargo.lock`
  Record the new dependency graph (regenerated automatically by `cargo build`).
- Modify `qlty-analysis/Cargo.toml`
  Consume the workspace `tree-sitter-scala` dependency.
- Modify `qlty-analysis/src/lang.rs`
  Register `mod scala`, re-export from the `pub use` block, and append `Box::<scala::Scala>::default()` to `ALL_LANGS` (placed alphabetically next to `ruby` and `rust` for readability).
- Create `qlty-analysis/src/lang/scala.rs`
  Full `Language` trait implementation for Scala. One file. Contents (each item is mandatory; verify each node-kind against the live `tree-sitter-scala 0.22.1` grammar's `node-types.json` before leaving it in):
  - `name() == "scala"`
  - `self_keyword() == Some("this")`
  - `class_query` capturing `class_definition`, `object_definition`, `trait_definition`, and `enum_definition` with `(identifier) @name` → `@definition.class`
  - `function_declaration_query` capturing `function_definition` with `(identifier) @name` and `(parameters) @parameters` → `@definition.function`. Also capture `function_declaration` (abstract def) for parity. Also capture lambda-style values where appropriate.
  - `field_query` capturing `class_parameter`, `val_definition`, `var_definition`, and `(this_expression) … (field_expression …)` field-access patterns inside class/object/trait bodies
  - Node buckets:
    - `if_nodes`: `if_expression`
    - `else_nodes`: `else_clause` (if exposed) or empty
    - `ternary_nodes`: empty (Scala uses `if`-as-expression, already covered)
    - `switch_nodes`: `match_expression`
    - `case_nodes`: `case_clause`
    - `loop_nodes`: `for_expression`, `while_expression`, `do_while_expression` (verify exact names)
    - `except_nodes`: `catch_clause` / `case_clause`-under-`try_expression` (whichever the grammar exposes; verify)
    - `try_expression_nodes`: `try_expression`
    - `jump_nodes`: `return_expression` plus `throw_expression` (Scala has no `break`/`continue` keywords; `scala.util.control.Breaks` is library-level and not a syntactic jump)
    - `return_nodes`: `return_expression`
    - `binary_nodes`: `infix_expression` (Scala's boolean operators are infix expressions; the bucket also feeds boolean-operator pruning via `boolean_operator_nodes`)
    - `boolean_operator_nodes`: `&&`, `||`
    - `field_nodes`: `field_expression` (also `this_expression` if needed for LCOM aggregation)
    - `call_nodes`: `call_expression`
    - `function_nodes`: `function_definition`
    - `closure_nodes`: `lambda_expression` (verify; may be `function_literal` or similar in this grammar version)
    - `comment_nodes`: `comment`, `block_comment`
    - `string_nodes`: `string`, `interpolated_string_expression`
    - `block_nodes`: `block`
    - `invisible_container_nodes`: `compilation_unit` (top-level, verify name)
  - `constructor_names()`: include the grammar's primary-constructor and auxiliary-constructor node names (Scala typically encodes the primary constructor inline on `class_definition`; auxiliary constructors are `function_definition` named `this`. Treat `this` as the constructor name and surface that via `constructor_names()` like other languages do.)
  - `function_name_node()`: default unless the grammar puts the def name somewhere other than the `name` field (verify — Scala typically uses `name:` correctly)
  - `get_parameter_names()`: walk `parameters` → `parameter` → `(identifier)`. Implement only if the default child-walk doesn't work.
  - `iterator_method_identifiers()`: include the standard Scala collection methods that drive complexity (`map`, `flatMap`, `filter`, `foreach`, `fold`, `foldLeft`, `foldRight`, `reduce`, `collect`, `find`, `forall`, `exists`, `count`, `groupBy`, `sortBy`, `zip`, `partition`). Mirror the Kotlin list as a starting point; trim and adjust to Scala's standard library.
  - `mutually_exclusive` test plus targeted node-kind unit tests under `#[cfg(test)] mod tests` covering: a class-with-methods snippet, an object snippet, a trait snippet, a case class, a `match` with guards, a for-comprehension, a `try`/`catch` block, an expression-bodied def, an auxiliary constructor (`def this(...)`), and a malformed snippet through `File::from_string("scala", ...)` that does not panic.
- Modify `qlty-config/default.toml`
  Add (placed near the other JVM languages for readability):
  ```toml
  [language.scala]
  globs = ["*.scala", "*.sc"]
  smells.duplication.filter_patterns = ["(import_declaration _)"]
  ```
  If the verified grammar node for Scala imports is named differently (e.g. `import_clause` or `import_expression`), use that exact name. The Identical/duplication fixtures will catch any mismatch.
- Modify `qlty-types/src/lib.rs`
  Add `"scala" => analysis::v1::Language::Scala,` to the `language_enum_from_name` match (placed alphabetically between `ruby` and `swift`).
- Create `qlty-types/tests/language_enum.rs`
  If the file already exists from the vbnet branch (it does on that branch but not yet on this one), append a Scala assertion. If it does not yet exist on `main`, create it with a single test asserting `language_enum_from_name("scala") == analysis::v1::Language::Scala`.
- Modify `qlty-cli/tests/lang.rs`
  Append `fn scala_tests()` mirroring `kotlin_tests`.
- Create the CLI fixture suite under `qlty-cli/tests/lang/scala/`:
  - `basic.toml` — `bin.name = "qlty"`, `args = ["build", "--no-plugins", "--print"]`, `status.code = 0`
  - `basic.stdout` — recorded snapshot
  - `basic.in/.gitignore` — `target/` etc, mirror `java/basic.in/.gitignore`
  - `basic.in/.qlty/qlty.toml` — `config_version = "0"`
  - `basic.in/BooleanLogic.scala` — long boolean expression, mixed `&&`/`||`, to exercise `boolean_logic` smell
  - `basic.in/FileComplexity.scala` — long file with many methods to exercise `file_complexity` smell
  - `basic.in/FunctionComplexity.scala` — single very-complex method to exercise `function_complexity` smell
  - `basic.in/Identical.scala` — class with two near-identical methods to exercise duplication (Identical Code)
  - `basic.in/Lines.scala` — exercises lines-of-code/comment-line metrics; includes block comments and line comments
  - `basic.in/Members.scala` — class + companion object + trait + case class; exercises classes/functions/fields/lcom
  - `basic.in/NestedControl.scala` — deeply nested `if`/`for`/`match` to exercise `nested_control` smell
  - `basic.in/Parameters.scala` — method with too many parameters to exercise `parameters` smell
  - `basic.in/Returns.scala` — method with many `return` statements to exercise `returns` smell
  - `basic.in/MatchGuards.scala` — `match` with `case … if guard` to exercise cyclomatic/cognitive on guards
  - `basic.in/ForComprehension.scala` — `for { … yield }` to exercise looping
  - `basic.in/Scala3Sample.scala` — small Scala 3 indentation-style snippet to surface grammar ceiling. If the grammar cannot parse it, that file should produce zero metrics rather than panic; the `basic.stdout` snapshot will record whatever the actual behavior is.
- Create `qlty-cli/tests/cmd/metrics/scala_json.{toml,stdout}` plus `scala_json.in/.gitignore`, `.qlty/qlty.toml`, `example.scala`
  Focused regression that proves `qlty metrics --all --json` emits `LANGUAGE_SCALA`.
- Create `qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.{toml,stdout}` plus its `.in/` directory with `.qlty/qlty.toml`, `Identical.scala`, `Identical2.scala`
  Proves duplicated `import` lines are filtered while duplicated executable code remains eligible.
- Add `mod scala` blocks (one Scala-specific scenario per file, following the project's "one thing per test" rule and the existing `mod java`/`mod kotlin`/`mod ruby` patterns) in:
  - `qlty-smells/src/metrics/metrics/cyclomatic.rs` — `match` with three case clauses + a guard increments cyclomatic correctly
  - `qlty-smells/src/metrics/metrics/cognitive.rs` — nested `for`-comprehension with `if` guard increments cognitive with proper nesting weight
  - `qlty-smells/src/metrics/metrics/fields.rs` — `val`, `var`, and class parameter all count as fields exactly once
  - `qlty-smells/src/metrics/metrics/lines_of_code.rs` — block comment vs line comment counted correctly
  - `qlty-smells/src/metrics/metrics/functions.rs` — auxiliary constructor (`def this(...)`) counted as a function, primary constructor not double-counted
  - `qlty-smells/src/metrics/metrics/classes.rs` — `class`, `object`, `trait`, `case class` each counted as one class
  - `qlty-smells/src/metrics/metrics/returns.rs` — explicit `return` counted; tail-expression implicit return not counted
  - `qlty-smells/src/metrics/metrics/parameters.rs` — multi-parameter-list (curried) method's total parameter count is the sum across lists
  - `qlty-smells/src/metrics/metrics/booleans.rs` — `&&`/`||` chain counted correctly via `infix_expression`
  - `qlty-smells/src/structure/checks/nested_control.rs` — nested `for` inside `if` triggers smell at expected nesting depth
  - `qlty-smells/src/structure/checks/returns.rs` — function with N returns triggers smell when over threshold
  - `qlty-smells/src/structure/checks/parameters.rs` — function over parameter threshold triggers smell
  - `qlty-smells/src/structure/checks/boolean_logic.rs` — long boolean chain triggers smell
  - `qlty-smells/src/structure/checks/function_complexity.rs` — complex function triggers smell
  - `qlty-smells/src/structure/checks/file_complexity.rs` — complex file triggers smell
  - One unit test (placed in `qlty-analysis/src/lang/scala.rs` `mod tests`) using `File::from_string("scala", "<malformed>")` and running it through any metric counter to assert no panic.

## Residual Risks

- **Scala 3 syntax coverage.** `tree-sitter-scala 0.22.1` has partial Scala 3 support. Unparseable Scala 3 files emit zero metrics rather than panic. Document in `Scala3Sample.scala` and accept the recorded `basic.stdout`.
- **`*.sc` script-file edge cases.** Top-level definitions outside any class parse fine in this grammar version, but ammonite-style magic imports (`$ivy`) may not parse. If the fixture surfaces a real failure, narrow globs to `*.scala` and add a follow-up issue.
- **Grammar node-kind drift.** Every node-kind string in `scala.rs` must be verified against the live `tree-sitter-scala 0.22.1` `node-types.json`. The `mutually_exclusive()` test plus the targeted unit tests will fail loudly if a string is wrong; do not paper over those failures by removing the strings — fix them.

---

## Tasks

### Task 1: Register Scala And Implement The Language Contract

**Files:**

- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `qlty-analysis/Cargo.toml`
- Modify: `qlty-analysis/src/lang.rs`
- Create: `qlty-analysis/src/lang/scala.rs`
- Modify: `qlty-config/default.toml`
- Modify: `qlty-types/src/lib.rs`
- Create or Modify: `qlty-types/tests/language_enum.rs`

- [ ] **Step 1: Identify or write the failing test**

Write the smallest red tests that prove registration and parser contract:

- `qlty-types/tests/language_enum.rs`:

  ```rust
  use qlty_types::language_enum_from_name;
  use qlty_types::analysis::v1::Language;

  #[test]
  fn scala_maps_to_scala_language_enum() {
      assert_eq!(language_enum_from_name("scala"), Language::Scala);
  }
  ```

- In `qlty-analysis/src/lang/scala.rs` `#[cfg(test)] mod tests`, add:
  - `fn registered_in_all_langs()` — `lang::from_str("scala")` returns `Some(_)` and `.name() == "scala"`
  - `fn parses_class()` — `File::from_string("scala", "class Foo { def bar(x: Int): Int = x + 1 }")` runs through `class_query` and yields one match
  - `fn parses_object()` — same pattern with `object Foo { … }`
  - `fn parses_trait()` — same pattern with `trait Foo { def bar(): Int }`
  - `fn parses_case_class()` — `case class Point(x: Int, y: Int)` exposes `x` and `y` as fields
  - `fn parses_match_with_guard()` — confirms `match_expression` and `case_clause` node kinds exist
  - `fn parses_for_comprehension()` — confirms `for_expression` node kind
  - `fn parses_aux_constructor_named_this()` — `def this(x: Int) = this()` is recognized as a constructor via `function_name_node()` returning `"this"`
  - `fn mutually_exclusive_node_buckets()` — calls the standard `mutually_exclusive` helper used by other languages
  - `fn malformed_input_does_not_panic()` — `File::from_string("scala", "class { ::: !!! ")` plus a metric counter call returns without panic

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p qlty-types --test language_enum scala
cargo test -p qlty-analysis scala
```
Expected: FAIL — `"scala"` is not mapped, the module does not exist, and `tree-sitter-scala` is not a dependency.

- [ ] **Step 3: Write minimal implementation**

Implement the real Scala language contract:

- Workspace deps: `Cargo.toml` add `tree-sitter-scala = "0.22.1"` to `[workspace.dependencies]`.
- `qlty-analysis/Cargo.toml`: add `tree-sitter-scala.workspace = true` to `[dependencies]`.
- `qlty-analysis/src/lang.rs`: `mod scala;`, re-export, push `Box::<scala::Scala>::default()` into `ALL_LANGS`.
- `qlty-config/default.toml`: add the `[language.scala]` section per File Structure above.
- `qlty-types/src/lib.rs`: add `"scala" => analysis::v1::Language::Scala,` arm.
- `qlty-analysis/src/lang/scala.rs`: full `Language` trait impl as described in File Structure. Take `kotlin.rs` as the closest analog and adapt every node-kind string against the live `tree-sitter-scala 0.22.1` `node-types.json` (vendored copy lives under the crate's `src/` after `cargo build`). Include the queries (`CLASS_QUERY`, `FUNCTION_DECLARATION_QUERY`, `FIELD_QUERY`) wired through a `Default` impl. Include `iterator_method_identifiers()` with the Scala collections list. Include the `#[cfg(test)] mod tests` with the tests written in Step 1.

When uncertain about a node kind, run a quick scratch script:
```bash
cargo run -q -p qlty-analysis --example dump_nodes scala 'class Foo { def bar = 1 }'
```
(If no such example exists in the repo, write a one-off `dbg!` test inside `mod tests` that prints `node.kind()` for the parse tree, run it, then delete the `dbg!` before committing.)

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -p qlty-types --test language_enum scala
cargo test -p qlty-analysis scala
```
Expected: PASS

- [ ] **Step 5: Refactor and verify**

- Remove duplicate node-name constants and dead query captures.
- Confirm every node-kind string against the actual grammar — do not leave guesses.
- Match the file shape and ordering of `kotlin.rs` for reviewer familiarity.
- Run the broader required suite to catch any registration regressions:

```bash
qlty fmt
qlty check --level=low --fix
cargo check
cargo test
```
Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock qlty-analysis/Cargo.toml qlty-analysis/src/lang.rs qlty-analysis/src/lang/scala.rs qlty-config/default.toml qlty-types/src/lib.rs qlty-types/tests/language_enum.rs
git commit -m "feat: register scala language support"
```

### Task 2: Add Scala-Flavored Metrics And Smells Unit Coverage

**Files:**

- Modify: `qlty-smells/src/metrics/metrics/cyclomatic.rs`
- Modify: `qlty-smells/src/metrics/metrics/cognitive.rs`
- Modify: `qlty-smells/src/metrics/metrics/fields.rs`
- Modify: `qlty-smells/src/metrics/metrics/lines_of_code.rs`
- Modify: `qlty-smells/src/metrics/metrics/functions.rs`
- Modify: `qlty-smells/src/metrics/metrics/classes.rs`
- Modify: `qlty-smells/src/metrics/metrics/returns.rs`
- Modify: `qlty-smells/src/metrics/metrics/parameters.rs`
- Modify: `qlty-smells/src/metrics/metrics/booleans.rs`
- Modify: `qlty-smells/src/structure/checks/nested_control.rs`
- Modify: `qlty-smells/src/structure/checks/returns.rs`
- Modify: `qlty-smells/src/structure/checks/parameters.rs`
- Modify: `qlty-smells/src/structure/checks/boolean_logic.rs`
- Modify: `qlty-smells/src/structure/checks/function_complexity.rs`
- Modify: `qlty-smells/src/structure/checks/file_complexity.rs`

- [ ] **Step 1: Identify or write the failing test**

In each file above, add `mod scala { … }` (or extend if one exists) inside the existing `#[cfg(test)] mod tests` block. Mirror the structure of the existing `mod java` / `mod kotlin` / `mod python` block in the same file. Add exactly one Scala-specific scenario per file as listed under File Structure. Each test follows the project rule: one assertion focus per test, no control flow, no custom assertion messages, `.unwrap()` is fine.

Skeleton example for `qlty-smells/src/metrics/metrics/cyclomatic.rs`:
```rust
mod scala {
    use super::*;

    #[test]
    fn match_with_guard() {
        let source = r#"
class Demo {
  def classify(n: Int): String = n match {
    case x if x < 0 => "neg"
    case 0 => "zero"
    case _ => "pos"
  }
}
"#;
        let file = File::from_string("scala", source);
        let count = Cyclomatic::default().count(&file);
        assert_eq!(count, /* recorded value */);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p qlty-smells scala
```
Expected: FAIL on the first run because expected counts have not yet been recorded. Use one quick green run to record the actual values, then lock them into the assertions. Do not weaken assertions to make them pass — if a recorded value disagrees with what the metric *should* count for the snippet, fix `scala.rs` (likely a missing node-kind in a bucket).

- [ ] **Step 3: Write minimal implementation**

If any test fails because of a real metric defect (wrong cyclomatic count, missing field, etc.), fix the underlying defect in `qlty-analysis/src/lang/scala.rs` — almost always a node-kind string in a bucket. Do not change the test's assertion to match wrong behavior.

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -p qlty-smells scala
```
Expected: PASS for every new `mod scala` test.

- [ ] **Step 5: Refactor and verify**

- Remove any throwaway `dbg!` calls.
- Make sure each test asserts one thing.
- Re-run the broader suite:

```bash
qlty fmt
qlty check --level=low --fix
cargo check
cargo test
```
Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add qlty-smells/src qlty-analysis/src/lang/scala.rs
git commit -m "feat: add scala metrics and structure unit coverage"
```

### Task 3: Prove End-To-End Scala Maintainability Through The CLI

**Files:**

- Modify: `qlty-cli/tests/lang.rs`
- Create: `qlty-cli/tests/lang/scala/basic.toml`
- Create: `qlty-cli/tests/lang/scala/basic.stdout`
- Create: `qlty-cli/tests/lang/scala/basic.in/.gitignore`
- Create: `qlty-cli/tests/lang/scala/basic.in/.qlty/qlty.toml`
- Create: `qlty-cli/tests/lang/scala/basic.in/BooleanLogic.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/FileComplexity.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/FunctionComplexity.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/Identical.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/Lines.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/Members.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/NestedControl.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/Parameters.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/Returns.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/MatchGuards.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/ForComprehension.scala`
- Create: `qlty-cli/tests/lang/scala/basic.in/Scala3Sample.scala`
- Create: `qlty-cli/tests/cmd/metrics/scala_json.toml`
- Create: `qlty-cli/tests/cmd/metrics/scala_json.stdout`
- Create: `qlty-cli/tests/cmd/metrics/scala_json.in/.gitignore`
- Create: `qlty-cli/tests/cmd/metrics/scala_json.in/.qlty/qlty.toml`
- Create: `qlty-cli/tests/cmd/metrics/scala_json.in/example.scala`
- Create: `qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.toml`
- Create: `qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.stdout`
- Create: `qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.in/.qlty/qlty.toml`
- Create: `qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.in/Identical.scala`
- Create: `qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.in/Identical2.scala`

- [ ] **Step 1: Identify or write the failing test**

- Add `fn scala_tests()` to `qlty-cli/tests/lang.rs`:
  ```rust
  #[test]
  fn scala_tests() {
      setup_and_run_test_cases("tests/lang/scala/**/*.toml");
  }
  ```
- Build the fixture suite under `qlty-cli/tests/lang/scala/` per File Structure. Use the Java fixture suite (`qlty-cli/tests/lang/java/basic.in/`) as the structural reference — copy its `.gitignore` and `.qlty/qlty.toml` verbatim.
- Each `.scala` fixture file must trigger its target metric/smell at least once. `Members.scala` covers the four type-shape cases (class + companion object + trait + case class). `MatchGuards.scala` and `ForComprehension.scala` exercise Scala-specific control flow. `Scala3Sample.scala` is a small Scala 3 indented snippet that may parse partially or not at all — record whatever the actual behavior is.
- Add the `metrics/scala_json.*` regression with a single `.scala` file and assert `LANGUAGE_SCALA` in the recorded stdout.
- Add the `smells/scala_ignore_duplication_patterns.*` regression with two `.scala` files where the only duplication is `import …` lines plus one block of duplicated executable code; the recorded stdout must show the executable block as duplicated and the imports as filtered.
- The `basic.toml` is identical in shape to `java/basic.toml`:
  ```toml
  bin.name = "qlty"
  args = ["build", "--no-plugins", "--print"]
  status.code = 0
  ```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p qlty --test integration scala_tests
cargo test -p qlty --test integration metrics_tests -- scala
cargo test -p qlty --test integration smells_tests -- scala
```
Expected: FAIL — snapshots are empty or stale; trycmd will print the recorded vs actual diff. On the first run, use `TRYCMD=overwrite` (or the project's documented snapshot-update mechanism) only after eyeballing the actual output to confirm it's correct. Do not blindly accept output. Specifically verify:
- `LANGUAGE_SCALA` appears in the metrics output.
- `Identical.scala` and `Identical2.scala` produce one duplication issue each for the executable block, none for the import lines.
- `BooleanLogic.scala`, `FileComplexity.scala`, `FunctionComplexity.scala`, `NestedControl.scala`, `Parameters.scala`, `Returns.scala` each emit at least one structure issue of the matching kind.
- `Members.scala` emits class/function/field stats for class, object, trait, and case class.
- `Scala3Sample.scala` either parses cleanly or produces zero issues — never panics, never crashes the run.

- [ ] **Step 3: Write minimal implementation**

Fix product behavior until snapshots are faithful:
- Adjust queries in `scala.rs` if file/function stats or structure issues are missing.
- Adjust node buckets if `boolean_logic`, `returns`, `loops`, or `nested_control` are undercounted.
- Adjust `function_name_node()` or `get_parameter_names()` if names or counts surface incorrectly.
- Finalize the `import_declaration` filter pattern in `default.toml` against the real grammar node.
- Do not weaken snapshots to accept missing Scala behavior. If `Scala3Sample.scala` reveals a real grammar limitation, document it inline in the file as a comment and accept the recorded reduced stats.

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -p qlty --test integration scala_tests
cargo test -p qlty --test integration metrics_tests -- scala
cargo test -p qlty --test integration smells_tests -- scala
```
Expected: PASS.

- [ ] **Step 5: Refactor and verify**

This is the completion gate for the whole feature. Run the repo-required checks plus the regression surfaces of neighboring languages whose tests share the registration and config files we touched:

```bash
cargo test -p qlty --test integration scala_tests
cargo test -p qlty --test integration kotlin_tests
cargo test -p qlty --test integration java_tests
cargo test -p qlty --test integration ruby_tests
cargo test -p qlty --test integration rust_tests
qlty fmt
qlty check --level=low --fix
cargo check
cargo test
```
Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add qlty-cli/tests/lang.rs qlty-cli/tests/lang/scala qlty-cli/tests/cmd/metrics/scala_json.toml qlty-cli/tests/cmd/metrics/scala_json.stdout qlty-cli/tests/cmd/metrics/scala_json.in qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.toml qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.stdout qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.in qlty-config/default.toml qlty-analysis/src/lang/scala.rs
git commit -m "feat: add scala maintainability support"
```

Open the PR in draft mode with title `feat: add scala maintainability support` and a Conventional Commits-formatted description summarizing parser registration, fixture coverage, and the Scala 3 residual risk.
