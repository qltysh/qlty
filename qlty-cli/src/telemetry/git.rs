use git2::Repository;
use regex::Regex;
use std::path::PathBuf;

const URL_STRIP_PATTERNS: [&str; 3] = [r"^(https?|git|ssh):\/\/", r"^([^@]+)@", r"\.git$"];

pub fn repository_identifier(repository_path: Option<PathBuf>) -> String {
    match repository_path {
        Some(ref path) => {
            if let Ok(repo) = Repository::init(path) {
                if let Ok(remote) = repo.find_remote("origin") {
                    if let Some(url) = remote.url() {
                        let normalized_url = normalize_git_url(url);
                        format!("{:x}", md5::compute(normalized_url))
                    } else {
                        "Unknown(URL)".to_owned()
                    }
                } else {
                    "Error(Remote)".to_owned()
                }
            } else {
                "Error(Repo)".to_owned()
            }
        }
        None => "Unknown".to_owned(),
    }
}

pub fn normalize_git_url(url: &str) -> String {
    let mut normalized_url = url.to_owned();

    for pattern in URL_STRIP_PATTERNS {
        let re = Regex::new(pattern).unwrap();
        normalized_url = re.replace(&normalized_url, "").to_string();
    }

    normalized_url.replace(':', "/")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn normalized_equality() {
        assert_eq!(
            normalize_git_url("https://github.com/qltysh/qlty.git"),
            normalize_git_url("git@github.com:qltysh/qlty.git")
        );

        assert_eq!(
            normalize_git_url("http://github.com/qltysh/qlty"),
            normalize_git_url("https://github.com/qltysh/qlty.git")
        );
    }
}
