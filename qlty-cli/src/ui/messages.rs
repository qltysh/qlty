use anyhow::Result;
use console::style;
use qlty_check::Report;
use qlty_types::analysis::v1::Message;

pub fn print_installation_error_messages(
    writer: &mut dyn std::io::Write,
    report: &Report,
) -> Result<()> {
    let installation_error_messages: Vec<&Message> = report
        .messages
        .iter()
        .filter(|m| m.ty.starts_with("executor.install."))
        .collect();

    if !installation_error_messages.is_empty() {
        writeln!(writer)?;
        writeln!(
            writer,
            "{}",
            style("Installation errors found:").red().bold()
        )?;
        for message in &installation_error_messages {
            writeln!(writer, "  - {}", message.message)?;
        }
        writeln!(writer)?;
    }

    Ok(())
}
