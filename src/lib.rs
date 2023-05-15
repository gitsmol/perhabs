use std::fs::{self, DirEntry, File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};

pub fn dirwalk(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<DirEntry> = vec![];
    let mut paths: Vec<PathBuf> = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirwalk(&path)?;
            } else {
                paths.push(entry.path());
                files.push(entry);
            }
        }
    }
    Ok(paths)
}

///
pub fn read_file(filepath: &PathBuf) -> Result<BufReader<File>, std::io::Error> {
    match File::open(filepath) {
        Ok(file) => {
            let lines = BufReader::new(file);
            Ok(lines)
        }
        Err(e) => Err(e),
    }
}

/// Write a string to a given filepath.
pub fn write_string_to_file(filepath: &Path, content: String) -> Result<(), io::Error> {
    match OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(filepath)
    {
        Ok(mut file) => match file.write_all(content.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn numvec_to_string(seq: &Vec<u32>) -> String {
    let mut result = String::new();
    for i in seq {
        result += &i.to_string();
        result += ", ";
    }
    result.trim_end_matches(", ").to_string()
}

#[derive(PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
