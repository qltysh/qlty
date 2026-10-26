//! Helpers for commands that run as Git hooks.

use anyhow::{bail, Result};
use console::style;
use std::io::{self, BufRead as _, Read as _};
use std::thread;

/// Reads the input Git passes to a pre-push hook on stdin. `None` when there
/// is nothing to push.
pub fn read_pre_push_stdin() -> Result<Option<String>> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    if buffer.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(buffer))
    }
}

/// Lets the user press Enter to skip a slow hook: the process exits
/// successfully, so Git continues.
pub fn exit_on_enter() {
    eprintln!("Tap {} to skip...", style("enter").bold(),);

    thread::spawn(move || loop {
        let mut input = String::new();

        if let Ok(tty) = std::fs::File::open("/dev/tty") {
            let mut tty_reader = io::BufReader::new(tty);
            tty_reader.read_line(&mut input).ok();

            if input == "\n" {
                std::process::exit(0);
            }
        }
    });
}

/// One ref being pushed, as a line of pre-push input:
/// `<local-ref> SP <local-object-name> SP <remote-ref> SP <remote-object-name>`.
///
/// https://git-scm.com/docs/githooks#_pre_push
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefUpdate {
    pub local_ref: String,
    pub local_object: String,
    pub remote_ref: String,
    pub remote_object: String,
}

impl RefUpdate {
    pub fn parse_all(input: &str) -> Result<Vec<Self>> {
        input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(Self::parse)
            .collect()
    }

    fn parse(line: &str) -> Result<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let [local_ref, local_object, remote_ref, remote_object] = parts[..] else {
            bail!("Unexpected Git pre-push input: {line:?}");
        };
        Ok(Self {
            local_ref: local_ref.to_owned(),
            local_object: local_object.to_owned(),
            remote_ref: remote_ref.to_owned(),
            remote_object: remote_object.to_owned(),
        })
    }

    /// The push deletes the remote ref.
    pub fn is_deletion(&self) -> bool {
        is_zero(&self.local_object)
    }

    /// The remote ref does not exist yet.
    pub fn creates_remote_ref(&self) -> bool {
        is_zero(&self.remote_object)
    }
}

fn is_zero(object: &str) -> bool {
    object.chars().all(|c| c == '0')
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO: &str = "0000000000000000000000000000000000000000";
    const LOCAL: &str = "1111111111111111111111111111111111111111";
    const REMOTE: &str = "2222222222222222222222222222222222222222";

    #[test]
    fn parses_every_line() {
        let input = format!(
            "refs/heads/a {LOCAL} refs/heads/a {REMOTE}\nrefs/heads/b {LOCAL} refs/heads/b {ZERO}\n"
        );
        let refs: Vec<String> = RefUpdate::parse_all(&input)
            .unwrap()
            .into_iter()
            .map(|update| update.local_ref)
            .collect();
        assert_eq!(refs, ["refs/heads/a", "refs/heads/b"]);
    }

    #[test]
    fn rejects_a_short_line() {
        assert!(RefUpdate::parse_all("refs/heads/a 1111").is_err());
    }

    #[test]
    fn a_zero_local_object_is_a_deletion() {
        let update = RefUpdate::parse(&format!("(delete) {ZERO} refs/heads/a {REMOTE}")).unwrap();
        assert!(update.is_deletion());
    }

    #[test]
    fn a_zero_remote_object_creates_the_remote_ref() {
        let update =
            RefUpdate::parse(&format!("refs/heads/a {LOCAL} refs/heads/a {ZERO}")).unwrap();
        assert!(update.creates_remote_ref());
    }
}
