use ehttp::Response;
use log::debug;
use poll_promise::Promise;
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, BufReader},
};

use super::{sentences::SentenceFile, AssetSource};

/// PerhabsConfig
/// PerhabsConfig contains information about file locations.
///

#[derive(Deserialize, Debug)]
pub struct PerhabsConfig {
    pub disk_root: String,
    pub web_root: String,
    pub config_path: String,
    pub excconfig_path: String,
    pub sentences_path: String,
    pub sentences_files: Vec<SentenceFile>,
    pub episodic_memory_path: String,
    pub episodic_memory_files: Vec<SentenceFile>,
    pub source: AssetSource,
}

impl Default for PerhabsConfig {
    fn default() -> Self {
        debug!("Getting Perhabs config: falling back to default.");
        Self {
            disk_root: String::from("./"),
            web_root: String::from("https://www.polyprax.nl/perhabs/"),
            // These are basically hardcoded because without a config
            // there is no way to know where to get this data...
            config_path: String::from("appdata/config.json"),
            excconfig_path: String::from("appdata/exercise_configs.json"),
            sentences_path: String::from("excdata/sentences/"),
            sentences_files: vec![SentenceFile {
                filename: String::from("sentences_EN.txt"),
                language: String::from("English"),
            }],
            episodic_memory_path: String::from("./excdata/episodic_memory/"),
            episodic_memory_files: vec![SentenceFile {
                filename: String::from("episodic_memory_EN.txt"),
                language: String::from("English"),
            }],
            source: AssetSource::Default,
        }
    }
}

impl PerhabsConfig {
    pub fn from_disk() -> io::Result<Self> {
        debug!("Getting Perhabs config: trying disk.");
        // Might as well hardcode this, after all the path is in the file...
        let file = File::open("./appdata/config.json")?;
        let reader = BufReader::new(file);
        let mut de = serde_json::Deserializer::from_reader(reader);
        let config = PerhabsConfig::deserialize(&mut de)?;
        Ok(config)
    }

    pub fn from_web() -> Promise<ehttp::Result<Response>> {
        debug!("Getting Perhabs config: trying web.");
        let path = format!(
            "{}{}",
            PerhabsConfig::default().web_root,
            PerhabsConfig::default().config_path
        );
        let (sender, promise) = Promise::new();
        let request = ehttp::Request::get(path);
        ehttp::fetch(request, move |response| {
            sender.send(response);
        });

        promise
    }
}
