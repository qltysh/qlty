use thiserror::Error;

const OUTPUT_TAIL_LINES: usize = 10;

#[derive(Debug, Error)]
#[error("Command {command:?} exited with code {exit_code}")]
pub struct ToolCommandError {
    pub command: Vec<String>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ToolCommandError {
    pub fn output_tail(&self) -> String {
        let source = if self.stderr.trim().is_empty() {
            &self.stdout
        } else {
            &self.stderr
        };

        let lines: Vec<&str> = source
            .lines()
            .map(str::trim_end)
            .filter(|line| !line.is_empty())
            .collect();

        lines[lines.len().saturating_sub(OUTPUT_TAIL_LINES)..].join("\n")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn command_error(stdout: &str, stderr: &str) -> ToolCommandError {
        ToolCommandError {
            command: vec!["npm".to_string(), "install".to_string()],
            exit_code: 1,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        }
    }

    #[test]
    fn display_includes_command_and_exit_code() {
        assert_eq!(
            command_error("", "").to_string(),
            r#"Command ["npm", "install"] exited with code 1"#
        );
    }

    #[test]
    fn output_tail_prefers_stderr() {
        let error = command_error("out line", "err line");
        assert_eq!(error.output_tail(), "err line");
    }

    #[test]
    fn output_tail_falls_back_to_stdout() {
        let error = command_error("out line", " \n");
        assert_eq!(error.output_tail(), "out line");
    }

    #[test]
    fn output_tail_returns_last_lines() {
        let lines: Vec<String> = (1..=12).map(|n| format!("line {n}")).collect();
        let error = command_error("", &lines.join("\n"));

        assert_eq!(error.output_tail(), lines[2..].join("\n"));
    }

    #[test]
    fn output_tail_skips_blank_lines() {
        let error = command_error("", "one\n\n  \ntwo");
        assert_eq!(error.output_tail(), "one\ntwo");
    }
}
