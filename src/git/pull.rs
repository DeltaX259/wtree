use std::process::Command;
use crate::utils::dir::get_current_dir;

pub fn fetch_repo() -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = get_current_dir();

    let status = Command::new("git")
    .args(["fetch", "--all"])
    .current_dir(&repo_path)
    .status()?;

    if !status.success() {
        return Err("git fetch failed".into());
    }

    println!("Successfully fetched repo");

    Ok(())
}


pub fn pull() -> Result<(), Box<dyn std::error::Error>> {
    fetch_repo()?;

    let current_dir = get_current_dir();
    let status = Command::new("git")
        .args(["pull", "--rebase"])
        .current_dir(current_dir)
        .status()?;

    if !status.success() {
        return Err("git pull failed".into())
    }

    Ok(())
}