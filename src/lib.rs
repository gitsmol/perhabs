use std::fs::{self, DirEntry, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::{error::Error, fmt};

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

pub fn read_file(filepath: &PathBuf) -> BufReader<File> {
    let _file = File::open(filepath).unwrap();
    let lines = BufReader::new(_file);
    return lines;
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

#[derive(Debug)]
pub enum PhError {
    // Errors from external libraries...
    Io(io::Error),
    // Errors raised by us...
    Regular(ErrorKind),
    Custom(String),
}

impl Error for PhError {
    fn description(&self) -> &str {
        match *self {
            PhError::Io(ref err) => err.description(),
            PhError::Regular(ref err) => err.as_str(),
            PhError::Custom(ref err) => err,
        }
    }
}

impl fmt::Display for PhError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PhError::Io(ref err) => err.fmt(f),
            PhError::Regular(ref err) => write!(f, "A regular error occurred {:?}", err),
            PhError::Custom(ref err) => write!(f, "A custom error occurred {:?}", err),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    InvalidInput, // etc
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match *self {
            ErrorKind::InvalidInput => "Invalid input",
        }
    }
}
