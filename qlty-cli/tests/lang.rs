use crate::helpers::setup_and_run_test_cases;

#[test]
fn rust_tests() {
    setup_and_run_test_cases("tests/lang/rust/**/*.toml");
}

#[test]
fn kotlin_tests() {
    setup_and_run_test_cases("tests/lang/kotlin/**/*.toml");
}

#[test]
fn ruby_tests() {
    setup_and_run_test_cases("tests/lang/ruby/**/*.toml");
}

#[test]
fn javascript_tests() {
    setup_and_run_test_cases("tests/lang/javascript/**/*.toml");
}

#[test]
fn typescript_tests() {
    setup_and_run_test_cases("tests/lang/typescript/**/*.toml");
}

#[test]
fn tsx_tests() {
    setup_and_run_test_cases("tests/lang/tsx/**/*.toml");
}

#[test]
fn php_tests() {
    setup_and_run_test_cases("tests/lang/php/**/*.toml");
}

#[test]
fn java_tests() {
    setup_and_run_test_cases("tests/lang/java/**/*.toml");
}

#[test]
fn python_tests() {
    setup_and_run_test_cases("tests/lang/python/**/*.toml");
}

#[test]
fn go_tests() {
    setup_and_run_test_cases("tests/lang/go/**/*.toml");
}

#[test]
fn csharp_tests() {
    setup_and_run_test_cases("tests/lang/csharp/**/*.toml");
}

#[test]
fn swift_tests() {
    setup_and_run_test_cases("tests/lang/swift/**/*.toml");
}
