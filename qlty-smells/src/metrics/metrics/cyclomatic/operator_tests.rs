use super::count;
use qlty_analysis::code::{File, NodeFilter};

fn assert_complexity(language: &str, source: &str, expected: usize) {
    let file = File::from_string(language, source);
    let tree = file.parse();
    let root = tree.root_node();

    assert!(
        !root.has_error(),
        "Invalid {language} fixture: {}",
        root.to_sexp()
    );
    assert_eq!(
        expected,
        count(&file, &root, &NodeFilter::empty()),
        "Unexpected cyclomatic complexity for {language}"
    );
}

#[test]
fn arithmetic_does_not_add_a_path() {
    assert_complexity(
        "typescript",
        r#"
            function increment(value: number) {
                return value + 1;
            }
        "#,
        1,
    );
}

#[test]
fn bit_shifts_do_not_add_a_path() {
    assert_complexity(
        "typescript",
        r#"
            function shiftByte(value: number) {
                return value << 8;
            }
        "#,
        1,
    );
}

#[test]
fn comparisons_do_not_add_a_path() {
    assert_complexity(
        "typescript",
        r#"
            function isExpected(value: number) {
                return value === 42;
            }
        "#,
        1,
    );
}

#[test]
fn each_short_circuit_operator_adds_a_path() {
    assert_complexity(
        "typescript",
        r#"
            function isReady(enabled: boolean, connected: boolean, cached: boolean) {
                return enabled && (connected || cached);
            }
        "#,
        3,
    );
}

// Only short-circuit conditions add paths in these examples.
// Bitwise operations, addition and comparison do not.
// Include unaffected languages to preserve their existing behavior.

#[test]
fn c_only_counts_short_circuit_operators() {
    assert_complexity(
        "c",
        r#"
            int matches_code(int high, int low, int expected, int enabled) {
                int code = (high << 8) | low;
                return enabled && code + 1 == expected;
            }
        "#,
        2,
    );
}

#[test]
fn cpp_only_counts_short_circuit_operators() {
    assert_complexity(
        "cpp",
        r#"
            bool matches_code(int high, int low, int expected, bool enabled) {
                int code = (high << 8) | low;
                return enabled && code + 1 == expected;
            }
        "#,
        2,
    );
}

#[test]
fn csharp_only_counts_short_circuit_operators() {
    assert_complexity(
        "csharp",
        r#"
            class Header {
                static bool MatchesCode(int high, int low, int expected, bool enabled) {
                    int code = (high << 8) | low;
                    return enabled && code + 1 == expected;
                }
            }
        "#,
        2,
    );
}

#[test]
fn elixir_only_counts_short_circuit_operators() {
    assert_complexity(
        "elixir",
        r#"
            defmodule Header do
              import Bitwise

              def matches_code(high, low, expected, enabled) do
                code = (high <<< 8) ||| low
                enabled and code + 1 == expected
              end
            end
        "#,
        2,
    );
}

#[test]
fn go_only_counts_short_circuit_operators() {
    assert_complexity(
        "go",
        r#"
            package header

            func matchesCode(high, low, expected int, enabled bool) bool {
                code := (high << 8) | low
                return enabled && code + 1 == expected
            }
        "#,
        2,
    );
}

#[test]
fn java_only_counts_short_circuit_operators() {
    assert_complexity(
        "java",
        r#"
            class Header {
                static boolean matchesCode(int high, int low, int expected, boolean enabled) {
                    int code = (high << 8) | low;
                    return enabled && code + 1 == expected;
                }
            }
        "#,
        2,
    );
}

#[test]
fn javascript_only_counts_short_circuit_operators() {
    assert_complexity(
        "javascript",
        r#"
            function matchesCode(high, low, expected, enabled) {
                const code = (high << 8) | low;
                return enabled && code + 1 === expected;
            }
        "#,
        2,
    );
}

#[test]
fn kotlin_only_counts_short_circuit_operators() {
    assert_complexity(
        "kotlin",
        r#"
            fun matchesCode(high: Int, low: Int, expected: Int, enabled: Boolean): Boolean {
                val code = (high shl 8) or low
                return enabled && code + 1 == expected
            }
        "#,
        2,
    );
}

#[test]
fn php_only_counts_short_circuit_operators() {
    assert_complexity(
        "php",
        r#"
            <?php
            function matchesCode(int $high, int $low, int $expected, bool $enabled): bool {
                $code = ($high << 8) | $low;
                return $enabled && $code + 1 === $expected;
            }
        "#,
        2,
    );
}

#[test]
fn python_only_counts_short_circuit_operators() {
    assert_complexity(
        "python",
        r#"
def matches_code(high, low, expected, enabled):
    code = (high << 8) | low
    return enabled and code + 1 == expected
"#,
        2,
    );
}

#[test]
fn ruby_only_counts_short_circuit_operators() {
    assert_complexity(
        "ruby",
        r#"
            def matches_code(high, low, expected, enabled)
              code = (high << 8) | low
              enabled && code + 1 == expected
            end
        "#,
        2,
    );
}

#[test]
fn rust_only_counts_short_circuit_operators() {
    assert_complexity(
        "rust",
        r#"
            fn matches_code(high: u32, low: u32, expected: u32, enabled: bool) -> bool {
                let code = (high << 8) | low;
                enabled && code + 1 == expected
            }
        "#,
        2,
    );
}

#[test]
fn scala_only_counts_short_circuit_operators() {
    assert_complexity(
        "scala",
        r#"
            object Header {
                def matchesCode(high: Int, low: Int, expected: Int, enabled: Boolean): Boolean = {
                    val code = (high << 8) | low
                    enabled && code + 1 == expected
                }
            }
        "#,
        2,
    );
}

#[test]
fn swift_non_branching_operators_keep_one_path() {
    assert_complexity(
        "swift",
        r#"
            func matchesCode(high: Int, low: Int, expected: Int) -> Bool {
                let code = (high << 8) | low
                return code + 1 == expected
            }
        "#,
        1,
    );
}

#[test]
fn typescript_only_counts_short_circuit_operators() {
    assert_complexity(
        "typescript",
        r#"
            function matchesCode(high: number, low: number, expected: number, enabled: boolean) {
                const code = (high << 8) | low;
                return enabled && code + 1 === expected;
            }
        "#,
        2,
    );
}

#[test]
fn tsx_only_counts_short_circuit_operators() {
    assert_complexity(
        "tsx",
        r#"
            function matchesCode(high: number, low: number, expected: number, enabled: boolean) {
                const code = (high << 8) | low;
                return enabled && code + 1 === expected;
            }
        "#,
        2,
    );
}

#[test]
fn vbnet_only_counts_short_circuit_operators() {
    assert_complexity(
        "vbnet",
        r#"
            Module Header
                Function MatchesCode(high As Integer, low As Integer, expected As Integer, enabled As Boolean) As Boolean
                    Dim code As Integer = (high << 8) Or low
                    Return enabled AndAlso code + 1 = expected
                End Function
            End Module
        "#,
        2,
    );
}
