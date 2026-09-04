use colored::{ColoredString, Colorize};
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

///////////////////////////////////////////////////

pub fn my_commit() -> Result<(), Box<dyn  std::error::Error>> {
    
    Ok(())
}


fn get_changed_files() -> Result<(), Box<dyn std::error::Error>> {
    let new_files = get_file_status(vec!["A "])?;
    let modified_files = get_file_status(vec!["M "])?;
    let deleted_files = get_file_status(vec!["D "])?;

    let mut committed_files: Vec<ColoredString> = Vec::new();
    for file in new_files.iter() {
        committed_files.push(file.green())
    }
    for file in modified_files.iter() {
        committed_files.push(file.yellow())
    }
    for file in deleted_files.iter() {
        committed_files.push(file.red());
    }
    for line in committed_files.iter() {
        println!("Commited files: {}", line);
    }
    Ok(())
}

fn get_file_status(wanted: Vec<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = get_git_output(&vec!["status", "--porcelain"], &get_current_dir())?;
    let mut output: Vec<String> = Vec::new();
    
    if !response.status.success() {
        let stderr = String::from_utf8_lossy(&response.stderr);
        return Err(stderr.trim().into());
    }

    let status = String::from_utf8_lossy(&response.stdout).to_string();
    for line in status.lines() {
        if wanted.iter().any(|prefix| line.starts_with(prefix)) {
            output.push(line[2..].to_string());
        }
    }
    Ok(output)
}