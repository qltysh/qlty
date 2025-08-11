// From: https://github.com/monkslc/hyperpolyglot

use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, fs::File as FS_File, io::BufReader, path::Path};

pub fn detect(
    path: &Path,
    interpreters: &HashMap<String, Vec<String>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let file = FS_File::open(path)?;
    let mut reader = BufReader::new(file);
    let language = get_language_from_shebang(&mut reader, interpreters)?;

    if crate::lang::from_str(&language).is_some() {
        return Ok(language);
    }

    Ok("".to_string())
}

pub fn get_language_from_shebang<R: std::io::BufRead>(
    reader: R,
    interpreters: &HashMap<String, Vec<String>>,
) -> Result<String, std::io::Error> {
    let mut lines = reader.lines();
    let shebang_line = match lines.next() {
        Some(line) => line,
        None => return Ok("".to_string()),
    }?;
    let mut extra_content = String::new();

    if !shebang_line.starts_with("#!") {
        return Ok("".to_string());
    }

    let language = shebang_line
        .split('/')
        .next_back()
        .and_then(|interpreter_line| {
            let mut splits = interpreter_line.split_whitespace();
            match splits.next() {
                // #!/usr/bin/env python
                Some("env") => splits.next(),
                // #!/usr/bin/sh [exec scala "$0" "$@"]
                Some("sh") => {
                    let lines: Vec<String> = lines.take(4).filter_map(|line| line.ok()).collect();
                    extra_content = lines.join("\n");
                    lazy_static! {
                        static ref SHEBANG_HACK_RE: Regex =
                            Regex::new(r"exec (\w+).+\$0.+\$@").unwrap();
                    }
                    let interpreter = SHEBANG_HACK_RE
                        .captures(&extra_content[..])
                        .and_then(|captures| captures.get(1))
                        .map(|interpreter| interpreter.as_str())
                        .unwrap_or("sh");
                    Some(interpreter)
                }
                // #!/usr/bin/python
                Some(interpreter) => Some(interpreter),
                // #!
                None => None,
            }
        })
        .and_then(|interpreter| {
            // #!/usr/bin/python2.6.3 -> #!/usr/bin/python2
            lazy_static! {
                static ref RE: Regex = Regex::new(r"[0-9]\.[0-9]").unwrap();
            }
            let parsed_interpreter = RE.split(interpreter).next().unwrap();
            interpreters
                .iter()
                .find(|(_, interpreters)| interpreters.contains(&parsed_interpreter.to_string()))
                .map(|(lang, _)| lang.to_string())
        });

    match language {
        Some(language) => Ok(language),
        None => Ok("".to_string()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use qlty_config::config::Builder;
    use std::{
        collections::HashMap,
        env,
        fs::{self, File},
        io::{Cursor, Write},
        path::PathBuf,
    };

    #[test]
    fn test_detect_get_language_by_shebang() {
        let mut temp_file_path = PathBuf::from(env::temp_dir());
        temp_file_path.push("temp_shebang_file");
        {
            let mut temp_file =
                File::create(&temp_file_path).expect("Failed to create temporary file");
            writeln!(temp_file, "#!/usr/bin/env node").expect("Failed to write to temporary file");
        }

        let result = detect(&temp_file_path, &mock_interpreters());

        fs::remove_file(&temp_file_path).expect("Failed to delete temporary file");

        assert_eq!(result.unwrap().as_str(), "javascript");
    }

    #[test]
    fn test_detect_get_unsupported_language() {
        let mut temp_file_path = PathBuf::from(env::temp_dir());
        temp_file_path.push("temp_unsupported_language_file");
        {
            let mut temp_file =
                File::create(&temp_file_path).expect("Failed to create temporary file");
            writeln!(temp_file, "#!/usr/bin/env haskell")
                .expect("Failed to write to temporary file");
        }

        let result = detect(&temp_file_path, &mock_interpreters());

        fs::remove_file(&temp_file_path).expect("Failed to delete temporary file");

        assert_eq!(result.unwrap().as_str(), "");
    }

    #[test]
    fn test_shebang_get_language() {
        assert_eq!(
            get_language_from_shebang(Cursor::new("#!/usr/bin/python"), &mock_interpreters())
                .unwrap(),
            "python"
        );
    }
    #[test]
    fn test_shebang_get_language_env() {
        assert_eq!(
            get_language_from_shebang(Cursor::new("#!/usr/bin/env node"), &mock_interpreters())
                .unwrap(),
            "javascript"
        );
    }

    #[test]
    fn test_shebang_get_language_with_minor_version() {
        assert_eq!(
            get_language_from_shebang(Cursor::new("#!/usr/bin/python2.6"), &mock_interpreters())
                .unwrap(),
            "python"
        );
    }

    #[test]
    fn test_shebang_empty_cases() {
        assert_eq!(
            get_language_from_shebang(Cursor::new("#!/usr/bin/env"), &mock_interpreters()).unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new("#!/usr/bin/parrot"), &mock_interpreters())
                .unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new("#!"), &mock_interpreters()).unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new(""), &mock_interpreters()).unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new("aslkdfjas;ldk"), &mock_interpreters()).unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new(" #!/usr/bin/python"), &mock_interpreters())
                .unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new(" #!/usr/bin/ "), &mock_interpreters()).unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new(" #!/usr/bin"), &mock_interpreters()).unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new(" #!/usr/bin"), &mock_interpreters()).unwrap(),
            ""
        );
        assert_eq!(
            get_language_from_shebang(Cursor::new(""), &mock_interpreters()).unwrap(),
            ""
        );
    }

    #[test]
    fn test_shebang_hack() {
        let content = Cursor::new(
            r#"#!/bin/sh
               exec python "$0" "$@"
               !#
            "#,
        );

        assert_eq!(
            get_language_from_shebang(content, &mock_interpreters()).unwrap(),
            "python"
        );
    }

    fn mock_interpreters() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();

        Builder::default_config()
            .unwrap()
            .file_types
            .iter()
            .for_each(|(lang, file_type)| {
                let _ = &file_type.interpreters.iter().for_each(|interpreter| {
                    map.entry(lang.clone())
                        .or_insert_with(Vec::new)
                        .push(interpreter.clone());
                });
            });
        map
    }
}
