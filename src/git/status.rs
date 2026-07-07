use std::process::Command;
use std::path::PathBuf;
use colored::Colorize;

pub fn get_git_status() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&current_dir)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }

    let status = String::from_utf8_lossy(&output.stdout).to_string();

    let mut staged : Vec<&str> = Vec::new();
    let mut unstaged : Vec<&str> = Vec::new();
    let mut untracked : Vec<&str> = Vec::new();
    let mut other : Vec<&str> = Vec::new();
    
    for line in status.lines() {
        if line.starts_with("A ") || line.starts_with("M ") {
            staged.push(line);
        } else if line.starts_with(" M") {
            unstaged.push(line);
        } else if line.starts_with("??") {
            untracked.push(line);
        } else {
            other.push(line);
        }
    }

    if !staged.is_empty() {
        println!("Staged files:");
        for line in &staged {
            println!(" {}", line[2..].to_string().green());
        }
    }

    if !unstaged.is_empty() {
        println!("\nUnstaged files:");
        for line in &unstaged {
            println!(" {}", line[2..].to_string().yellow());
        }
    }

    if !untracked.is_empty() {
        println!("\nUntracked files:");
        for line in &untracked {
            println!(" {}", line[2..].to_string().red());
        }
    }

    if !other.is_empty() {
        println!("\nOther files:");
        for line in &other {
            println!(" {}", line[2..].to_string().red().italic());
        }
    }

    if staged.is_empty() && unstaged.is_empty() && untracked.is_empty() && other.is_empty() {
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

    let output = Command::new("git")
        .args(["log", "-n", &n, "--pretty=format:%C(red)%h - %C(green)%an, %C(blue)%ar : %C(white)%s", "--color=always"])
        .current_dir(&current_dir)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }

    Ok(())
}
