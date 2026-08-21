use std::process::Command;
use std::path::PathBuf;
use colored::Colorize;
use crate::utils::dir::{get_top_dir, get_current_dir};
use crate::utils::worktree::get_all_worktrees;
use crate::utils::git::{get_git_status, get_git_output};
use crate::git::pull::fetch_repo;

pub fn delete_branch(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut path = get_top_dir()?;
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
    
    let current_dir = get_current_dir();
    let output = get_git_output(&vec!["branch", "-D", branch], &current_dir)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }

    Ok(())
}

pub fn add_branch(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_top_dir()?;

    let status = get_git_status(&vec!["worktree", "add", branch], &path)?;

    if !status.success() {
        return Err("git worktree add failed".into());
    }
    println!("Fetching...");
    fetch_repo()?;

    match set_upstream(branch) {
        Ok(()) => { 
            println!("Set upstream");
            let repo_dir = format!("{}/{}", path.display(), branch);
            git_pull(PathBuf::from(repo_dir))?;
        }
        Err(_) => {println!("Failed to set upstream");}
    };
    
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

fn set_upstream(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let c = format!("--set-upstream-to=origin/{branch}");
    let _ = get_git_status(&vec!["branch",  &c, branch], &current_dir)?;

    Ok(())
}

fn git_pull(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Pulling: {}", &path.display());
    let _ = get_git_status(&vec!["pull", "--rebase"], &path)?;
    Ok(())
}