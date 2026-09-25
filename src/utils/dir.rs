use std::path::PathBuf;
use std::fs;

pub fn get_current_dir() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub fn get_top_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = get_current_dir();
    match dir_finder(&dir, ".bare") {
        Some(dir) => return Ok(dir),
        None => {
            match dir_finder(&dir, ".git") {
                Some(dir) => return Ok(dir),
                None => return Err("Cannot find \".bare\" or \".git\" in directory structure".into())
            }
        }
    }

}

fn dir_finder(dir: &PathBuf, extention: &str) -> Option<PathBuf> {
    let mut dir = dir.to_owned();

    loop {
        let bare = dir.join(extention);
        if bare.exists() {
            return Some(PathBuf::from(dir))
        }

        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => {
                return None;
            }
        }
    }
}

pub fn get_top_git() ->  Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = get_current_dir();
    match dir_finder(&dir, ".git") {
        Some(dir) => return Ok(dir),
        None => return Err("Could not find .git".into())
    }
}

