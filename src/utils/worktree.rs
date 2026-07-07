use std::process::Command;
use crate::utils::dir::{
    get_top_dir,
    get_current_dir,
};

pub fn get_base() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = get_top_dir()?;
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&base_dir)
        .output()?;

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }
    Ok(())
}

pub fn worktree_top() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", get_top_dir()?.display());
    Ok(())
}

pub fn get_all_worktrees() -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();

    let output = Command::new("git")
        .args(["branch"])
        // .args(["branch", "--color=always"])
        .current_dir(&current_dir)
        .output()
        .expect("Failed to run 'git branch'");

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.into());
    }
}

pub fn get_current_worktree() -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();

    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&current_dir)
        .output()?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }
}

