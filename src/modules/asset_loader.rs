use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

use ehttp::Response;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use self::{exercise_config_collection::ExerciseConfigCollection, perhabs_config::PerhabsConfig};

pub mod exercise_config;
pub mod exercise_config_collection;
pub mod perhabs_config;
pub mod sentences;

/// AppData is loaded when launching Perhabs. Individual modules/windows get app-wide
/// data through a reference to this struct.
pub struct AppData {
    pub config: Option<PerhabsConfig>,
    pub config_promise: Option<Promise<ehttp::Result<Response>>>,
    pub excconfig: Option<ExerciseConfigCollection>,
    pub excconfig_promise: Option<Promise<ehttp::Result<Response>>>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            config: None,
            config_promise: None,
            excconfig: None,
            excconfig_promise: None,
        }
    }
}

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
