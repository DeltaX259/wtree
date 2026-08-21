use std::path::PathBuf;
use crate::utils::dir::{
    get_current_dir,
    get_top_dir,
};
use crate::utils::worktree::get_current_worktree;
use crate::utils::git::{get_git_status, get_git_output};


pub fn amend(all: bool, push: bool) -> Result<(), Box<dyn std::error::Error>> {
    let branch = get_current_worktree()?;
    let path = get_top_dir()?;
    let current_branch = PathBuf::from(format!("{}/{}", path.display(), branch).trim().to_string());

    if all {
        let _ = get_git_status(&vec!["add", "."], &current_branch)?;
    }

    let _ = get_git_status(&vec!["commit", "--amend", "--no-edit"], &current_branch)?;

    if push {
        git_push(true)?;
    }
    
    Ok(())
}

pub fn git_push(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let branch = get_current_worktree()?;

    let mut args = vec!("push", "-q");

    match check_upstream() {
        Err(_) => {
            args.push("--set-upstream");
            args.push("origin");
            args.push(&branch);
            println!("Set upstream branch to: {}", &branch);
        },
        _ => (),
    }

    if force {
        args.push("--force-with-lease");
    }

    let _ = get_git_status(&args, &current_dir)?;

    println!("Push successful");

    Ok(())
}

fn check_upstream() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let output = get_git_output(&vec!["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"], &current_dir)?;

    if output.status.success() {
        return Ok(())
    } else {
        return Err("No upstream found".into())
    }
}
