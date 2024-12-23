use qlty_coverage::parser::Cobertura;
use qlty_coverage::Parser;

#[test]
fn cobertura_results() {
    // Make sure that the <?xml version="1.0"?> tag is always right at the beginning of the string to avoid parsing errors
    let input = include_str!("fixtures/cobertura/sample.xml");

    let parsed_results = Cobertura::new().parse_text(input).unwrap();
    insta::assert_yaml_snapshot!(parsed_results, @r#"
    - path: Main.java
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "3"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "3"
        - "3"
        - "3"
        - "3"
        - "-1"
        - "-1"
        - "-1"
        - "3"
        - "-1"
        - "3"
        - "3"
        - "-1"
        - "3"
        - "3"
        - "3"
    - path: search/BinarySearch.java
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "3"
        - "-1"
        - "-1"
        - "-1"
        - "3"
        - "-1"
        - "12"
        - "-1"
        - "21"
        - "15"
        - "-1"
        - "9"
        - "0"
        - "9"
        - "6"
        - "-1"
        - "3"
        - "9"
        - "-1"
        - "3"
    - path: search/ISortedArraySearch.java
    - path: search/LinearSearch.java
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "3"
        - "2"
        - "-1"
        - "-1"
        - "9"
        - "-1"
        - "9"
        - "3"
        - "6"
        - "-1"
        - "0"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "3"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "5"
    "#);
}
