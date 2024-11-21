use std::path::{Path, MAIN_SEPARATOR_STR};

#[macro_export]
/// Join a path together from a list of string-like arguments.
///
/// # Example
///     let path = join_path_string!("a", "b", "c");
///     assert_eq!(path, "a/b/c"); // on Unix systems
macro_rules! join_path_string {
    ($($x:expr),*) => {
        {
            let mut path = std::path::PathBuf::new();
            $(
                path.push($x);
            )*
            path.to_string_lossy().to_string().replace('\\', "/")
        }
    };
}

pub fn path_to_string<'path>(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .to_string_lossy()
        .to_string()
        .replace('\\', "/")
}

pub fn path_to_native_string(path: impl AsRef<Path>) -> String {
    path_to_string(path)
        .replace(['\\', '/'], MAIN_SEPARATOR_STR)
}

mod test {
    #[test]
    fn test_join_path_string() {
        let path = join_path_string!(
            "a",
            "b".to_string(),
            std::path::PathBuf::from("c"),
            std::path::Path::new("d")
        );
        assert_eq!(path, "a/b/c/d");
    }

    #[test]
    fn test_path_to_string() {
        let path = std::path::PathBuf::from("a\\b\\c");
        assert_eq!(crate::utils::fs::path_to_string(path), "a/b/c");
    }

    #[test]
    fn test_path_to_native_string() {
        let path = join_path_string!("a", "b", "c");
        assert_eq!(
            crate::utils::fs::path_to_native_string(path),
            format!(
                "a{}b{}c",
                std::path::MAIN_SEPARATOR,
                std::path::MAIN_SEPARATOR
            )
        );

        let path = "a/b/c";
        assert_eq!(
            crate::utils::fs::path_to_native_string(path),
            format!(
                "a{}b{}c",
                std::path::MAIN_SEPARATOR,
                std::path::MAIN_SEPARATOR
            )
        );
    }
}
