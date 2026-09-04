use std::path::PathBuf;
use colored::Colorize;
use crate::utils::git::{get_git_output};

pub fn get_git_status() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();


    let args = vec!["status", "--porcelain"];
    let output = get_git_output(&args, &current_dir)?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }

    let status = String::from_utf8_lossy(&output.stdout).to_string();

    let mut staged : Vec<&str> = Vec::new();
    let mut unstaged : Vec<&str> = Vec::new();
    let mut staged_deleted : Vec<&str> = Vec::new();
    let mut unstaged_deleted : Vec<&str> = Vec::new();
    let mut other : Vec<&str> = Vec::new();
    
    for line in status.lines() {
        match &line[..2] {
            "A " | "M " => staged.push(line),
            " M" | "??"=> unstaged.push(line),
            " D" => unstaged_deleted.push(line),
            "D " => staged_deleted.push(line),
            _ => other.push(line),
        }
    }

    if !staged.is_empty() {
        println!("\nStaged files:");
        for line in &staged {
            println!(" {}", line[2..].to_string().green());
        }

        if !staged_deleted.is_empty() {
            for line in &staged_deleted {
                println!(" {}", line[2..].to_string().red())
            }
        }
    }

    if !unstaged.is_empty() {
        println!("\nUnstaged files:");
        for line in &unstaged {
            println!(" {}", line[2..].to_string().yellow());
        }

        if !unstaged_deleted.is_empty() {
            for line in &unstaged_deleted {
                println!(" {}", line[2..].to_string().red())
            }
        }
    }
    
    if !other.is_empty() {
        println!("\nOther files:");
        for line in &other {
            println!(" {}", line[2..].to_string().red());
        }
    }

    if staged.is_empty() && unstaged.is_empty() && unstaged_deleted.is_empty() && staged_deleted.is_empty() && other.is_empty() {
        println!("Everything is up to date");
    }

    Ok(())
}

fn get_current_dir() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub fn get_logs(length: Option<String>) -> Result<(), Box<dyn std::error::Error>>{
    let current_dir = get_current_dir();
    let n = length.unwrap_or("10".to_string());

    let args = vec!["log", "-n", &n, "--pretty=format:%C(red)%h - %C(green)%an, %C(blue)%ar : %C(white)%s", "--color=always"];
    let output = get_git_output(&args, &current_dir)?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }

    Ok(())
}


