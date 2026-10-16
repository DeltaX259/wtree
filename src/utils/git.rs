use std::io::Error;
use std::process::{Command, Output, ExitStatus};
use std::path::PathBuf;
use crate::utils::dir::get_current_dir;

pub fn get_git_output(args: &Vec<&str>, dir: &PathBuf) -> Result<Output, Error> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
}

pub fn get_git_status(args: &Vec<&str>, dir: &PathBuf) -> Result<ExitStatus, Error> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
}

#[derive(Clone, PartialEq)]
pub enum FileState {
    New,
    Modified,
    Deleted,
    Renamed,
    Error,
}

#[allow(dead_code)]
pub fn get_status(path: &str) -> Result<FileState, Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let args = vec!["status", "--porcelain", path];
    let output = get_git_output(&args, &current_dir)?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string().into())
    }
    let line = String::from_utf8_lossy(&output.stdout).to_string();
    if line.len() < 2 { return Err("Empty git status --porcelain line".into())}
    
    match &line[..2] {
        "A " | " A" | "??" => return Ok(FileState::New),
        "M " | " M" => return Ok(FileState::Modified),
        "D " | " D" => return Ok(FileState::Deleted),
        "R " | " R" => return Ok(FileState::Renamed),
        _ => return Ok(FileState::Error),
    };
}