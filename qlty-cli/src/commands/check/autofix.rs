use crate::ui::Steps;
use anyhow::Result;
use qlty_check::{executor::staging_area::StagingArea, Results, Settings};

#[cfg(feature = "llm")]
pub fn autofix(
    results: &Results,
    settings: &Settings,
    staging_area: &StagingArea,
    steps: Option<&mut Steps>,
) -> Result<Results> {
    use console::Emoji;
    use qlty_llm::Autofixer;
    use tracing::info;
    static ROBOT: Emoji<'_, '_> = Emoji("🤖  ", "");

    if settings.ai && !results.issues.is_empty() {
        let mut autofixer = Autofixer::new(settings, staging_area, results);
        autofixer.plan();

        if autofixer.completions_count() > 0 {
            if let Some(steps) = steps {
                steps.start(ROBOT, "Generating AI completions...".to_string());
                eprintln!();
            }
            let results = autofixer.generate_fixes()?;
            return Ok(results);
        } else {
            info!("No issues to fix with AI");
        }
    }
    Ok(results.clone())
}

#[cfg(not(feature = "llm"))]
pub fn autofix(
    results: &Results,
    _settings: &Settings,
    _staging_area: &StagingArea,
    _steps: Option<&mut Steps>,
) -> Result<Results> {
    Ok(results.clone())
}
