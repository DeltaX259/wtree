use clap::{Parser, Subcommand};
use std::{
    fs,
    path::PathBuf,
    process::{Command, ExitCode}
};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "wtree")]
#[command(about = "A simple git worktree helper")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "init")]
    #[command(about="Initialise a git worktree/repo")]
    Clone {
        #[arg(help="Remote repo url")]
        repo: String,
        #[arg(short = 'b', help="Specific branch to clone")]
        branch: Option<String>
    },

    #[command(about="Fetch updates from remote repo")]
    Fetch,

    #[command(visible_alias="remove", visible_alias="rm")]
    #[command(about="Delete local download of worktree")]
    Delete {
        #[arg(help="Branch to remove from local repo")]
        branch: String,
    },
    
    #[command(about="Remove branch from local repo and local worktree")]
    Purge {
        branch: String,
    },

    #[command(about="Download remote branch and add to local worktree")]
    Add {
        #[arg(help="Branch to download")]
        branch: String,
    },

    #[command(about="List local/downloaded branchs")]
    List{
        #[arg(short = 'a', long = "all")]
        #[arg(help="List all available worktrees")]
        all: bool,
    },

    #[command(about="Returns top worktree directory")]
    Top,

    #[command(about="Returns current branch")]
    #[command(alias = "branch")]
    Worktree,

    #[command(about="Get commit logs")]
    #[command(visible_alias = "logs", visible_alias = "history")]
    Log {
        #[arg(short = 'n')]
        #[arg(help="Number of commits to show")]
        length: Option<String>,
    },

    #[command(about="base worktree/branch")]
    Base,

    #[command(visible_alias = "stat")]
    #[command(about="Lists staged, unstaged, untracked files")]
    Status,
    
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Clone { repo, branch } => {
            if let Err(e) = clone_repo(&repo, branch) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Fetch => {
            if let Err(e) = fetch_repo() {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Delete { branch } => {
            if let Err(e) = delete_branch(&branch) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Purge { branch } => {
            if let Err(e) = purge_branch(&branch) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Add { branch } => {
            if let Err(e) = add_branch(&branch) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::List { all }=> {
            if let Err(e) = branch_list(all) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Top => {
            if let Err(e) = worktree_top() {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Worktree => {
            match get_current_worktree() {
                Ok(worktree) => println!("Current worktree: {}", worktree.trim()),
                Err(e) => {
                    eprintln!("[Error]: {e}");
                    return ExitCode::FAILURE;
                },
            }
        }
        Commands::Log { length } => {
            if let Err(e) = get_logs(length) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Base => {
            if let Err(e) = get_base() {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Status => {
            if let Err(e) = get_status() {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE
            }
        }
    }
    ExitCode::SUCCESS
}

fn clone_repo(repo_url: &str, branch: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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

    let status = Command::new("git")
        .args([
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ])
        .current_dir(&repo_dir)
        .status()?;

    if !status.success() {
        return Err("git fetch failed".into());
    }

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
        return Err("git worktree prune failed".into());
    }

    println!("Repository initialized at {}", repo_dir.display());

    Ok(())
}

fn fetch_repo() -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = get_current_dir();

    let status = Command::new("git")
    .args([
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ])
    .current_dir(&repo_path)
    .status()?;

    if !status.success() {
        return Err("git fetch failed".into());
    }

    println!("Successfully fetched repo");

    Ok(())
}

fn delete_branch(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut path = get_top_dir().unwrap();
    path.push(branch);

    std::fs::remove_dir_all(&path)?;

    let status = Command::new("git")
        .args(["worktree", "prune"])
        .status()?;

    if !status.success() {
        return Err("git worktree prune failed".into());
    }

    Ok(())
}

fn purge_branch(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    delete_branch(branch)?;
    let output = Command::new("git")
        .args(["branch", "-D", branch])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().into());
    }

    Ok(())
}

fn add_branch(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_top_dir()?;

    let status = Command::new("git")
        .args(["worktree", "add", branch])
        .current_dir(&path)
        .status()?;
    
        if !status.success() {
        return Err("git worktree prune failed".into());
    }

    println!("Added {}", &branch);
    Ok(())
}

fn branch_list(all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let worktrees_result = get_all_worktrees();

    let all_wortrees = match worktrees_result {
        Ok(worktrees) => worktrees,
        Err(e) => return Err(e),
    };


    if all {
        for item in all_wortrees.lines() {
            if item.starts_with('*') {
                println!("(current) {}", item[2..item.len()].to_string().green());
            } else if item.starts_with('+') {
                println!("(local)   {}", item[2..item.len()].to_string().yellow());
            } else {
                println!("          {}", item[2..item.len()].to_string());
            }
        }
    } else {
        for item in all_wortrees.lines() {
            if item.starts_with('*') {
                println!("(current) {}", item[2..item.len()].to_string().green());
            } else if item.starts_with('+') {
                println!("          {}", item[2..item.len()].to_string());
            }
        }

    }

    Ok(())
}

fn worktree_top() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", get_top_dir()?.display());
    Ok(())
}

fn get_current_dir() -> PathBuf {
    std::env::current_dir().unwrap()
}

fn get_top_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut dir = get_current_dir();

    loop {
        let bare = dir.join(".bare");
        if bare.exists() && fs::metadata(&bare).map(|m| m.is_dir()).unwrap_or(false) {
            return Ok(PathBuf::from(dir))
        }

        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => {
                return Err("Not a compatabile directory: no '.bare' found".into());
            }
        }
    }
}

fn get_all_worktrees() -> Result<String, Box<dyn std::error::Error>> {
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

fn get_current_worktree() -> Result<String, Box<dyn std::error::Error>> {
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

fn get_logs(length: Option<String>) -> Result<(), Box<dyn std::error::Error>>{
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

fn get_base() -> Result<(), Box<dyn std::error::Error>> {
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

fn get_status() -> Result<(), Box<dyn std::error::Error>> {
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
        if line.starts_with("A ") {
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