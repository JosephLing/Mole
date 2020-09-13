use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::error::CustomError;

/// there is probably a nice library for this but ahow
pub fn search_dir(path: &PathBuf, file_type: &str, underscore: bool) -> Vec<PathBuf> {
    let mut f: Vec<PathBuf> = Vec::new();
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            if let Some(ending) = entry.path().extension() {
                if ending == file_type
                    && (!underscore
                        || (underscore
                            && !path_file_name_to_string(&entry.path())
                                .expect("Could not decode file name")
                                .starts_with("_")))
                {
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
pub fn path_file_name_to_string(file_path: &Path) -> Result<String, CustomError> {
    Ok(file_path
        .file_name()
        .ok_or(CustomError::IOError(format!(
            "Could not find file name from {:?}",
            file_path
        )))?
        .to_str()
        .ok_or(CustomError::IOError(format!(
            "Could not convert the file name into a valid utf-8 string {:?}",
            file_path.file_name().unwrap()
        )))?
        .to_owned()
        .replace(".html", ""))
}
