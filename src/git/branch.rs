use std::process::Command;
use colored::Colorize;
use crate::utils::dir::get_top_dir;
use crate::utils::worktree::get_all_worktrees;

pub fn delete_branch(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut path = get_top_dir().unwrap();
    path.push(branch);

    std::fs::remove_dir_all(&path)?;

    let status = Command::new("git")
        .args(["worktree", "prune"])
        .status()?;

    if !status.success() {
        return Err("git worktree prune failed".into());
    }

    Ok(())
}

pub fn purge_branch(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    delete_branch(branch)?;
    let branch = branch.trim_end_matches("/");
    let output = Command::new("git")
        .args(["branch", "-D", branch])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }

    Ok(())
}

pub fn add_branch(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_top_dir()?;

    let status = Command::new("git")
        .args(["worktree", "add", branch])
        .current_dir(&path)
        .status()?;
    
        if !status.success() {
        return Err("git worktree prune failed".into());
    }

    println!("Added {}", &branch);
    Ok(())
}

pub fn branch_list(all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let worktrees_result = get_all_worktrees();

    let all_wortrees = match worktrees_result {
        Ok(worktrees) => worktrees,
        Err(e) => return Err(e),
    };


    if all {
        for item in all_wortrees.lines() {
            if item.starts_with('*') {
                println!("(current) {}", item[2..item.len()].to_string().green());
            } else if item.starts_with('+') {
                println!("(local)   {}", item[2..item.len()].to_string().yellow());
            } else {
                println!("          {}", item[2..item.len()].to_string());
            }
        }
    } else {
        for item in all_wortrees.lines() {
            if item.starts_with('*') {
                println!("(current) {}", item[2..item.len()].to_string().green());
            } else if item.starts_with('+') {
                println!("          {}", item[2..item.len()].to_string());
            }
        }

    }

    Ok(())
}
