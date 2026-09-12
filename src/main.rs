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

    #[command(about="Commit staged changes")]
    Commit,

    #[command(about="File differences")]
    Diff {
        file: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    macro_rules! run {
        ($expr:expr) => {
            if let Err(e) = $expr {
                eprintln!("[Error]: {e}");
                return ExitCode::FAILURE;
            }
        };
    }
    match cli.command {
        Commands::Clone { repo, branch } => run!(git::clone::clone_repo(&repo, branch)),
        Commands::Fetch => run!(git::pull::fetch_repo()),
        Commands::Delete { branch } => run!(git::branch::delete_branch(&branch)),
        
        Commands::Purge { branch } => run!(git::branch::purge_branch(&branch)),
        Commands::List { all }=> run!(git::branch::branch_list(all)),
        Commands::Top => run!(worktree_top()),
        
        Commands::Log { length } => run!(git::status::get_logs(length)),
        Commands::Base => run!(get_base()),
        Commands::Status => run!(git::status::get_git_status()),
        
        Commands::Unstage { file, all } => run!(git::staging::unstage(file, all)),
        Commands::Stage { files, all } => {
            if files.is_none() && !all {
                run!(git::staging::stage_selector())
            } else {
                run!(git::staging::stage_files(files, all))
            }
        },
        Commands::Amend { all, push } => run!(git::commit::amend(all, push)),

        Commands::Commit => run!(git::commit::make_commit()),
        Commands::Push { force } => run!(git::commit::git_push(force)),
        Commands::Diff { file } => run!(git::diff::diff(file)),
        
        Commands::Pull => run!(git::pull::pull()),
        Commands::Branch { branch } => {
            match branch {
                Some(branch) => run!(git::branch::add_branch(&branch)),
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
    }
    ExitCode::SUCCESS
}
