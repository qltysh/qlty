# Scala Maintainability Support — Test Plan

This test plan reconciles the agreed testing strategy with the implementation plan at `docs/plans/2026-04-29-scala-maintainability-support.md`. The strategy still holds: every harness it relies on (direct `File::from_string` API, programmatic planner, trycmd CLI fixture, JSON output capture) already exists in the workspace and needs zero new infrastructure. No paid services, no external infra, no new harness code. The implementation plan adds two CLI fixtures (`metrics/scala_json.*` and `smells/scala_ignore_duplication_patterns.*`) beyond what the strategy enumerated; both reuse the existing trycmd harness and tighten user-visible coverage of `LANGUAGE_SCALA` emission and `import` filtering, so they are folded into this plan without expanding scope.

## Strategy reconciliation notes (no user approval required)

- The strategy explicitly named only the `lang/scala/basic.{toml,in,stdout}` fixture as the end-to-end check. The implementation plan adds two narrower CLI regressions (`metrics/scala_json.*` for `LANGUAGE_SCALA` emission, `smells/scala_ignore_duplication_patterns.*` for the `import_declaration` filter). Both reuse the existing trycmd harness, both prove user-visible behavior, and neither expands cost or scope. Folded in.
- The strategy mentioned a `mod scala` per-metric block in every analogous file. The implementation plan enumerates the exact files (9 metrics, 6 structure checks). Adopted verbatim.
- The strategy called for a malformed-input non-panic test. The implementation plan places it inside `qlty-analysis/src/lang/scala.rs#mod tests`. Kept there — that's where parser harnesses live in every other language.
- `qlty-types/tests/language_enum.rs` may need to be created if it does not exist on the branch. Either way, a single assertion test for `language_enum_from_name("scala") == Language::Scala` is required.

## Harness requirements

No new harnesses. Existing harnesses used:

- **Direct API harness** — `qlty_analysis::code::File::from_string("scala", source)` plus the `Cyclomatic::default().count(&file)`-style metric callers and `Check`-trait check callers. Used by the per-metric `mod scala` blocks and by the malformed-input non-panic test.
- **trycmd CLI fixture harness** — `qlty-cli/tests/lang.rs::setup_and_run_test_cases` and `qlty-cli/tests/integration.rs` driving the real `qlty` binary against `.in/` fixture trees and diffing stdout against `.stdout` snapshots. Used by all CLI scenario tests.
- **Workspace registration harness** — `qlty_analysis::lang::from_str` lookup and `qlty_types::language_enum_from_name` lookup. Used by registration tests.

---

## Test plan

Tests are ordered by how directly they prove the user goal: "running `qlty metrics`/`qlty smells`/`qlty build` against a Scala project produces real metrics and real smells, the same way it does for Java/Kotlin/Ruby." The trycmd CLI fixtures sit at the top because they are the only tests that exercise the real binary against real `.scala` files end-to-end.

### 1. `qlty build --no-plugins --print` against a representative Scala project emits structure issues, duplication issues, and stats

- **Name**: Running `qlty build --no-plugins --print` on a Scala project emits the same kinds of issues and stats it emits for Java
- **Type**: scenario
- **Disposition**: new
- **Harness**: trycmd CLI fixture harness
- **Preconditions**: A new fixture tree at `qlty-cli/tests/lang/scala/basic.in/` containing `.qlty/qlty.toml` (`config_version = "0"`), `.gitignore`, and the per-smell trigger files: `BooleanLogic.scala`, `FileComplexity.scala`, `FunctionComplexity.scala`, `Identical.scala`, `Lines.scala`, `Members.scala` (class + companion object + trait + case class), `NestedControl.scala`, `Parameters.scala`, `Returns.scala`, `MatchGuards.scala`, `ForComprehension.scala`, `Scala3Sample.scala`. `qlty-cli/tests/lang.rs` registers a `scala_tests()` invocation of `setup_and_run_test_cases("tests/lang/scala/**/*.toml")`.
- **Actions**: `qlty build --no-plugins --print` (driven by `basic.toml` with `bin.name = "qlty"`, `args = ["build", "--no-plugins", "--print"]`, `status.code = 0`).
- **Expected outcome**: `basic.stdout` snapshot contains, at minimum: a `boolean_logic` issue keyed to `BooleanLogic.scala`, a `file_complexity` issue keyed to `FileComplexity.scala`, a `function_complexity` issue keyed to `FunctionComplexity.scala`, an identical-code duplication issue spanning `Identical.scala` (and any pair file), a `nested_control` issue keyed to `NestedControl.scala`, a `parameters` issue keyed to `Parameters.scala`, a `returns` issue keyed to `Returns.scala`, file-level stats lines for `Members.scala` showing classes/functions/fields > 0, and exit code 0. Source of truth: the implementation plan's "Desired End State" plus the equivalent rows that already appear in `qlty-cli/tests/lang/java/basic.stdout` for the analogous Java fixture.
- **Interactions**: `qlty-config/default.toml` `[language.scala]` glob registration, `qlty-analysis/src/lang/scala.rs` query and node-bucket coverage, `qlty-types::language_enum_from_name`, and the duplication filter pipeline. This is the highest-fidelity end-to-end check; if any seam is wrong, this fails.

### 2. `qlty metrics --all --json` emits `LANGUAGE_SCALA` for `.scala` files

- **Name**: Running `qlty metrics --all --json` on a Scala source file emits `LANGUAGE_SCALA` in the JSON output
- **Type**: regression
- **Disposition**: new
- **Harness**: trycmd CLI fixture harness
- **Preconditions**: New fixture at `qlty-cli/tests/cmd/metrics/scala_json.in/` with `.gitignore`, `.qlty/qlty.toml` (`config_version = "0"`), and a single non-trivial `example.scala` containing a class with at least one method.
- **Actions**: `qlty metrics --all --json` (driven by `scala_json.toml`).
- **Expected outcome**: Recorded `scala_json.stdout` snapshot includes the literal string `"LANGUAGE_SCALA"` and emits per-file/per-function stats records for `example.scala`. Source of truth: `qlty.analysis.v1.Language::Scala` exists in the proto (`qlty-types/src/protos/qlty.analysis.v1.rs:771`); the plan requires `language_enum_from_name("scala")` to resolve to it.
- **Interactions**: `qlty-types::language_enum_from_name`, the metrics serializer, file discovery via `[language.scala].globs`.

### 3. `qlty smells` filters duplicated `import` statements but reports duplicated executable code

- **Name**: When two Scala files share `import` lines and one duplicated method body, only the method body is reported as a duplication issue
- **Type**: scenario
- **Disposition**: new
- **Harness**: trycmd CLI fixture harness
- **Preconditions**: New fixture at `qlty-cli/tests/cmd/smells/scala_ignore_duplication_patterns.in/` with `.qlty/qlty.toml` (`config_version = "0"`), `Identical.scala`, and `Identical2.scala`. Both files share an identical block of `import` statements at the top and one identical method body. The remainder of each file is unique enough to fall below similarity threshold.
- **Actions**: `qlty smells --all --no-snippets --json` (or whichever invocation the existing analogous Kotlin/Java fixture uses; mirror that fixture's `args`).
- **Expected outcome**: Recorded snapshot shows exactly one identical-code duplication issue covering the duplicated method body, and no duplication issue covering the import block. Source of truth: implementation plan's "Contracts And Invariants" — `smells.duplication.filter_patterns = ["(import_declaration _)"]` in `qlty-config/default.toml`.
- **Interactions**: `default.toml` filter pattern, the duplication-pattern filter pipeline, and the verified Scala node kind for imports (`import_declaration`). If the real grammar names the node differently, this test fails first and forces the right fix.

### 4. The Scala language registers in `ALL_LANGS` and resolves by name

- **Name**: `qlty_analysis::lang::from_str("scala")` returns a registered Scala language whose `name()` is `"scala"`
- **Type**: integration
- **Disposition**: new
- **Harness**: workspace registration harness
- **Preconditions**: `qlty-analysis/src/lang/scala.rs` exists, `mod scala` is added to `qlty-analysis/src/lang.rs`, and `Box::<scala::Scala>::default()` is appended to `ALL_LANGS`.
- **Actions**: From `#[cfg(test)] mod tests` in `qlty-analysis/src/lang/scala.rs`, call `qlty_analysis::lang::from_str("scala")`.
- **Expected outcome**: The result is `Some(_)` and the wrapped language's `name()` returns `"scala"`. Source of truth: implementation plan's "Contracts And Invariants" requirement that `from_str("scala")` resolves and that the canonical name is `scala`.
- **Interactions**: `lang.rs` module wiring.

### 5. `language_enum_from_name("scala")` resolves to `Language::Scala`

- **Name**: The proto-language lookup recognizes `"scala"` and returns the existing `Language::Scala` variant
- **Type**: integration
- **Disposition**: new
- **Harness**: workspace registration harness
- **Preconditions**: `qlty-types/src/lib.rs` adds the `"scala" => analysis::v1::Language::Scala` arm. Test file `qlty-types/tests/language_enum.rs` exists (create if absent on this branch).
- **Actions**: Call `qlty_types::language_enum_from_name("scala")` from the integration test.
- **Expected outcome**: Equals `qlty_types::analysis::v1::Language::Scala`. Source of truth: implementation plan's invariant; the proto enum already contains `LANGUAGE_SCALA`.
- **Interactions**: Proto enum mapping, used by metrics output serialization.

### 6. `match` with guards is counted by cyclomatic complexity

- **Name**: A method whose body is a `match` with three case clauses (one with an `if` guard) increments cyclomatic complexity once per case and once per guard
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/cyclomatic.rs`.
- **Actions**: Build a Scala source string with one `match` containing `case x if x < 0 => …`, `case 0 => …`, `case _ => …`. Call `Cyclomatic::default().count(&File::from_string("scala", source))`.
- **Expected outcome**: Recorded count equals the same count produced for the analogous Java `switch`/`if`-chain plus one for the guard, locked in on first green run. Source of truth: how `mod java`/`mod kotlin` count the analogous construct in the same file.
- **Interactions**: `case_nodes`, `if_nodes`, and the `binary_nodes`/`boolean_operator_nodes` buckets in `scala.rs`.

### 7. Nested for-comprehension with an `if` guard increments cognitive complexity with proper nesting weight

- **Name**: A `for { x <- xs; y <- ys if y > 0 } yield x + y` inside a method increments cognitive complexity at the expected nesting depth
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/cognitive.rs`.
- **Actions**: Build a Scala method containing a nested for-comprehension with a guard. Call `Cognitive::default().count(...)`.
- **Expected outcome**: Recorded count reflects nesting weight (loop +1, nested loop +2, guard +1 at depth 2). Source of truth: cognitive complexity rules already encoded in `mod java`/`mod kotlin` blocks of the same file.
- **Interactions**: `loop_nodes`, `if_nodes`, and the cognitive nesting tracker.

### 8. `val`, `var`, and class parameters are each counted as one field

- **Name**: A class with one `val`, one `var`, and one class parameter exposes three fields, each counted exactly once
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/fields.rs`.
- **Actions**: Build a Scala class `class Foo(p: Int) { val a = 1; var b = 2 }`. Run the field-count metric.
- **Expected outcome**: Field count is 3. Source of truth: `mod rust`/`mod java`/`mod kotlin` patterns in the same file plus the `field_query` defined in `scala.rs`.
- **Interactions**: `field_query`, `field_nodes`.

### 9. Block comments and line comments are counted distinctly in lines-of-code

- **Name**: A Scala file with one block comment and one line comment yields the expected `comment_lines` count
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/lines_of_code.rs`.
- **Actions**: Source includes both `/* … */` and `// …`. Run lines-of-code.
- **Expected outcome**: Recorded counts match the comment-line accounting locked in by analogous languages. Source of truth: `mod java`/`mod ruby` rows in the same file.
- **Interactions**: `comment_nodes` bucket.

### 10. Auxiliary `def this(...)` constructors count as functions; primary constructor does not double-count

- **Name**: A class with one `def` and one auxiliary `def this(x: Int) = this()` reports two functions, not three
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/functions.rs`. `function_name_node()` in `scala.rs` returns `"this"` for auxiliary constructors.
- **Actions**: Build the source, run the function-count metric.
- **Expected outcome**: Function count is 2. Source of truth: implementation plan's `constructor_names()` contract for `this`.
- **Interactions**: `function_nodes`, `constructor_names()`, and primary-constructor handling on `class_definition`.

### 11. `class`, `object`, `trait`, and `case class` each count as one class

- **Name**: A file containing one of each of `class`, `object`, `trait`, and `case class` reports four classes
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/classes.rs`.
- **Actions**: Build a four-shape source file, run the class-count metric.
- **Expected outcome**: Class count is 4. Source of truth: plan's `class_query` covering `class_definition`, `object_definition`, `trait_definition`, `enum_definition`; case classes parse as `class_definition` with a `case` modifier.
- **Interactions**: `class_query` in `scala.rs`.

### 12. Explicit `return` is counted; tail-expression implicit returns are not

- **Name**: A method that uses both an early `return x` and a tail expression reports exactly one `return` for the returns metric
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/returns.rs`.
- **Actions**: Build a Scala method with one `return` and a tail expression, run the returns metric.
- **Expected outcome**: Returns count is 1. Source of truth: `mod ruby`/`mod java` rows in the same file applied to Scala's expression-oriented model where only `return_expression` counts.
- **Interactions**: `return_nodes` bucket; explicitly excludes implicit returns.

### 13. Curried parameter lists sum across lists for the parameter count

- **Name**: A curried method `def add(a: Int)(b: Int)(c: Int): Int` reports a parameter count of 3
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/parameters.rs`.
- **Actions**: Run the parameters metric on the curried method.
- **Expected outcome**: Parameter count is 3. Source of truth: plan's `get_parameter_names()` contract walking each `parameters` group.
- **Interactions**: `function_declaration_query` capture group `@parameters` and the parameter walker.

### 14. `&&`/`||` chains are counted as boolean operators via `infix_expression`

- **Name**: An `if (a && b || c && d)` increments the boolean-operator metric by the expected count
- **Type**: unit
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/metrics/metrics/booleans.rs`.
- **Actions**: Run the boolean-operator metric on a method with that condition.
- **Expected outcome**: Recorded count matches the analogous Java/Kotlin count locked in on first green run. Source of truth: `mod java`/`mod kotlin` rows in the same file.
- **Interactions**: `binary_nodes` and `boolean_operator_nodes` buckets — Scala renders `&&`/`||` as `infix_expression`.

### 15. The `nested_control` smell fires on a `for` inside an `if` past the threshold

- **Name**: A method nesting `for` inside `if` deeply enough to exceed the configured threshold reports a `nested_control` issue at the expected line
- **Type**: scenario
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/structure/checks/nested_control.rs`.
- **Actions**: Build a Scala source crossing the configured nesting threshold, run the `nested_control` check.
- **Expected outcome**: Exactly one issue is reported pointing at the inner control structure. Source of truth: `mod java`/`mod ruby`/`mod go` rows in the same file.
- **Interactions**: `if_nodes`, `loop_nodes`, `case_nodes` cooperate with the structural-nesting check.

### 16. The `returns` smell fires on a method with too many explicit returns

- **Name**: A Scala method with explicit `return` statements above the configured threshold reports one `returns` issue
- **Type**: scenario
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/structure/checks/returns.rs`.
- **Actions**: Build the source, run the check.
- **Expected outcome**: Exactly one issue. Source of truth: `mod ruby` row in the same file.
- **Interactions**: `return_nodes`.

### 17. The `parameters` smell fires on a method with too many parameters

- **Name**: A Scala method with parameter count above threshold reports one `parameters` issue
- **Type**: scenario
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/structure/checks/parameters.rs`.
- **Actions**: Build the source, run the check.
- **Expected outcome**: Exactly one issue. Source of truth: `mod ruby` row in the same file.
- **Interactions**: `function_declaration_query` parameters capture.

### 18. The `boolean_logic` smell fires on a long boolean expression

- **Name**: A Scala condition mixing many `&&` and `||` operators above threshold reports one `boolean_logic` issue
- **Type**: scenario
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/structure/checks/boolean_logic.rs`.
- **Actions**: Build the source, run the check.
- **Expected outcome**: Exactly one issue. Source of truth: existing analogous mods in the same file.
- **Interactions**: `binary_nodes`, `boolean_operator_nodes`.

### 19. The `function_complexity` smell fires on a deliberately complex function

- **Name**: A single Scala method with cyclomatic + cognitive complexity above the configured threshold reports one `function_complexity` issue
- **Type**: scenario
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/structure/checks/function_complexity.rs`.
- **Actions**: Build the source, run the check.
- **Expected outcome**: Exactly one issue. Source of truth: existing analogous mods.
- **Interactions**: All complexity-bearing buckets (`if_nodes`, `loop_nodes`, `case_nodes`, `binary_nodes`, `try_expression_nodes`, `jump_nodes`).

### 20. The `file_complexity` smell fires on a deliberately complex file

- **Name**: A Scala file with aggregate complexity above the configured threshold reports one `file_complexity` issue
- **Type**: scenario
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: `mod scala` block added to `qlty-smells/src/structure/checks/file_complexity.rs`.
- **Actions**: Build the source, run the check.
- **Expected outcome**: Exactly one issue. Source of truth: existing analogous mods.
- **Interactions**: Aggregation across the file's functions.

### 21. The Scala language's node-kind buckets are mutually exclusive

- **Name**: No node kind appears in more than one of the Scala `Language` trait's node-kind lists
- **Type**: invariant
- **Disposition**: new
- **Harness**: direct API harness (existing `mutually_exclusive` helper used by every other `lang/*.rs`)
- **Preconditions**: `qlty-analysis/src/lang/scala.rs` includes a `mutually_exclusive_node_buckets` test mirroring the equivalent test in `kotlin.rs`/`ruby.rs`.
- **Actions**: Run the mutual-exclusion checker on the new language instance.
- **Expected outcome**: No duplicate node kinds across buckets. Source of truth: the same invariant enforced by every other language.
- **Interactions**: All node-bucket lists in `scala.rs`.

### 22. `class_definition`, `object_definition`, `trait_definition`, and `case class` are recognized by the class query

- **Name**: Each of the four Scala type-shape declarations is matched by `class_query`
- **Type**: integration
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: Tests added to `qlty-analysis/src/lang/scala.rs#mod tests` (one per shape).
- **Actions**: Parse a snippet of each shape via `File::from_string("scala", ...)` and run the language's class query.
- **Expected outcome**: Each snippet yields exactly one class match with the expected `name` capture. Source of truth: implementation plan's `class_query` enumeration.
- **Interactions**: tree-sitter-scala node-types.json.

### 23. `function_name_node()` returns `"this"` for an auxiliary constructor

- **Name**: An auxiliary constructor `def this(x: Int) = this()` exposes the function name `"this"`
- **Type**: integration
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: Test added to `qlty-analysis/src/lang/scala.rs#mod tests`.
- **Actions**: Parse the snippet and call `function_name_node()` on the matched function definition.
- **Expected outcome**: Returned name is `"this"`. Source of truth: implementation plan's `constructor_names()` contract.
- **Interactions**: `function_declaration_query`, `constructor_names()`.

### 24. Malformed Scala input does not panic when fed through a metric counter

- **Name**: `File::from_string("scala", "<garbage>")` followed by a metric call returns a number rather than panicking
- **Type**: boundary
- **Disposition**: new
- **Harness**: direct API harness
- **Preconditions**: Test added to `qlty-analysis/src/lang/scala.rs#mod tests`.
- **Actions**: Build a deliberately malformed Scala string (`"class { ::: !!! "`), pass through `Cyclomatic::default().count(...)` (or any metric).
- **Expected outcome**: Returns a `usize` (typically 0 or 1) without panicking. Source of truth: testing-strategy explicit failure-mode requirement.
- **Interactions**: tree-sitter parse-error tolerance, metric counters.

### 25. Existing language CLI fixtures still pass after Scala registration

- **Name**: Adding Scala does not change the recorded outputs for Java, Kotlin, Ruby, or Rust CLI fixtures
- **Type**: regression
- **Disposition**: existing
- **Harness**: trycmd CLI fixture harness
- **Preconditions**: All current `qlty-cli/tests/lang/{java,kotlin,ruby,rust}/**/*.toml` fixtures pass on `main`.
- **Actions**: Run `cargo test -p qlty --test integration kotlin_tests`, `java_tests`, `ruby_tests`, `rust_tests`.
- **Expected outcome**: All recorded snapshots still match. Source of truth: those fixtures themselves; their stdout snapshots define the regression baseline.
- **Interactions**: Shared `default.toml`, shared `ALL_LANGS`, shared `language_enum_from_name` — every place this PR edits is shared infrastructure, so any unintended behavior change in another language surfaces here.

### 26. The full `cargo test` workspace suite stays green

- **Name**: `cargo test` across the workspace passes on the Scala branch
- **Type**: regression
- **Disposition**: existing
- **Harness**: cargo test runner
- **Preconditions**: All implementation tasks complete and the workspace compiles.
- **Actions**: `cargo test`.
- **Expected outcome**: Zero failing tests across all crates. Source of truth: project's CI baseline and `CLAUDE.md` requirement to run all tests before commit.
- **Interactions**: Every crate. Catches accidental coupling.

---

## Coverage summary

**Action space covered:**

- `qlty build --no-plugins --print` against Scala — Test 1 (end-to-end primary acceptance gate).
- `qlty metrics --all --json` for Scala — Test 2.
- `qlty smells --all` duplication filtering for Scala — Test 3.
- File discovery via `[language.scala]` globs for `*.scala` and `*.sc` — covered transitively through Tests 1, 2, 3 (each fixture relies on glob registration to be picked up).
- Workspace registration (`from_str`, `language_enum_from_name`) — Tests 4 and 5.
- All metrics (cyclomatic, cognitive, fields, lines-of-code, functions, classes, returns, parameters, booleans) — Tests 6–14.
- All structure smells (nested_control, returns, parameters, boolean_logic, function_complexity, file_complexity) — Tests 15–20.
- `Language` trait internal consistency (mutually exclusive buckets, class shapes, constructor-name handling) — Tests 21–23.
- Failure mode (malformed input non-panic) — Test 24.
- Regression safety for adjacent languages and the workspace — Tests 25–26.

**Explicit exclusions, per the agreed strategy:**

- **Scala 3-only syntax (givens/extensions, indented blocks, enums beyond what `enum_definition` covers).** Coverage is best-effort; `Scala3Sample.scala` in the basic fixture surfaces whatever the grammar can do today and locks it into the `basic.stdout` snapshot. Risk: future Scala 3 idioms may be analyzed weakly or not at all. Surfaces visibly as zero metrics on those constructs.
- **Ammonite-style `.sc` magic imports (`$ivy`, `$file`).** Not covered. Risk: a `.sc` file using ammonite syntax may fail to parse and emit zero metrics. Mitigated by the malformed-input non-panic guarantee (Test 24): worst case is silent under-reporting, never a crash.
- **Manual UI/QA inspection.** Excluded by strategy and by skill rules. All checks are reproducible artifacts.
- **Performance benchmarking.** Strategy assessed performance risk as low. No perf test included; `cargo test` wall time is the implicit floor.
- **Differential comparison against an external Scala metrics tool.** No runnable reference exists in this environment; strategy explicitly noted N/A.
- **Duplication-threshold tuning across Scala idioms.** Out of scope; flagged as a follow-up tuning task if Test 1's recorded snapshot reveals noise.

**Test mix balance:** 9 unit tests (6–14), 11 scenario/integration/regression tests (1–5, 15–20, 22–23, 25), 2 invariant/boundary tests (21, 24), 1 workspace regression (26). Unit tests are 35% of the total, the majority of evidence (Tests 1–5, 15–20, 25–26) is real-system or product-behavior tests, satisfying the prioritization rule that unit tests must not dominate.
