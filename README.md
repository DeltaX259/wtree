# wtree

**A simple CLI tool for working with Git worktrees.**

Do you find yourself having multiple copies of a repo for different branches?
Do you find yourself having to stash your work, switch branches to review a PR, then unstash your work?

Then wtree might be able to help. wtree is a Rust-based helper that tries to make managing Git worktrees easy.
<br/>

## Overview

Clones a bare repository (into .bare) using `wtree clone <git repo>` <br/>
Allows the user to have multiple branches checked out at once, using git worktrees: `wtree add <branch>` <br/>
Branches can be easily switched between by changing directories, while preserving your current work  <br/>
These branches will be stored with all their files under `path/to/repo/<branch>` <br/>
```
wtree
    ├── dev
    ├── main
    └── preprod
```

Simplified, colorful outputs for some existing git commands: <br/>
- `wtree status` in place of `git status` <br/>
- `wtree list` in place of `git branch` <br/>
- `wtree log` in place of `git log` <br/>
<br/>

## Installation
### Build from source

**Prerequisites:** <br/>
Install Rust: https://rust-lang.org/tools/install/ <br/>

**Download and build**
```bash
git clone https://github.com/DeltaX259/wtree
cd wtree
cargo build --release
```
To be able to call wtree from any directory (bash example):
```bash
mkdir -p ~/.local/bin
echo -e 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
cp target/release/wtree ~/.local/bin/
```
### Download pre-compiled version(s) from releases
https://github.com/DeltaX259/wtree/releases <br/>
<br/>
To be able to call wtree from any directory (bash example):
```bash
mkdir -p ~/.local/bin
echo -e 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
cp <path-to-downlaoded-file> ~/.local/bin/wtree
```

## Features

Commands:
-  **clone**     Initialise a git worktree/repo [aliases: init]
-  **fetch**     Fetch updates from remote repo
-  **delete**    Delete local download of worktree [aliases: remove, rm]
-  **purge**     Remove branch from local repo and local worktree
-  **add**       Download remote branch and add to local worktree
-  **list**      List local/downloaded branchs
-  **top**       Returns top worktree directory
-  **worktree**  Returns current branch
-  **log**       Get commit logs [aliases: logs, history]
-  **base**      base worktree/branch
-  **status**    Lists staged, unstaged, untracked files [aliases: stat]
-  **amend**     amend your last commit
-  **push**      push staged changes
-  **unstage**   Move files from staged to unstaged [aliases: restore]
-  **stage**     Stage files to commit
-  **diff**      Fille differences
-  **help**      Print this message or the help of the given subcommand(s)
