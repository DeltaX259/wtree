use std::env::current_dir;
use std::path::PathBuf;

use crate::utils::dir::get_current_dir;
use crate::utils::git::{get_git_output, get_git_status};

pub fn fetch_repo() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();

    let status = get_git_status(&vec!["fetch", "--all"], &current_dir)?;

    if !status.success() {
        return Err("git fetch failed".into());
    }

    println!("Successfully fetched repo");

    Ok(())
}


pub fn pull() -> Result<(), Box<dyn std::error::Error>> {
    fetch_repo()?;

    let current_dir = get_current_dir();
    let status = get_git_status(&vec!["pull", "--rebase"], &current_dir)?;

    if !status.success() {
        return Err("git pull failed".into())
    }

    Ok(())
}

pub fn check_upstream(path: String, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Entered check upstream");
    let output = get_git_output(&vec!["status", "-sb"], &PathBuf::from(&path))?;
    let lines = String::from_utf8_lossy(&output.stdout).to_string();
    let mut line = "";
    for l in lines.lines() {
        line = l;
        break
    }
    let check = format!("## {}...origin/{}", &branch, &branch);
    if line.contains(&check) {
        get_git_status(&vec!["branch", &format!("--set-upstream-to=origin/{}", &branch)], &PathBuf::from(&path))?;
    }
    Ok(())
}
