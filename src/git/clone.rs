use std::path::PathBuf;
use std::process::Command;
use std::fs;
use crate::git::pull::fetch_repo;
use crate::utils::dir::get_current_dir;

pub fn clone_repo(repo_url: &str, branch: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let repo_name = repo_url
        .split('/')
        .last()
        .ok_or("invalid repository URL")?
        .trim_end_matches(".git");

    let repo_dir = PathBuf::from(format!("{}/{}", get_current_dir().display().to_string(), repo_name));
    let bare_dir = repo_dir.join(".bare");
    fs::create_dir_all(&bare_dir)?;

    let status = match branch {
        Some(ref branch_name) => {
            Command::new("git")
                .arg("clone")
                .arg("--bare")
                .arg(repo_url)
                .arg(".bare")
                .arg("-b")
                .arg(&branch_name)
                .current_dir(&repo_dir)
                .status()?
        }
        None => {
            Command::new("git")
                .arg("clone")
                .arg("--bare")
                .arg(repo_url)
                .arg(".bare")
                .current_dir(&repo_dir)
                .status()?
        }
    };

    if !status.success() {
        return Err("git clone --bare failed".into());
    }

    fs::write(repo_dir.join(".git"), "gitdir: ./.bare\n")?;

    fetch_repo()?;

    let branch = match branch {
        Some(ref branch_name) => branch_name,
        None => {
            let output = Command::new("git")
                .args(["branch", "--show-current"])
                .current_dir(&repo_dir)
                .output()
                .unwrap();
            &String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string()
        }
    };

    let status = Command::new("git")
        .args(["worktree", "add", branch])
        .current_dir(&repo_dir)
        .status()?;

        if !status.success() {
            return Err("git worktree add failed".into());
        }

    println!("Repository initialized");

    Ok(())
}
