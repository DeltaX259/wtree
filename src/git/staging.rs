use std::process::Command;
use colored::Colorize;
use std::collections::HashSet;

use crate::utils::dir::{
    get_current_dir,
};

pub fn stage_files(mut files: Option<Vec<String>>, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let staged_files_initial = get_staged_files()?;

    if all {
        let mut file = Vec::new();
        file.push(".".to_string());
        files = Some(file);        
    }
    
    let files = match files {
        Some(x) => x,
        None => {
            println!("No files passed");
            return Ok(())
        }
    };
    
    for file in files.iter() {
        stage_file(file)?;
    }

    let staged_files_final = get_staged_files()?;
    let diff = list_diff(staged_files_final, staged_files_initial);
    for value in diff.into_iter() {
        println!("{} -> {}", value.red(), value.green());
    }
    Ok(())
}

fn stage_file(file: &String) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();

    let _ = Command::new("git")
        .args(["add", &file])
        .current_dir(current_dir)
        .status()?;
    
    Ok(())
}

pub fn unstage(file: Option<String>, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let staged_files_initial = get_staged_files()?;

    if file.is_none() && !all {
        return Err("No file specified".into());
    }

    if !all {
        let file = file.unwrap();
        unstage_file(file)?;
    } else {
        let current_dir = get_current_dir();

        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&current_dir)
            .output()?;

        let status = String::from_utf8_lossy(&output.stdout).to_string();
        for line in status.lines() {
            if line.starts_with("A ") || line.starts_with("M ") {
                unstage_file(line[3..].to_string())?;
            }
        }
    }
    
    let staged_files_final = get_staged_files()?;
    let diff = list_diff(staged_files_initial, staged_files_final);
    for value in diff.into_iter() {
        println!("{} -> {}", value.green(), value.red());
    }

    Ok(())
}

fn unstage_file(file: String) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_current_dir();

    let status = Command::new("git")
        .args(["restore", "--staged", &file])
        .current_dir(&path)
        .status()?;

    if !status.success() {
        return Err(format!("Failed to stage file {}", file).into())
    }
  
    Ok(())
}

fn get_staged_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(&current_dir)
        .output()?;
    
    let staged_files_list = String::from_utf8_lossy(&output.stdout).to_string();
    let staged_files = string_to_vec(staged_files_list)?;
    Ok(staged_files)
}

fn string_to_vec(list: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result: Vec<String> = Vec::new();
    for line in list.lines() {
        result.push(line.trim().to_string());
    }
    return Ok(result)
}

fn list_diff(l1: Vec<String>, l2: Vec<String>) -> Vec<String> {
    let list_2: HashSet<String> = l2.into_iter().collect();

    l1.into_iter()
        .filter(|s| !list_2.contains(s))
        .clone()
        .collect()
}