use crate::formats::Formats;
use anyhow::Result;
use std::path::PathBuf;
use std::str::FromStr;

// Format specification priority:
// 1. The format specified in the path, e.g.: simplecov:./coverage/coverage.json
// 2. The format specified in the command line arguments, through --report-format simplecov
// 3. The format is inferred from the file extension or contents
pub fn extract_path_and_format(
    path: &str,
    base_format: Option<Formats>,
) -> Result<(PathBuf, Formats)> {
    if let Some((potential_format, rest)) = path.split_once(':') {
        match Formats::from_str(potential_format) {
            Ok(format) => Ok((PathBuf::from(rest), format)),
            Err(_) => match base_format {
                Some(format) => Ok((PathBuf::from(path), format)),
                None => Ok((PathBuf::from(path), Formats::try_from(path.as_ref())?)),
            },
        }
    } else if let Some(format) = base_format {
        Ok((PathBuf::from(path), format))
    } else {
        Ok((PathBuf::from(path), Formats::try_from(path.as_ref())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::Formats;

    #[test]
    fn test_format_in_path_explicit() {
        let path = "simplecov:./coverage/coverage.json";
        let result = extract_path_and_format(path, None).unwrap();
        assert_eq!(result.0, PathBuf::from("./coverage/coverage.json"));
        assert_eq!(result.1, Formats::Simplecov);
    }

    #[test]
    fn test_format_in_path_invalid_but_base_format_used() {
        let path = "invalidfmt:./coverage/coverage.json";
        let base = Some(Formats::Lcov);
        let result = extract_path_and_format(path, base).unwrap();
        assert_eq!(result.0, PathBuf::from(path));
        assert_eq!(result.1, Formats::Lcov);
    }

    #[test]
    fn test_format_in_path_invalid_and_no_base_format_fallbacks_to_try_from() {
        // Suppose "invalidfmt:./foo.txt" fails parsing and TryFrom also fails
        let path = "invalidfmt:./foo.txt";
        let result = extract_path_and_format(path, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_not_in_path_but_base_format_provided() {
        let path = "./coverage/lcov.info";
        let base = Some(Formats::Lcov);
        let result = extract_path_and_format(path, base).unwrap();
        assert_eq!(result.0, PathBuf::from(path));
        assert_eq!(result.1, Formats::Lcov);
    }

    #[test]
    fn test_format_not_in_path_or_base_infer_from_path() {
        let path = "./coverage/lcov.info"; // Assuming try_from recognizes this
        let result = extract_path_and_format(path, None).unwrap();
        assert_eq!(result.0, PathBuf::from(path));
        assert_eq!(result.1, Formats::Lcov);
    }

    #[test]
    fn test_windows_path() {
        let path = "D:/a/qlty-action/qlty-action/coverage/coverage/clover.xml";
        let result = extract_path_and_format(path, None).unwrap();
        assert_eq!(result.0, PathBuf::from(path));
        assert_eq!(result.1, Formats::Clover);
    }
}
