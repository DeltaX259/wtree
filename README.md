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

- **Clone** – Clone a git worktree/repo (use -b to specify a branch)
- **Fetch** – Fetch updates from remote repo
- **Add** – Add a new worktree/branch
- **Delete** – Remove a local worktree directory
- **Purge** – Delete branch from both worktree and repository (doesn't delete from remote source)
- **List** – Show local branches (use -a to show all branches)
- **Top** – Print the root directory containing your worktrees
- **Current Worktree** – Show the active branch in the current worktree
- **Base** – Get the branch of the base worktree (this shows the inital branch copied using `wtree clone`
- **Status** – Colored Git status (`staged`, `unstaged`, `untracked`, etc.)
- **Log** – Show commit history with simplified and colorful output

