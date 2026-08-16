use std::io::Error;
use std::process::{Command, Output, ExitStatus};
use std::path::PathBuf;

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