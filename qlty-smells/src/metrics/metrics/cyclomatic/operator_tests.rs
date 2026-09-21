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

#[test]
fn comparisons_in_conditions_do_not_count_twice() {
    assert_complexity(
        "typescript",
        r#"
            function isReady(count: number, limit: number, enabled: boolean) {
                if (enabled && count + 1 < limit) {
                    return true;
                }
                return false;
            }
        "#,
        3,
    );
}

#[test]
fn short_circuit_operators_inside_comparisons_still_count() {
    assert_complexity(
        "typescript",
        r#"
            function agrees(enabled: boolean, ready: boolean, expected: boolean) {
                return (enabled && ready) === expected;
            }
        "#,
        2,
    );
}

#[test]
fn comments_and_operator_strings_do_not_add_paths() {
    assert_complexity(
        "javascript",
        r#"
            function choose(primary, fallback) {
                const label = "&&" + "||";
                return primary /* prefer the primary value */ || fallback + label;
            }
        "#,
        2,
    );
}

#[test]
fn javascript_and_typescript_null_coalescing_adds_paths() {
    for language in ["javascript", "typescript", "tsx"] {
        assert_complexity(
            language,
            r#"
                function choose(primary, secondary, fallback) {
                    return primary ?? secondary ?? fallback;
                }
            "#,
            3,
        );
    }
}

#[test]
fn csharp_null_coalescing_adds_paths() {
    assert_complexity(
        "csharp",
        r#"
            class Settings {
                static string Choose(string primary, string secondary, string fallback) {
                    return primary ?? secondary ?? fallback;
                }
            }
        "#,
        3,
    );
}

#[test]
fn php_null_coalescing_adds_paths() {
    assert_complexity(
        "php",
        r#"
            <?php
            function choose($primary, $secondary, $fallback) {
                return $primary ?? $secondary ?? $fallback;
            }
        "#,
        3,
    );
}

#[test]
fn cpp_alternative_operators_only_count_short_circuiting() {
    assert_complexity(
        "cpp",
        r#"
            bool is_ready(int flags, int mask, bool enabled, bool cached) {
                int selected = (flags bitand mask) bitor (flags xor mask);
                return enabled and (selected not_eq 0 or cached);
            }
        "#,
        3,
    );
}

#[test]
fn php_keyword_operators_only_count_short_circuiting() {
    assert_complexity(
        "php",
        r#"
            <?php
            function isReady(bool $enabled, bool $connected, bool $cached): bool {
                return $enabled AnD ($connected oR ($cached xor $enabled));
            }
        "#,
        3,
    );
}

#[test]
fn ruby_keyword_operators_add_paths() {
    assert_complexity(
        "ruby",
        r#"
            def is_ready(enabled, connected, cached)
              enabled and (connected or cached)
            end
        "#,
        3,
    );
}

#[test]
fn kotlin_chained_operators_add_paths() {
    assert_complexity(
        "kotlin",
        r#"
            fun isReady(enabled: Boolean, connected: Boolean, cached: Boolean): Boolean {
                return enabled /* gate access */ && connected && (cached || connected)
            }
        "#,
        4,
    );
}

#[test]
fn vbnet_mixed_case_short_circuit_operators_add_paths() {
    assert_complexity(
        "vbnet",
        r#"
            Module Settings
                Function IsReady(enabled As Boolean, connected As Boolean, cached As Boolean) As Boolean
                    Return enabled aNdAlSo (connected OrELse cached)
                End Function
            End Module
        "#,
        3,
    );
}

#[test]
fn vbnet_eager_boolean_operators_do_not_add_paths() {
    assert_complexity(
        "vbnet",
        r#"
            Module Settings
                Function IsReady(enabled As Boolean, connected As Boolean, cached As Boolean) As Boolean
                    Return enabled And (connected Or (cached Xor enabled))
                End Function
            End Module
        "#,
        1,
    );
}

#[test]
fn vbnet_handles_line_continuations_and_comments() {
    assert_complexity(
        "vbnet",
        r#"
            Module Settings
                Function IsReady(enabled As Boolean, connected As Boolean, cached As Boolean) As Boolean
                    Return enabled _
                        AndAlso (connected OrElse _
                        cached)
                End Function
            End Module
        "#,
        3,
    );

    assert_complexity(
        "vbnet",
        r#"
            Module Settings
                Function Both(enabled As Boolean, connected As Boolean) As Boolean
                    Return enabled And connected ' AndAlso would short-circuit here.
                End Function
            End Module
        "#,
        1,
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
