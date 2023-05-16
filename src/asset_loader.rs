use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use ehttp::{Response, Result};
use log::{self, debug};
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use crate::sessions::spatial_drawing::painters::Puzzle;

/// AppData is loaded when launching Perhabs. Individual modules/windows get app-wide
/// data through a reference to this struct.
pub struct AppData {
    pub config: Option<PerhabsConfig>,
    pub config_promise: Option<Promise<Result<Response>>>,
    pub excconfig: Option<ExcConfig>,
    pub excconfig_promise: Option<Promise<Result<Response>>>,
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

/// PerhabsConfig
/// PerhabsConfig contains information about file locations.
///

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

#[derive(Deserialize, Debug)]
pub struct PerhabsConfig {
    pub config_path_disk: String,
    pub config_path_web: String,
    pub excconfig_path_disk: String,
    pub excconfig_path_web: String,
    pub sentences_path_disk: String,
    pub sentences_path_web: String,
    pub sentences_files: Vec<SentenceFile>,
    pub source: AssetSource,
}

impl Default for PerhabsConfig {
    fn default() -> Self {
        debug!("Getting Perhabs config: falling back to default.");
        Self {
            config_path_disk: String::from("./appdata/config.json"),
            config_path_web: String::from("https://www.polyprax.nl/perhabs/appdata/config.json"),
            excconfig_path_disk: String::from("./appdata/config.json"),
            excconfig_path_web: String::from(
                "https://www.polyprax.nl/perhabs/appdata/excconfig.json",
            ),
            sentences_path_disk: String::from("./excdata/sentences/"),
            sentences_path_web: String::from("https://www.polyprax.nl/perhabs/excdata/sentences/"),
            sentences_files: vec![SentenceFile {
                filename: String::from("sentences_EN.txt"),
                language: String::from("English"),
            }],
            source: AssetSource::Default,
        }
    }
}

impl PerhabsConfig {
    pub fn from_disk() -> io::Result<Self> {
        debug!("Getting Perhabs config: trying disk.");
        let file = File::open("./appdata/config.json")?;
        let reader = BufReader::new(file);
        let mut de = serde_json::Deserializer::from_reader(reader);
        let config = PerhabsConfig::deserialize(&mut de)?;
        Ok(config)
    }

    pub fn from_web() -> Promise<Result<Response>> {
        debug!("Getting Perhabs config: trying web.");
        let path = PerhabsConfig::default().config_path_web;
        let (sender, promise) = Promise::new();
        let request = ehttp::Request::get(path);
        ehttp::fetch(request, move |response| {
            sender.send(response);
        });

        promise
    }
}

/// ExcerciseConfig
/// The ExcerciseConfig struct finds the most relevant config source using new().
///
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExcConfig {
    pub source: AssetSource,
    pub vergence: Vec<VergenceEx>,
    pub spatial_drawing: Vec<Puzzle>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VergenceEx {
    pub name: String,
    pub levels: Vec<Level>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Level {
    pub name: String,
    pub params: Parameters,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Parameters {
    pub step: isize,
}

impl Default for ExcConfig {
    fn default() -> Self {
        debug!("Getting excercise config: falling back to default.");
        ExcConfig {
            source: AssetSource::Default,
            vergence: vec![
                VergenceEx {
                    name: String::from("Convergence"),
                    levels: vec![Level {
                        name: String::from("Easy"),
                        params: Parameters { step: 1 },
                    }],
                },
                VergenceEx {
                    name: String::from("Divergence"),
                    levels: vec![Level {
                        name: String::from("Easy"),
                        params: Parameters { step: -1 },
                    }],
                },
            ],
            spatial_drawing: vec![Puzzle::new(5)],
        }
    }
}

impl ExcConfig {
    pub fn from_disk(path: &String) -> io::Result<ExcConfig> {
        debug!("Getting excercise config: trying disk.");
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut de = serde_json::Deserializer::from_reader(reader);
        let config = ExcConfig::deserialize(&mut de)?;
        Ok(config)
    }

    pub fn from_web(path: &String) -> Promise<Result<Response>> {
        debug!("Getting Perhabs config: trying web.");
        let (sender, promise) = Promise::new();
        let request = ehttp::Request::get(path);
        ehttp::fetch(request, move |response| {
            sender.send(response);
        });

        promise
    }
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct SentenceFile {
    pub filename: String,
    pub language: String,
}

/// Sentences
/// get_sentences finds the most relevant source for practice sentences.
///
pub fn get_sentences_disk(path: String) -> io::Result<Vec<String>> {
    debug!("Getting sentences: trying disk.");
    let file = File::open(path)?;
    let lines = BufReader::new(file);
    let mut contents = vec![];
    for line in lines.lines() {
        if let Ok(ip) = line {
            contents.push(ip);
        }
    }
    Ok(contents)
}

pub fn get_sentences_web(path: String) -> Promise<Result<Response>> {
    debug!("Getting Perhabs config: trying web.");
    let (sender, promise) = Promise::new();
    let request = ehttp::Request::get(path);
    ehttp::fetch(request, move |response| {
        sender.send(response);
    });

    promise
}

pub fn read_sentences_promise(file: &str) -> io::Result<Vec<String>> {
    debug!("Getting sentences: trying web.");
    let mut contents: Vec<String> = vec![];
    for line in file.lines() {
        contents.push(String::from(line));
    }
    Ok(contents)
}

pub fn default_sentences() -> Vec<String> {
    debug!("Getting sentences: falling back to default.");
    let sentences = vec![
        "these are waves, not mountains",
        "don't worry, be happy",
        "here follows extraordinarily complicated verbiage",
        "Sun rises in the east and sets in the west.",
        "Apples are a popular fruit.",
        "Mountains are made of rock and ice.",
        "Birds fly through the sky.",
        "Water is essential for life.",
        "She loves to dance in the rain.",
        "The cake was delicious.",
        "He can play the piano beautifully.",
        "They are going on a road trip.",
        "I will always remember this moment.",
        "We should explore more of the city.",
        "The sun was shining brightly.",
        "He has a contagious laugh.",
        "She dreams of traveling the world.",
    ];
    let mut returnvec = vec![];
    for s in sentences {
        returnvec.push(String::from(s))
    }
    returnvec
}

pub fn loading(ui: &mut egui::Ui) {
    ui.horizontal_centered(|ui| {
        ui.heading("Loading...");
        ui.spinner();
    });
}
