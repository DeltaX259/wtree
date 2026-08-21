use crate::utils::dir::get_current_dir;
use crate::utils::git::{get_git_status};

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