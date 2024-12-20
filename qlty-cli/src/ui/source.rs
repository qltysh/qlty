use console::style;
use qlty_types::analysis::v1::Issue;

pub fn formatted_source(issue: &Issue) -> String {
    if !issue.rule_key.is_empty() {
        format!("{}", style(issue.rule_id()).dim())
    } else {
        format!("{}", style(issue.tool.clone()).dim())
    }
}
