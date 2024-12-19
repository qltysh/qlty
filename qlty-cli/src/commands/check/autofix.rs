use crate::ui::Steps;
use anyhow::Result;
use console::Emoji;
use qlty_check::llm::Fixer;
use qlty_check::{executor::staging_area::StagingArea, Results, Settings};
use tracing::info;

static ROBOT: Emoji<'_, '_> = Emoji("🤖  ", "");

pub fn autofix(
    results: &Results,
    settings: &Settings,
    staging_area: &StagingArea,
    steps: Option<&mut Steps>,
) -> Result<Results> {
    if settings.ai && !results.issues.is_empty() {
        let mut fixer = Fixer::new(settings, staging_area, results);
        fixer.plan();

        if fixer.completions_count() > 0 {
            if let Some(steps) = steps {
                steps.start(ROBOT, "Generating AI completions...".to_string());
                eprintln!();
            }
            let results = fixer.generate_fixes()?;
            return Ok(results);
        } else {
            info!("No issues to fix with AI");
        }
    }
    Ok(results.clone())
}
