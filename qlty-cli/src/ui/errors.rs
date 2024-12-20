use anyhow::Result;
use console::style;
use qlty_check::Report;

#[derive(Debug)]
pub struct ErrorsFormatter {
    report: Report,
}

impl ErrorsFormatter {
    pub fn new(report: &Report) -> Self {
        Self {
            report: report.clone(),
        }
    }
}

impl ErrorsFormatter {
    pub fn write_to(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        let cwd = std::env::current_dir().expect("Unable to identify current directory");

        for invocation in &self.report.invocations {
            let absolute_outfile_path = invocation.outfile_path();
            let outfile_path = pathdiff::diff_paths(&absolute_outfile_path, &cwd).unwrap();

            writeln!(
                writer,
                "{}",
                style(format!("# {}:", outfile_path.display())).dim()
            )?;

            let content = std::fs::read_to_string(absolute_outfile_path)?;
            writeln!(writer, "{}", content)?;
        }

        Ok(())
    }
}
