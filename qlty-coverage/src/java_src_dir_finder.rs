use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use qlty_analysis::utils::fs::path_to_string;
use std::path::PathBuf;

const EXCLUSION_WORDS: &[&str] = &["test", "tests", "testing", "tester", "build"];
const EXACT_EXCLUSION_COMPONENTS: &[&str] = &["node_modules"];

#[derive(Debug, Clone)]
pub enum ExclusionStrategy {
    UserDefined(Vec<String>),
    DefaultHeuristics,
}

#[derive(Debug)]
pub struct JavaSrcDirFinder {
    root: PathBuf,
    exclusion_strategy: ExclusionStrategy,
}

impl JavaSrcDirFinder {
    pub fn new(root: PathBuf, exclusion_strategy: ExclusionStrategy) -> Self {
        Self {
            root,
            exclusion_strategy,
        }
    }

    pub fn find(&self) -> Result<Vec<PathBuf>> {
        let exclude_globset = self.build_exclude_globset()?;

        let mut found_dirs = Vec::new();

        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let relative_path = path_to_string(path.strip_prefix(&self.root).unwrap_or(path));

            if Self::is_java_src_dir(&relative_path) {
                if self.should_exclude(&relative_path, &exclude_globset) {
                    tracing::debug!("Excluding Java src dir: {}", path.display());
                    continue;
                }

                tracing::debug!("Found Java src dir: {}", relative_path);
                found_dirs.push(PathBuf::from(&relative_path));
            }
        }

        found_dirs.sort();
        Ok(found_dirs)
    }

    fn is_java_src_dir(relative_path: &str) -> bool {
        let components: Vec<&str> = relative_path.split('/').collect();
        let len = components.len();

        if len < 3 {
            return false;
        }

        let third_last = components[len - 3];
        let last = components[len - 1];

        third_last == "src" && (last == "java" || last == "kotlin")
    }

    fn build_exclude_globset(&self) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        if let ExclusionStrategy::UserDefined(patterns) = &self.exclusion_strategy {
            for pattern in patterns {
                builder.add(Glob::new(pattern)?);
            }
        }

        Ok(builder.build()?)
    }

    fn should_exclude(&self, relative_path: &str, exclude_globset: &GlobSet) -> bool {
        match &self.exclusion_strategy {
            ExclusionStrategy::UserDefined(_) => exclude_globset.is_match(relative_path),
            ExclusionStrategy::DefaultHeuristics => self.matches_default_exclusions(relative_path),
        }
    }

    fn matches_default_exclusions(&self, relative_path: &str) -> bool {
        let components: Vec<&str> = relative_path.split('/').collect();

        for component in &components {
            let lower = component.to_lowercase();

            if EXACT_EXCLUSION_COMPONENTS.iter().any(|exc| lower == *exc) {
                return true;
            }

            let words = Self::split_into_words(component);
            if words
                .iter()
                .any(|word| EXCLUSION_WORDS.contains(&word.as_str()))
            {
                return true;
            }
        }

        false
    }

    fn split_into_words(component: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut current_word = String::new();

        for c in component.chars() {
            if !c.is_alphanumeric() {
                if !current_word.is_empty() {
                    words.push(std::mem::take(&mut current_word));
                }
            } else if c.is_uppercase() && !current_word.is_empty() {
                words.push(std::mem::take(&mut current_word));
                current_word.push(c.to_ascii_lowercase());
            } else {
                current_word.push(c.to_ascii_lowercase());
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_dir(base: &Path, relative: &str) {
        let path = base.join(relative);
        fs::create_dir_all(&path).unwrap();
    }

    #[test]
    fn split_into_words_handles_simple() {
        assert_eq!(JavaSrcDirFinder::split_into_words("test"), vec!["test"]);
        assert_eq!(
            JavaSrcDirFinder::split_into_words("contest"),
            vec!["contest"]
        );
    }

    #[test]
    fn split_into_words_handles_dashes() {
        assert_eq!(
            JavaSrcDirFinder::split_into_words("my-test"),
            vec!["my", "test"]
        );
        assert_eq!(
            JavaSrcDirFinder::split_into_words("test-thing"),
            vec!["test", "thing"]
        );
    }

    #[test]
    fn split_into_words_handles_underscores() {
        assert_eq!(
            JavaSrcDirFinder::split_into_words("my_test"),
            vec!["my", "test"]
        );
        assert_eq!(
            JavaSrcDirFinder::split_into_words("test_thing"),
            vec!["test", "thing"]
        );
    }

    #[test]
    fn split_into_words_handles_camel_case() {
        assert_eq!(
            JavaSrcDirFinder::split_into_words("testSomething"),
            vec!["test", "something"]
        );
        assert_eq!(
            JavaSrcDirFinder::split_into_words("somethingTest"),
            vec!["something", "test"]
        );
        assert_eq!(
            JavaSrcDirFinder::split_into_words("MyTest"),
            vec!["my", "test"]
        );
        assert_eq!(
            JavaSrcDirFinder::split_into_words("androidTest"),
            vec!["android", "test"]
        );
    }

    #[test]
    fn split_into_words_preserves_non_test_words() {
        assert_eq!(
            JavaSrcDirFinder::split_into_words("contest"),
            vec!["contest"]
        );
        assert_eq!(
            JavaSrcDirFinder::split_into_words("protest"),
            vec!["protest"]
        );
        assert_eq!(
            JavaSrcDirFinder::split_into_words("testify"),
            vec!["testify"]
        );
    }

    #[test]
    fn matches_default_exclusions_excludes_test_variants() {
        let finder = JavaSrcDirFinder::new(PathBuf::new(), ExclusionStrategy::DefaultHeuristics);

        assert!(finder.matches_default_exclusions("test/src/main/java"));
        assert!(finder.matches_default_exclusions("my-test/src/main/java"));
        assert!(finder.matches_default_exclusions("test-thing/src/main/java"));
        assert!(finder.matches_default_exclusions("testSomething/src/main/java"));
        assert!(finder.matches_default_exclusions("somethingTest/src/main/java"));
        assert!(finder.matches_default_exclusions("src/testing/java"));
        assert!(finder.matches_default_exclusions("src/tester/java"));
    }

    #[test]
    fn matches_default_exclusions_allows_non_test_words() {
        let finder = JavaSrcDirFinder::new(PathBuf::new(), ExclusionStrategy::DefaultHeuristics);

        assert!(!finder.matches_default_exclusions("contest/src/main/java"));
        assert!(!finder.matches_default_exclusions("protest/src/main/java"));
        assert!(!finder.matches_default_exclusions("testify/src/main/java"));
        assert!(!finder.matches_default_exclusions("attest/src/main/java"));
        assert!(!finder.matches_default_exclusions("detest/src/main/java"));
    }

    #[test]
    fn matches_default_exclusions_handles_node_modules() {
        let finder = JavaSrcDirFinder::new(PathBuf::new(), ExclusionStrategy::DefaultHeuristics);

        assert!(finder.matches_default_exclusions("node_modules/pkg/src/main/java"));
        assert!(!finder.matches_default_exclusions("my_modules/src/main/java"));
    }

    #[test]
    fn matches_default_exclusions_handles_build() {
        let finder = JavaSrcDirFinder::new(PathBuf::new(), ExclusionStrategy::DefaultHeuristics);

        assert!(finder.matches_default_exclusions("build/src/main/java"));
        assert!(finder.matches_default_exclusions("build-output/src/main/java"));
        assert!(!finder.matches_default_exclusions("rebuild/src/main/java"));
        assert!(!finder.matches_default_exclusions("prebuild/src/main/java"));
    }

    #[test]
    fn is_java_src_dir_matches_maven_java() {
        assert!(JavaSrcDirFinder::is_java_src_dir("src/main/java"));
        assert!(JavaSrcDirFinder::is_java_src_dir("app/src/main/java"));
        assert!(JavaSrcDirFinder::is_java_src_dir(
            "project/module/src/main/java"
        ));
    }

    #[test]
    fn is_java_src_dir_matches_maven_kotlin() {
        assert!(JavaSrcDirFinder::is_java_src_dir("src/main/kotlin"));
        assert!(JavaSrcDirFinder::is_java_src_dir("app/src/main/kotlin"));
    }

    #[test]
    fn is_java_src_dir_matches_gradle_variants() {
        assert!(JavaSrcDirFinder::is_java_src_dir("src/debug/java"));
        assert!(JavaSrcDirFinder::is_java_src_dir("src/release/kotlin"));
        assert!(JavaSrcDirFinder::is_java_src_dir("app/src/production/java"));
        assert!(JavaSrcDirFinder::is_java_src_dir(
            "lib/src/androidTest/kotlin"
        ));
    }

    #[test]
    fn is_java_src_dir_rejects_too_few_components() {
        assert!(!JavaSrcDirFinder::is_java_src_dir("java"));
        assert!(!JavaSrcDirFinder::is_java_src_dir("main/java"));
        assert!(!JavaSrcDirFinder::is_java_src_dir(""));
    }

    #[test]
    fn is_java_src_dir_rejects_wrong_structure() {
        assert!(!JavaSrcDirFinder::is_java_src_dir("app/main/java"));
        assert!(!JavaSrcDirFinder::is_java_src_dir("src/java"));
        assert!(!JavaSrcDirFinder::is_java_src_dir("source/main/java"));
    }

    #[test]
    fn is_java_src_dir_rejects_subdirectories_of_source_roots() {
        assert!(!JavaSrcDirFinder::is_java_src_dir("src/main/java/com"));
        assert!(!JavaSrcDirFinder::is_java_src_dir(
            "src/main/java/com/example"
        ));
        assert!(!JavaSrcDirFinder::is_java_src_dir(
            "src/main/java/com/example/kotlin"
        ));
        assert!(!JavaSrcDirFinder::is_java_src_dir(
            "app/src/main/java/com/gusto/foundation/async/kotlin"
        ));
    }

    #[test]
    fn is_java_src_dir_rejects_non_java_kotlin_endings() {
        assert!(!JavaSrcDirFinder::is_java_src_dir("src/main/scala"));
        assert!(!JavaSrcDirFinder::is_java_src_dir("src/main/groovy"));
        assert!(!JavaSrcDirFinder::is_java_src_dir("src/main/resources"));
    }

    #[test]
    fn finds_maven_structure() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/main/java/com/example");
        create_dir(root, "src/main/kotlin/com/example");

        let finder =
            JavaSrcDirFinder::new(root.to_path_buf(), ExclusionStrategy::DefaultHeuristics);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 2);
        assert!(dirs
            .iter()
            .any(|p| p.ends_with("src/main/java") || p.ends_with("src\\main\\java")));
        assert!(dirs
            .iter()
            .any(|p| p.ends_with("src/main/kotlin") || p.ends_with("src\\main\\kotlin")));
    }

    #[test]
    fn finds_gradle_structure() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "app/src/main/java/com/example");
        create_dir(root, "lib/src/debug/java/com/example");
        create_dir(root, "lib/src/release/kotlin/com/example");

        let finder =
            JavaSrcDirFinder::new(root.to_path_buf(), ExclusionStrategy::DefaultHeuristics);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 3);
    }

    #[test]
    fn excludes_node_modules_with_default_heuristics() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/main/java/com/example");
        create_dir(root, "node_modules/some-package/src/main/java");

        let finder =
            JavaSrcDirFinder::new(root.to_path_buf(), ExclusionStrategy::DefaultHeuristics);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 1);
        assert!(dirs[0].ends_with("src/main/java") || dirs[0].ends_with("src\\main\\java"));
    }

    #[test]
    fn excludes_test_directories_with_default_heuristics() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/main/java/com/example");
        create_dir(root, "src/test/java/com/example");
        create_dir(root, "test/src/main/java/com/example");
        create_dir(root, "build/src/main/java/com/example");

        let finder =
            JavaSrcDirFinder::new(root.to_path_buf(), ExclusionStrategy::DefaultHeuristics);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 1);
        assert!(dirs[0].ends_with("src/main/java") || dirs[0].ends_with("src\\main\\java"));
    }

    #[test]
    fn uses_user_defined_exclusions() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "app/src/main/java/com/example");
        create_dir(root, "legacy/src/main/java/com/example");

        let finder = JavaSrcDirFinder::new(
            root.to_path_buf(),
            ExclusionStrategy::UserDefined(vec!["legacy/**".to_string()]),
        );
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 1);
        assert!(
            dirs[0].to_string_lossy().contains("app/src/main/java")
                || dirs[0].to_string_lossy().contains("app\\src\\main\\java")
        );
    }

    #[test]
    fn returns_empty_when_no_sources() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/lib");
        create_dir(root, "app/code");

        let finder =
            JavaSrcDirFinder::new(root.to_path_buf(), ExclusionStrategy::DefaultHeuristics);
        let dirs = finder.find().unwrap();

        assert!(dirs.is_empty());
    }

    #[test]
    fn handles_nested_projects() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "project-a/src/main/java/com/a");
        create_dir(root, "project-b/src/main/kotlin/com/b");
        create_dir(root, "project-c/module/src/main/java/com/c");

        let finder =
            JavaSrcDirFinder::new(root.to_path_buf(), ExclusionStrategy::DefaultHeuristics);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 3);
    }

    #[test]
    fn does_not_match_subdirectories_of_source_roots() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "app/src/main/java/com/example/kotlin");

        let finder =
            JavaSrcDirFinder::new(root.to_path_buf(), ExclusionStrategy::DefaultHeuristics);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 1);
        assert!(dirs[0].ends_with("src/main/java") || dirs[0].ends_with("src\\main\\java"));
    }

    #[test]
    fn allows_directories_with_test_substring() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir(root, "src/main/java/com/example");
        create_dir(root, "contest/src/main/java/com/example");

        let finder =
            JavaSrcDirFinder::new(root.to_path_buf(), ExclusionStrategy::DefaultHeuristics);
        let dirs = finder.find().unwrap();

        assert_eq!(dirs.len(), 2);
    }
}
