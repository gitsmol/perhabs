use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

pub mod exercise_config;
pub mod exercise_config_collection;
pub mod perhabs_config;
pub mod sentences;

/// Describes where an asset was loaded from.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AssetSource {
    Disk,
    Web,
    Default,
    Unknown,
}

impl AssetSource {
    pub fn to_string(&self) -> String {
        match self {
            AssetSource::Disk => String::from("Disk"),
            AssetSource::Web => String::from("Web"),
            AssetSource::Default => String::from("Default"),
            AssetSource::Unknown => String::from("Unknown"),
        }
    }
}

/// Write a string to a given filepath.
pub fn write_string_to_file(filepath: &Path, content: String) -> Result<(), io::Error> {
    match OpenOptions::new()
        .append(false)
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
