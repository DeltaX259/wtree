use std::process::Command;
use crate::utils::dir::{
    get_current_dir,
    get_top_dir,
};
use crate::utils::worktree::get_current_worktree;


pub fn amend(all: bool, push: bool) -> Result<(), Box<dyn std::error::Error>> {
    let branch = get_current_worktree()?;
    let path = get_top_dir()?;
    let current_branch = format!("{}/{}", path.display(), branch).trim().to_string();

    if all {
        let _ = Command::new("git")
            .args(["add", "."])
            .current_dir(&current_branch)
            .status()?;
    }

    let _ = Command::new("git")
        .args(["commit", "--amend", "--no-edit"])
        .current_dir(&current_branch)
        .status()?;

    if push {
        git_push(true)?;
    }
    
    Ok(())
}

pub fn git_push(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let mut args = vec!("push");

    if force {
        args.push("--force-with-lease");
    }

    let _ = Command::new("git")
        .args(&args)
        .current_dir(current_dir)
        .status()?;

    Ok(())
}
