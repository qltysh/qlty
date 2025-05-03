use std::fmt;
use toml::map::Map;
use toml::Value;

// This file is basically a fork of serde-toml-merge crate:
// https://github.com/jdrouet/serde-toml-merge
// MIT License:
// https://github.com/jdrouet/serde-toml-merge/blob/843aa83780c39175865254f3b2548fd4880d718e/Cargo.toml#L8

#[derive(Debug, PartialEq)]
pub struct Error {
    pub path: String,
    pub expected: &'static str,
    pub existing: &'static str,
}

impl Error {
    pub fn new(path: String, expected: &'static str, existing: &'static str) -> Self {
        Self {
            path,
            expected,
            existing,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Incompatible types at path \"{}\", expected \"{}\" received \"{}\".",
            self.path, self.expected, self.existing
        )
    }
}

#[derive(Debug)]
pub struct TomlMerge {}

impl TomlMerge {
    fn merge_into_table_inner(
        value: &mut Map<String, Value>,
        other: Map<String, Value>,
        path: &str,
    ) -> Result<(), Error> {
        for (name, inner) in other {
            if let Some(existing) = value.remove(&name) {
                let inner_path = format!("{path}.{name}");
                value.insert(name, Self::merge_inner(existing, inner, &inner_path)?);
            } else {
                value.insert(name, inner);
            }
        }
        Ok(())
    }

    fn merge_inner(value: Value, other: Value, path: &str) -> Result<Value, Error> {
        match (value, other) {
            (Value::String(_), Value::String(inner)) => Ok(Value::String(inner)),
            (Value::Integer(_), Value::Integer(inner)) => Ok(Value::Integer(inner)),
            (Value::Float(_), Value::Float(inner)) => Ok(Value::Float(inner)),
            (Value::Boolean(_), Value::Boolean(inner)) => Ok(Value::Boolean(inner)),
            (Value::Datetime(_), Value::Datetime(inner)) => Ok(Value::Datetime(inner)),
            (Value::Array(mut existing), Value::Array(inner)) => {
                if existing.iter().all(|v| v.is_table()) && inner.iter().all(|v| v.is_table()) {
                    existing.extend(inner);
                    Ok(Value::Array(existing))
                } else {
                    let extender = Value::String("...".to_string());
                    if inner.contains(&extender) {
                        existing.extend(inner.into_iter().filter(|v| v.as_str().unwrap() != "..."));
                        Ok(Value::Array(existing))
                    } else {
                        Ok(Value::Array(inner))
                    }
                }
            }
            (Value::Table(mut existing), Value::Table(inner)) => {
                Self::merge_into_table_inner(&mut existing, inner, path)?;
                Ok(Value::Table(existing))
            }
            (v, o) => Err(Error::new(path.to_owned(), v.type_str(), o.type_str())),
        }
    }

    /// Merges two toml values into a single one.
    pub fn merge(value: Value, other: Value) -> Result<Value, Error> {
        Self::merge_inner(value, other, "$")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! should_fail {
        ($first: expr, $second: expr) => {
            should_fail!($first, $second,)
        };
        ($first: expr, $second: expr,) => {{
            let first = $first.parse::<Value>().unwrap();
            let second = $second.parse::<Value>().unwrap();
            TomlMerge::merge(first, second).unwrap_err()
        }};
    }

    macro_rules! should_match {
        ($first: expr, $second: expr, $result: expr) => {
            should_match!($first, $second, $result,)
        };
        ($first: expr, $second: expr, $result: expr,) => {
            let first = $first.parse::<Value>().unwrap();
            let second = $second.parse::<Value>().unwrap();
            let result = $result.parse::<Value>().unwrap();
            assert_eq!(TomlMerge::merge(first, second).unwrap(), result);
        };
    }

    #[test]
    fn with_basic() {
        should_match!(
            r#"
        string = "foo"
        integer = 42
        float = 42.24
        boolean = true
        keep_me = true
        "#,
            r#"
        string = "bar"
        integer = 43
        float = 24.42
        boolean = false
        missing = true
        "#,
            r#"
        string = "bar"
        integer = 43
        float = 24.42
        boolean = false
        keep_me = true
        missing = true
        "#,
        );
    }

    #[test]
    fn replace_array() {
        should_match!(
            r#"foo = ["a", "b"]"#,
            r#"foo = ["c", "d"]"#,
            r#"foo = ["c", "d"]"#,
        );
    }

    #[test]
    fn extend_array() {
        should_match!(
            r#"foo = ["a", "b"]"#,
            r#"foo = ["...", "c", "d"]"#,
            r#"foo = ["a", "b", "c", "d"]"#,
        );
    }

    #[test]
    fn with_table() {
        should_match!(
            r#"
            [foo]
            bar = "baz"
        "#,
            r#"
            [foo]
            hello = "world"
        "#,
            r#"
            [foo]
            bar = "baz"
            hello = "world"
        "#,
        );
    }

    #[test]
    fn invalid_kinds() {
        assert_eq!(
            should_fail!("foo = true", "foo = 42"),
            Error::new("$.foo".to_owned(), "boolean", "integer")
        );
        assert_eq!(
            should_fail!("foo = \"true\"", "foo = 42.5"),
            Error::new("$.foo".to_owned(), "string", "float")
        );
    }

    #[test]
    fn merging_array_of_tables() {
        let first = r#"
        [[plugin]]
        name = "ripgrep"
        "#;
        let second = r#"
        [[plugin]]
        name = "gitleaks"
        "#;
        let result = r#"
        [[plugin]]
        name = "ripgrep"

        [[plugin]]
        name = "gitleaks"
        "#;

        should_match!(first, second, result);
    }
}
