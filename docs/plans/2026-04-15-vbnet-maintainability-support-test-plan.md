# VB.NET Maintainability Support Test Plan

This test plan crystallizes and extends the testing strategy embedded in the implementation plan. The plan targets three layers: unit tests for the parser/language contract and shared metrics normalization, integration tests for the CLI surfaces (`build`, `metrics`, `smells`), and regression gates for existing language suites that share registration files.

Sources of truth: the VB.NET language specification (case-insensitive identifiers), the existing C#/C/C++ maintainability test suites as structural references, and the implementation plan contracts and invariants.

---

## Harness requirements

### H1: trycmd snapshot harness (existing, extend)

- **What it does:** Runs the `qlty` binary against a git-backed fixture repo and compares JSON stdout to a committed snapshot file.
- **Exposes:** Full CLI JSON output (issues, stats, metadata) via `trycmd::TestCases`.
- **Complexity:** Zero build cost -- the harness already exists in `qlty-cli/tests/helpers.rs`.
- **Tests that depend on it:** T05, T06, T07, T08, T09, T10, T11, T12.

### H2: Rust unit test harness (existing, extend)

- **What it does:** Uses `File::from_string("<lang>", "<source>")` to parse source fragments and assert against tree-sitter queries, node traversals, and metrics counters.
- **Exposes:** Direct access to `Language` trait methods, query captures, and metrics `count()` functions.
- **Complexity:** Zero build cost -- pattern established in every existing `lang/*.rs` and `metrics/*.rs` module.
- **Tests that depend on it:** T01, T02, T03, T04, T13, T14, T15, T16, T17, T18, T19.

---

## Test plan

### T01: Language enum maps "vbnet" to LANGUAGE_VBDOTNET

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** `qlty-types` crate compiled with VB.NET mapping added.
- **Actions:** Call `language_enum_from_name("vbnet")`.
- **Expected outcome:** Returns `analysis::v1::Language::Vbdotnet`. Source of truth: implementation plan contract "qlty_types::language_enum_from_name(\"vbnet\") must return analysis::v1::Language::Vbdotnet".
- **Interactions:** None.

### T02: Language registry resolves "vbnet" by name

- **Type:** unit
- **Disposition:** extend (existing `language_names` test in `qlty-analysis/src/lang.rs`)
- **Harness:** H2
- **Preconditions:** `qlty-analysis` crate compiled with VB.NET module registered in `ALL_LANGS`.
- **Actions:** Call `crate::lang::from_str("vbnet")`.
- **Expected outcome:** Returns `Some(lang)` where `lang.name() == "vbnet"`. Source of truth: implementation plan contract "qlty-analysis::lang::from_str(\"vbnet\") must succeed".
- **Interactions:** Tests the `ALL_LANGS` static vector and `from_str` lookup.

### T03: VB.NET parser creates a valid parser instance

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** Grammar crate (published or vendored) wired into `qlty-analysis`.
- **Actions:** Call `VBNet::default().parser()`.
- **Expected outcome:** Does not panic. Source of truth: every existing language module has this test (e.g. `language_parser` for Rust).
- **Interactions:** Exercises tree-sitter grammar loading.

### T04: VB.NET node bucket kinds are mutually exclusive

- **Type:** invariant
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET `Language` trait implementation exists.
- **Actions:** Collect all node kinds from `if_nodes()`, `elsif_nodes()`, `switch_nodes()`, `case_nodes()`, `ternary_nodes()`, `loop_nodes()`, `except_nodes()`, `try_expression_nodes()`, `jump_nodes()`, `return_nodes()`, `binary_nodes()`, `field_nodes()`, `call_nodes()`, `function_nodes()`, `closure_nodes()`, `comment_nodes()`, `string_nodes()`, `boolean_operator_nodes()`, `block_nodes()` into one vector.
- **Expected outcome:** The total count equals the count of unique entries (no duplicates across buckets). Source of truth: every existing language module has a `mutually_exclusive` test (C, C++, C#, etc.).
- **Interactions:** None.

### T05: VB.NET build snapshot produces correct structure issues and stats

- **Type:** scenario
- **Disposition:** new
- **Harness:** H1
- **Preconditions:** VB.NET fixture repo at `qlty-cli/tests/lang/vbnet/basic.in/` containing `.vb` files for boolean logic, nested control, file complexity, function complexity, identical code, line counts, members, parameters, and returns. Git-initialized by the test harness.
- **Actions:** Run `qlty build --no-plugins --print` via trycmd.
- **Expected outcome:** JSON stdout matches `basic.stdout` snapshot. Must include:
  - Issues with `"language": "LANGUAGE_VBDOTNET"` for duplication and structure categories.
  - Stats entries for each `.vb` file with `"language": "LANGUAGE_VBDOTNET"` and non-zero `lines`, `codeLines`, `functions`, `classes`, `complexity`, `cyclomatic` counts appropriate to each fixture file's content.
  - Source of truth: parallel C# `basic.stdout` structure for the same fixture categories, plus VB.NET language specification for syntax differences.
- **Interactions:** Exercises the full build pipeline: file discovery via `default.toml` globs, tree-sitter parsing, all metrics executors, all smells executors, and JSON serialization.

### T06: VB.NET metrics JSON emits LANGUAGE_VBDOTNET

- **Type:** integration
- **Disposition:** new
- **Harness:** H1
- **Preconditions:** Fixture repo at `qlty-cli/tests/cmd/metrics/vbnet_json.in/` with a single `example.vb` file.
- **Actions:** Run `qlty metrics --all --json` via trycmd.
- **Expected outcome:** JSON stdout includes a `stats` array entry with `"language": "LANGUAGE_VBDOTNET"`, non-zero line counts, and appropriate metrics. Source of truth: implementation plan desired end state "qlty metrics --all --json emits file and function stats for .vb files with language = LANGUAGE_VBDOTNET".
- **Interactions:** Exercises the metrics CLI path independently from `build`.

### T07: VB.NET Imports lines are filtered from duplication detection

- **Type:** integration
- **Disposition:** new
- **Harness:** H1
- **Preconditions:** Fixture repo at `qlty-cli/tests/cmd/smells/vbnet_ignore_duplication_patterns.in/` with two `.vb` files sharing duplicated `Imports` lines but differing in non-import executable code.
- **Actions:** Run `qlty smells --all --no-snippets --json` via trycmd.
- **Expected outcome:** The output contains zero duplication issues for the `Imports` lines, while duplicated executable code (if present and above threshold) is still detected. Source of truth: implementation plan desired end state "duplicated Imports lines are filtered out like Java import and C# using lines" plus the `[language.csharp]` `smells.duplication.filter_patterns` precedent.
- **Interactions:** Exercises the `default.toml` filter pattern for VB.NET `Imports` statements and the duplication detection pipeline.

### T08: Existing C language tests still pass after VB.NET registration

- **Type:** regression
- **Disposition:** existing
- **Harness:** H1
- **Preconditions:** VB.NET module registered. `qlty-cli/tests/lang/c/` fixtures unchanged.
- **Actions:** Run `cargo test -p qlty --test integration c_tests`.
- **Expected outcome:** All C language snapshot tests pass. Source of truth: C test suite established in commit `a56a8f32`.
- **Interactions:** VB.NET registration modifies shared files (`lang.rs`, `default.toml`, `lib.rs`) that C depends on.

### T09: Existing C++ language tests still pass after VB.NET registration

- **Type:** regression
- **Disposition:** existing
- **Harness:** H1
- **Preconditions:** VB.NET module registered. `qlty-cli/tests/lang/cpp/` fixtures unchanged.
- **Actions:** Run `cargo test -p qlty --test integration cpp_tests`.
- **Expected outcome:** All C++ language snapshot tests pass. Source of truth: C++ test suite established in commit `a56a8f32`.
- **Interactions:** Same shared registration files as T08.

### T10: Existing C# language tests still pass after VB.NET registration

- **Type:** regression
- **Disposition:** existing
- **Harness:** H1
- **Preconditions:** VB.NET module registered. `qlty-cli/tests/lang/csharp/` fixtures unchanged.
- **Actions:** Run `cargo test -p qlty --test integration csharp_tests`.
- **Expected outcome:** All C# language snapshot tests pass. Source of truth: existing C# test suite.
- **Interactions:** C# is the closest sibling language and shares `default.toml` duplication filter patterns.

### T11: Existing Swift language tests still pass after VB.NET registration

- **Type:** regression
- **Disposition:** existing
- **Harness:** H1
- **Preconditions:** VB.NET module registered. `qlty-cli/tests/lang/swift/` fixtures unchanged.
- **Actions:** Run `cargo test -p qlty --test integration swift_tests`.
- **Expected outcome:** All Swift language snapshot tests pass. Source of truth: existing Swift test suite.
- **Interactions:** Shares `lang.rs` registration.

### T12: Existing metrics and smells CLI tests still pass

- **Type:** regression
- **Disposition:** existing
- **Harness:** H1
- **Preconditions:** VB.NET additions do not modify existing test fixtures.
- **Actions:** Run `cargo test -p qlty --test integration metrics_tests` and `cargo test -p qlty --test integration smells_tests`.
- **Expected outcome:** All existing metrics and smells snapshot tests pass. Source of truth: existing test suites.
- **Interactions:** New VB.NET fixtures are added alongside existing ones under the same glob patterns.

### T13: VB.NET class query captures Class and Interface declarations

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET grammar wired.
- **Actions:** Parse a VB.NET snippet containing `Public Class Foo ... End Class` using `File::from_string("vbnet", ...)` and run `class_query()` matches.
- **Expected outcome:** Query returns a `@definition.class` capture with `@name` = "Foo". Source of truth: implementation plan Step 1 "class query capture on a simple Class ... End Class".
- **Interactions:** Validates that the tree-sitter grammar and query are compatible.

### T14: VB.NET function query captures Sub, Function, and Sub New

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET grammar wired.
- **Actions:** Parse snippets containing `Public Sub DoWork()`, `Public Function GetValue() As Integer`, and `Public Sub New()` and run `function_declaration_query()` matches.
- **Expected outcome:** Each produces a `@definition.function` capture with the correct `@name`. Source of truth: implementation plan Step 1 "function query capture on Sub, Function, and Sub New".
- **Interactions:** Validates function name extraction path.

### T15: VB.NET constructor_names includes "New"

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET `Language` implementation exists.
- **Actions:** Call `VBNet::default().constructor_names()`.
- **Expected outcome:** Contains `"New"`. Source of truth: implementation plan contract "constructor_names() recognizing New".
- **Interactions:** Feeds into LCOM constructor exclusion logic.

### T16: VB.NET call_identifiers handles implicit and Me.Member calls

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET grammar wired.
- **Actions:** Parse `Me.DoWork()` and an implicit receiver call and run `call_identifiers()`.
- **Expected outcome:** `Me.DoWork()` returns `(Some("Me"), "DoWork")`. Implicit call returns `(Some("Me"), "<name>")` or `(None, "<name>")` per the grammar's semantics. Source of truth: implementation plan Step 1 "call_identifiers() for implicit receiver calls and Me.Member()".
- **Interactions:** Feeds into LCOM group connectivity and cognitive recursion detection.

### T17: VB.NET field_identifiers handles Me.Field access

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET grammar wired.
- **Actions:** Parse `Me.Value` and run `field_identifiers()`.
- **Expected outcome:** Returns `("Me", "Value")`. Source of truth: implementation plan Step 1 "field_identifiers() for Me.Member" plus C# `field_identifier_read` test pattern.
- **Interactions:** Feeds into LCOM field grouping and field count metrics.

### T18: VB.NET get_parameter_names extracts parameter names correctly

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET grammar wired.
- **Actions:** Parse `Public Sub DoWork(x As Integer, y As String)` and call `get_parameter_names()` on the parameters node.
- **Expected outcome:** Returns `["x", "y"]`. Source of truth: implementation plan Step 1 "get_parameter_names() extracting parameter names from real VB.NET parameter syntax".
- **Interactions:** Feeds into function-parameters smell detection.

### T19: VB.NET function_name_node returns the actual member name

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET grammar wired.
- **Actions:** Parse a VB.NET method declaration and call `function_name_node()` on the function node.
- **Expected outcome:** Returns a node whose text is the method name (not the keyword `Sub`/`Function`). Source of truth: implementation plan Step 1 "function_name_node() returning the actual VB.NET member name node".
- **Interactions:** Feeds into cognitive complexity recursion tracking and all function-name-based metrics.

### T20: LCOM skips VB.NET constructor with case-insensitive "New" matching

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET language module with `normalize_identifier()` override.
- **Actions:** Create a VB.NET source with a class containing `Sub New()` that accesses `Me.field`, and compute LCOM.
- **Expected outcome:** LCOM = 0 (constructor excluded from group counting). Must also work if the source says `sub new` or `Sub NEW`. Source of truth: implementation plan Task 2 Step 1 "VB.NET Sub New or mixed-case sub new is still treated as a constructor".
- **Interactions:** Exercises the `is_constructor()` function with normalized identifiers.

### T21: LCOM groups connect across mixed-case method calls

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET language module with `normalize_identifier()` override.
- **Actions:** Create a VB.NET source with a class where method A calls `Me.dothing()` and method B is declared as `Sub DoThing()`. Both access the same field. Compute LCOM.
- **Expected outcome:** LCOM = 1 (single connected group, not 2 disconnected groups). Source of truth: implementation plan Task 2 Step 1 "method group still connects when one method calls Me.dothing() and the declaration is DoThing".
- **Interactions:** Exercises normalized identifier insertion in LCOM `Group.functions` set.

### T22: LCOM field groups unify across mixed-case Me.field access

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET language module with `normalize_identifier()` override.
- **Actions:** Create a VB.NET source with a class where method A accesses `Me.Value` and method B accesses `me.value`. Compute LCOM.
- **Expected outcome:** LCOM = 1 (one group connected by the shared field, not 2 groups). Source of truth: implementation plan Task 2 Step 1 "Me.Value and me.value end up in the same field group".
- **Interactions:** Exercises normalized field name insertion in LCOM `Group.fields` set.

### T23: Cognitive complexity counts mixed-case self-recursion

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET language module with `normalize_identifier()` override.
- **Actions:** Create a VB.NET source with a function `DoWork` that calls `Me.dowork()`. Compute cognitive complexity.
- **Expected outcome:** Cognitive complexity >= 1 (the recursive call is counted). Source of truth: implementation plan Task 2 Step 1 "mixed-case self-recursion still counts".
- **Interactions:** Exercises normalized function name tracking in `CognitiveComplexity.functions` and `is_recursive_call()`.

### T24: Fields metric normalizes deduplicated field names for VB.NET

- **Type:** unit
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** VB.NET language module with `normalize_identifier()` override.
- **Actions:** Create a VB.NET source that accesses `Me.Foo` and `Me.foo`. Compute field count.
- **Expected outcome:** Field count = 1 (not 2). Source of truth: implementation plan Task 2 Step 1 "deduplicated field names normalize before insertion into the set".
- **Interactions:** Exercises normalized field name insertion in `fields::count()`.

### T25: Case-sensitive languages are unaffected by normalize_identifier default

- **Type:** regression
- **Disposition:** new
- **Harness:** H2
- **Preconditions:** `normalize_identifier()` default implementation added to `Language` trait.
- **Actions:** Verify that the default `normalize_identifier()` returns the input unchanged for a case-sensitive language (e.g., `Rust::default()`).
- **Expected outcome:** `normalize_identifier("Foo") == "Foo"` and `normalize_identifier("foo") == "foo"` (no case folding). Source of truth: implementation plan "Keep the implementation generic. Do not add a VB.NET branch inside the metrics executors."
- **Interactions:** Confirms no behavioral change for all 14 existing languages.

### T26: VB.NET build snapshot Members.vb proves mixed-case Me/member/New behavior

- **Type:** scenario
- **Disposition:** new
- **Harness:** H1
- **Preconditions:** Members.vb fixture file in the VB.NET build snapshot repo, containing classes with mixed-case `Me`/member access and `Sub New` constructors.
- **Actions:** Run `qlty build --no-plugins --print` and inspect the stats entry for `Members.vb`.
- **Expected outcome:** LCOM, fields, and function counts reflect correct case-insensitive grouping. Constructor methods are excluded from LCOM. Source of truth: implementation plan "make Members.vb prove mixed-case Me / member / New behavior through real emitted stats and issues".
- **Interactions:** End-to-end validation of the normalization chain through the real CLI surface.

---

## Coverage summary

### Areas covered

- **Language registration:** T01 (type mapping), T02 (name resolution), T03 (parser instantiation).
- **Grammar and query contract:** T04 (node bucket invariant), T13 (class query), T14 (function query), T15 (constructor names), T16 (call identifiers), T17 (field identifiers), T18 (parameter names), T19 (function name node).
- **Case-insensitive identifier normalization:** T20 (LCOM constructor), T21 (LCOM method groups), T22 (LCOM field groups), T23 (cognitive recursion), T24 (field deduplication), T25 (default no-op for case-sensitive languages).
- **CLI build surface:** T05 (full build snapshot), T26 (Members.vb mixed-case proof).
- **CLI metrics surface:** T06 (metrics JSON with LANGUAGE_VBDOTNET).
- **CLI smells surface:** T07 (Imports duplication filtering).
- **Regression gates for shared registration files:** T08 (C), T09 (C++), T10 (C#), T11 (Swift), T12 (metrics + smells).

### Areas explicitly excluded

- **Plugin-based linting for VB.NET:** The implementation plan explicitly targets the built-in maintainability path only, not plugins. No plugin tests are needed.
- **VB.NET language alias "vb":** The plan explicitly forbids a `vb` alias and `LANGUAGE_VB` enum value. No tests for aliases.
- **README content validation:** README updates are documentation-only. The test plan does not assert on README contents; this is verified by human review at PR time.
- **Performance benchmarks:** The VB.NET addition is structurally identical to C/C# additions. No new performance-critical paths are introduced. The risk does not warrant dedicated performance testing beyond the existing CI timeout gates.

### Risks from exclusions

- If a future change introduces a VB.NET plugin, plugin-interaction tests would need to be added separately.
- README correctness depends on manual review; an incorrect language list would not be caught by automated tests.
