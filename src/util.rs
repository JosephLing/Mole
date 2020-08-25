use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::error::CustomError;

/// there is probably a nice library for this but ahow
pub fn search_dir(path: &PathBuf, file_type: &str) -> Vec<PathBuf> {
    let mut f: Vec<PathBuf> = Vec::new();
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            if let Some(ending) = entry.path().extension() {
                if ending == file_type {
                    f.push(entry.path());
                }
            }
        }
    }

    f
}

pub fn read_file(path: &Path) -> Result<String, CustomError> {
    match read_to_string(path)?.parse::<String>() {
        Ok(c) => Ok(c),
        Err(e) => Err(CustomError::IOError(e.to_string())),
    }
}

/// note: should only be used for .html files
pub fn path_file_name_to_string(file_path: &Path) -> Option<String> {
    Some(
        file_path
            .file_name()?
            .to_str()?
            .to_owned()
            .replace(".html", ""),
    )
}
