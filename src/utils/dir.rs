use std::path::PathBuf;
use std::fs;

pub fn get_current_dir() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub fn get_top_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
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


