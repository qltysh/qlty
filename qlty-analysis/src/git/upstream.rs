use anyhow::Result;
use git2::Repository;
use qlty_config::Workspace;
use tracing::{info, debug};

pub fn compute_upstream(workspace: &Workspace, specified: &Option<String>) -> Option<String> {
    if let Some(specified) = specified {
        Some(specified.to_owned())
    } else if let Ok(upstream) = infer_upstream(workspace) {
        info!("Inferred upstream: {:?}", upstream);
        upstream
    } else {
        None
    }
}

fn find_remote_default_branch(repo: &Repository) -> Option<String> {
    if let Ok(_) = repo.find_remote("origin") {
        if let Ok(head) = repo.find_reference("refs/remotes/origin/HEAD") {
            if let Some(target) = head.symbolic_target() {
                return Some(target.replace("refs/remotes/", ""));
            }
            debug!("Target not found for HEAD");
        }
    }

    None
}

fn infer_upstream(workspace: &Workspace) -> Result<Option<String>> {
    let repo = workspace.repo()?;

    match find_remote_default_branch(&repo) {
        Some(branch) => Ok(Some(branch)),
        None => {
            debug!("Could not find HEAD branch for remote 'origin'. Checking for main/master/develop.");
            default_upstream(&repo)
        },
    }
}

fn default_upstream(repo: &Repository) -> Result<Option<String>> {
    let known_default_branches = ["main", "master", "develop"];

    for branch_name in known_default_branches.iter() {
        if repo.find_branch(branch_name, git2::BranchType::Local).is_ok() {
            debug!("Found {} branch. Using as upstream.", branch_name);
            return Ok(Some(branch_name.to_string()));
        }
    };

    debug!("No main/master/develop branch found.");
    Ok(None)
}
