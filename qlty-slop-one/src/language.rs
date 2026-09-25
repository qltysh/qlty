//! Languages Qlty can measure, with the display names slopdetect sends to Jev.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::OnceLock;

use globset::{Glob, GlobSet, GlobSetBuilder};
use qlty_analysis::code::language_detector::get_language_from_shebang;
use qlty_config::config::Builder;
use serde::Serialize;

use crate::error::Error;

/// A language Qlty's structure and duplication analysis supports.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Language {
    C,
    Cpp,
    CSharp,
    Elixir,
    Go,
    Java,
    JavaScript,
    Kotlin,
    Php,
    Python,
    Ruby,
    Rust,
    Scala,
    Swift,
    Tsx,
    TypeScript,
    VbNet,
}

impl Language {
    /// Qlty's internal language name, as used in `default.toml` and `File::language_name`.
    pub fn qlty_name(self) -> &'static str {
        match self {
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CSharp => "csharp",
            Self::Elixir => "elixir",
            Self::Go => "go",
            Self::Java => "java",
            Self::JavaScript => "javascript",
            Self::Kotlin => "kotlin",
            Self::Php => "php",
            Self::Python => "python",
            Self::Ruby => "ruby",
            Self::Rust => "rust",
            Self::Scala => "scala",
            Self::Swift => "swift",
            Self::Tsx => "tsx",
            Self::TypeScript => "typescript",
            Self::VbNet => "vbnet",
        }
    }

    pub fn from_qlty_name(name: &str) -> Option<Self> {
        ALL.into_iter()
            .find(|language| language.qlty_name() == name)
    }

    /// The language Qlty assigns to a file named `Input{suffix}` with these
    /// contents: the first `[language.*].globs` match from `default.toml`, or,
    /// when no glob matches, the interpreter named on a shebang line.
    pub fn detect(suffix: &str, text: &str) -> Result<Self, Error> {
        let file_name = format!("Input{suffix}");
        let detector = detector();

        if let Some(language) = detector.by_glob(&file_name) {
            return Ok(language);
        }

        detector.by_shebang(text).ok_or(Error::UnknownLanguage)
    }

    /// The display name slopdetect derives from Qlty's `LANGUAGE_*` enum name:
    /// a small lookup table, else Python's `str.title()` of the suffix. The
    /// table's keys never matched several enum names, so those fall through
    /// to `title()`. This string is sent to Jev and is part of the cached
    /// request identity, so it stays exactly as slopdetect produced it.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cpp => "C_Plus_Plus",
            Self::CSharp => "C_Sharp",
            Self::Elixir => "Elixir",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::JavaScript => "JavaScript",
            Self::Kotlin => "Kotlin",
            Self::Php => "PHP",
            Self::Python => "Python",
            Self::Ruby => "Ruby",
            Self::Rust => "Rust",
            Self::Scala => "Scala",
            Self::Swift => "Swift",
            Self::Tsx => "Tsx",
            Self::TypeScript => "TypeScript",
            Self::VbNet => "Vbdotnet",
        }
    }
}

impl Serialize for Language {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.display_name())
    }
}

struct Detector {
    globs: Vec<(Language, GlobSet)>,
    interpreters: HashMap<String, Vec<String>>,
}

impl Detector {
    fn by_glob(&self, file_name: &str) -> Option<Language> {
        self.globs
            .iter()
            .find(|(_, glob_set)| glob_set.is_match(file_name))
            .map(|(language, _)| *language)
    }

    fn by_shebang(&self, text: &str) -> Option<Language> {
        let name = get_language_from_shebang(Cursor::new(text), &self.interpreters).ok()?;
        Language::from_qlty_name(&name)
    }
}

fn detector() -> &'static Detector {
    static DETECTOR: OnceLock<Detector> = OnceLock::new();
    DETECTOR.get_or_init(|| {
        let config = Builder::default_config().expect("the bundled default.toml should be valid");
        let mut globs = Vec::new();
        let mut interpreters = HashMap::new();

        for language in ALL {
            let Some(settings) = config.language.get(language.qlty_name()) else {
                continue;
            };
            globs.push((language, glob_set(&settings.globs)));
            if !settings.interpreters.is_empty() {
                interpreters.insert(
                    language.qlty_name().to_string(),
                    settings.interpreters.clone(),
                );
            }
        }

        Detector {
            globs,
            interpreters,
        }
    })
}

fn glob_set(globs: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        builder.add(Glob::new(glob).expect("the bundled default.toml globs should be valid"));
    }
    builder
        .build()
        .expect("the bundled default.toml globs should build")
}

pub const ALL: [Language; 17] = [
    Language::C,
    Language::Cpp,
    Language::CSharp,
    Language::Elixir,
    Language::Go,
    Language::Java,
    Language::JavaScript,
    Language::Kotlin,
    Language::Php,
    Language::Python,
    Language::Ruby,
    Language::Rust,
    Language::Scala,
    Language::Swift,
    Language::Tsx,
    Language::TypeScript,
    Language::VbNet,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_by_extension() {
        assert_eq!(
            Language::detect(".py", "x = 1\n").unwrap(),
            Language::Python
        );
    }

    #[test]
    fn uses_only_the_last_extension() {
        assert_eq!(Language::detect(".ts", "").unwrap(), Language::TypeScript);
    }

    #[test]
    fn rejects_unknown_extension() {
        let error = Language::detect(".unsupported", "meaningless input").unwrap_err();
        assert!(matches!(error, Error::UnknownLanguage));
    }

    #[test]
    fn detects_python_shebang_without_extension() {
        let text = "#!/usr/bin/env python3\nprint(1)\n";
        assert_eq!(Language::detect("", text).unwrap(), Language::Python);
    }

    #[test]
    fn detects_versioned_interpreter_shebang() {
        assert_eq!(
            Language::detect("", "#!/usr/bin/ruby2.7\n").unwrap(),
            Language::Ruby
        );
    }

    #[test]
    fn extension_wins_over_shebang() {
        let text = "#!/usr/bin/env node\nx = 1\n";
        assert_eq!(Language::detect(".py", text).unwrap(), Language::Python);
    }

    #[test]
    fn rejects_unknown_interpreter() {
        let error = Language::detect("", "#!/usr/bin/env haskell\n").unwrap_err();
        assert!(matches!(error, Error::UnknownLanguage));
    }

    #[test]
    fn rejects_no_extension_and_no_shebang() {
        assert!(Language::detect("", "plain text\n").is_err());
    }

    #[test]
    fn display_names_match_slopdetect() {
        assert_eq!(Language::Cpp.display_name(), "C_Plus_Plus");
        assert_eq!(Language::CSharp.display_name(), "C_Sharp");
        assert_eq!(Language::VbNet.display_name(), "Vbdotnet");
        assert_eq!(Language::JavaScript.display_name(), "JavaScript");
    }

    #[test]
    fn serializes_as_display_name() {
        assert_eq!(serde_json::to_string(&Language::Php).unwrap(), "\"PHP\"");
    }
}
