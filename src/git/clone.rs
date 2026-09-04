use std::path::PathBuf;
use std::fs;
use crate::git::pull::fetch_repo;
use crate::utils::dir::get_current_dir;
use crate::utils::git::{get_git_status, get_git_output};


pub fn clone_repo(repo_url: &str, branch: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let repo_name = repo_url
        .split('/')
        .last()
        .ok_or("invalid repository URL")?
        .trim_end_matches(".git");

    let repo_dir = PathBuf::from(format!("{}/{}", get_current_dir().display().to_string(), repo_name));
    let bare_dir = repo_dir.join(".bare");
    fs::create_dir_all(&bare_dir)?;

    let mut args = vec!("clone", "-q", "--bare", repo_url, ".bare");

    match branch {
        Some(ref branch_name) => {
            args.push("-b");
            args.push(&branch_name);
        },
        None => (),
    };

    let status = get_git_status(&args, &repo_dir)?;

    if !status.success() {
        return Err("git clone --bare failed".into());
    }

    fs::write(repo_dir.join(".git"), "gitdir: ./.bare\n")?;

    fetch_repo()?;

    let branch = match branch {
        Some(ref branch_name) => branch_name,
        None => {
            let output = get_git_output(&vec!["branch", "--show-current"], &repo_dir)?;

            &String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string()
        }
    };

    let status = get_git_status(&vec!["worktree", "add", &branch], &repo_dir)?;

    if !status.success() {
        return Err("git worktree add failed".into());
    }
    // crate::git::pull::check_upstream(&branch)?;

    println!("Repository initialized");

    Ok(())
}
