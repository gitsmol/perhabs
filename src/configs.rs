use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::PathBuf,
};

use serde::Deserialize;
use serde_json::json;

// Exercise config model
#[derive(Deserialize, Debug)]
pub struct ExcConfig {
    pub exercises: Vec<Exercise>,
}

#[derive(Deserialize, Debug)]
pub struct Exercise {
    pub name: String,
    pub levels: Vec<Level>,
}

#[derive(Deserialize, Debug)]
pub struct Level {
    pub name: String,
    pub params: Parameters,
}

#[derive(Deserialize, Debug)]
pub struct Parameters {
    pub step: isize,
}

pub fn get_exc_config(filename: &PathBuf) -> ExcConfig {
    // TODO: wasm gets file from the web

    // TODO: not-wasm gets file from disk

    // This can fail so we provide a hardcoded default.
    match read_exc_config(filename) {
        Ok(res) => res,
        Err(_) => default_exc_config(),
    }
}

/// Get a config from disk
fn read_exc_config(filename: &PathBuf) -> io::Result<ExcConfig> {
    let _file = File::open(filename)?;
    let mut lines = BufReader::new(_file);
    let mut result = String::new();
    lines.read_to_string(&mut result).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Error reading config file: {}", e),
        )
    })?;

    let json: ExcConfig = serde_json::from_str(&result).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Error parsing json file: {}", e),
        )
    })?;

    Ok(json)
}

/// Provide some defaults because wasm can't use read_config.
fn default_exc_config() -> ExcConfig {
    let json_data = json!({
        "exercises": [
            {
                "name": "Divergence",
                "levels": [
                    {
                        "name": "Medium",
                        "params": {
                            "step": 2
                        }
                    }
                ]
            },
            {
                "name": "Convergence",
                "levels": [

                    {
                        "name": "Medium",
                        "params": {
                            "step": -2
                        }
                    }
                ]
            }
        ]
    });

    // Deserialize the JSON object into a Vec of Exercise objects
    let defexconf: ExcConfig = serde_json::from_value(json_data).unwrap();

    defexconf
}
