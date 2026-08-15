use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod git;
mod utils;
use utils::worktree::{
    get_base,
    worktree_top,
    get_current_worktree,
};

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

    #[command(about="List local/downloaded branchs")]
    List {
        #[arg(short = 'a', long = "all")]
        #[arg(help="List all available worktrees")]
        all: bool,
    },

    #[command(about="Returns top worktree directory")]
    Top,

    #[command(about="Returns current branch, or creates new branch if name provided")]
    Branch {
        #[arg(help="Branch to add/download")]
        branch: Option<String>,
    },

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

    #[command(about="amend your last commit")]
    Amend {
        #[arg(short = 'a', long = "all")]
        all: bool,
        #[arg(short = 'p', long = "push")]
        push: bool,
    },

    #[command(about="push staged changes")]
    Push {
        #[arg(short = 'f', long = "force")]
        force: bool,
    },

    #[command(about="pull remote changes/updates")]
    Pull,
    
    #[command(visible_alias = "restore")]
    #[command(about="Move files from staged to unstaged")]
    Unstage {
        file: Option<String>,
        #[arg(short='a', long="all")]
        all: bool,
    },

    #[command(about="Stage files to commit")]
    #[command(visible_alias = "add")]
    Stage {
        #[arg(help="Files to stage")]
        files: Option<Vec<String>>,
        #[arg(help="Stage all files")]
        #[arg(short = 'a', long = "all")]
        all: bool,
    },
    
    #[command(about="Fille differences")]
    Diff {
        file: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Clone { repo, branch } => {
            if let Err(e) = git::clone::clone_repo(&repo, branch) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Fetch => {
            if let Err(e) = git::pull::fetch_repo() {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Delete { branch } => {
            if let Err(e) = git::branch::delete_branch(&branch) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Purge { branch } => {
            if let Err(e) = git::branch::purge_branch(&branch) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Branch { branch } => {
            match branch {
                Some(branch) => {
                    if let Err(e) = git::branch::add_branch(&branch) {
                        eprintln!("[Error]: {e}");
                        return ExitCode::FAILURE;
                    }
                }
                None => {
                    match get_current_worktree() {
                        Ok(worktree) => println!("Current worktree: {}", worktree.trim()),
                        Err(e) => {
                            eprintln!("[Error]: {e}");
                            return ExitCode::FAILURE;
                        },
                    }
                }
            }
 
        }
        Commands::List { all }=> {
            if let Err(e) = git::branch::branch_list(all) {
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

        Commands::Log { length } => {
            if let Err(e) = git::status::get_logs(length) {
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
            if let Err(e) = git::status::get_git_status() {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE
            }
        }
        Commands::Unstage { file, all } => {
            if let Err(e) = git::staging::unstage(file, all) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Stage { files, all } => {
            if let Err(e) = git::staging::stage_files(files, all) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE
            }
        }
        Commands::Amend { all, push } => {
            if let Err(e) = git::commit::amend(all, push) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Push { force } => {
            if let Err(e) = git::commit::git_push(force) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Diff { file } => {
            if let Err(e) = git::diff::diff(file) {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
        Commands::Pull => {
            if let Err(e) = git::pull::pull() {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}


