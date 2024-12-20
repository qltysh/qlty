use console::style;
use qlty_types::analysis::v1::Level;

pub fn formatted_level(level: Level) -> String {
    match level {
        Level::High => style("high  ").red().to_string(),
        Level::Medium => style("medium").magenta().to_string(),
        Level::Low => style("low   ").yellow().to_string(),
        Level::Fmt => style("fmt   ").dim().to_string(),
        _ => format!("{:?}", level),
    }
}
