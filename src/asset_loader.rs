use log::{self, debug};
use reqwest::Error;
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

/// PerhabsConfig
/// PerhabsConfig contains information about file locations.
///

#[derive(Deserialize, Debug)]
pub enum Source {
    Disk,
    Web,
    Default,
    Unknown,
}
#[derive(Deserialize, Debug)]
pub struct PerhabsConfig {
    excconfig_path_disk: String,
    excconfig_path_web: String,
    pub sentences_path_disk: String,
    pub sentences_path_web: String,
    pub sentences_files: Vec<SentenceFile>,
    pub source: Source,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct SentenceFile {
    pub filename: String,
    pub language: String,
}

impl Default for PerhabsConfig {
    fn default() -> Self {
        debug!("Getting Perhabs config: falling back to default.");
        Self {
            excconfig_path_disk: String::from("./appdata/config.json"),
            excconfig_path_web: String::from("https://www.polyprax.nl/perhabs/appdata/config.json"),
            sentences_path_disk: String::from("./excdata/sentences/"),
            sentences_path_web: String::from("https://www.polyprax.nl/perhabs/excdata/sentences/"),
            sentences_files: vec![SentenceFile {
                filename: String::from("sentences_EN.txt"),
                language: String::from("English"),
            }],
            source: Source::Default,
        }
    }
}

impl PerhabsConfig {
    pub fn new() -> Self {
        debug!("Getting Perhabs config.");
        if let Ok(mut res) = PerhabsConfig::from_disk() {
            res.source = Source::Disk;
            return res;
        }

        if let Ok(mut res) = PerhabsConfig::from_web() {
            res.source = Source::Web;
            return res;
        } else {
            PerhabsConfig::default()
        }
    }

    fn from_disk() -> io::Result<Self> {
        debug!("Getting Perhabs config: trying disk.");
        let file = File::open("./appdata/config.json")?;
        let reader = BufReader::new(file);
        let mut de = serde_json::Deserializer::from_reader(reader);
        let config = PerhabsConfig::deserialize(&mut de)?;
        Ok(config)
    }

    fn from_web() -> Result<Self, reqwest::Error> {
        debug!("Getting Perhabs config: trying web.");
        let resp = reqwest::blocking::get("https://www.polyprax.nl/perhabs/appdata/config.json")?;
        let result: PerhabsConfig = resp.json()?;
        Ok(result)
    }
}

/// ExcerciseConfig
/// The ExcerciseConfig struct finds the most relevant config source using new().
///
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

impl Default for ExcConfig {
    fn default() -> Self {
        debug!("Getting excercise config: falling back to default.");
        ExcConfig {
            exercises: vec![
                Exercise {
                    name: String::from("Convergence"),
                    levels: vec![Level {
                        name: String::from("Easy"),
                        params: Parameters { step: 1 },
                    }],
                },
                Exercise {
                    name: String::from("Divergence"),
                    levels: vec![Level {
                        name: String::from("Easy"),
                        params: Parameters { step: -1 },
                    }],
                },
            ],
        }
    }
}

impl ExcConfig {
    pub fn new() -> Self {
        debug!("Getting excercise config.");
        let config = PerhabsConfig::new();
        if let Ok(res) = ExcConfig::from_disk(&config.excconfig_path_disk) {
            return res;
        }
        if let Ok(res) = ExcConfig::from_web(&config.excconfig_path_web) {
            return res;
        } else {
            ExcConfig::default()
        }
    }
    fn from_disk(path: &String) -> io::Result<ExcConfig> {
        debug!("Getting excercise config: trying disk.");
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut de = serde_json::Deserializer::from_reader(reader);
        let config = ExcConfig::deserialize(&mut de)?;
        Ok(config)
    }
    fn from_web(path: &String) -> Result<ExcConfig, reqwest::Error> {
        debug!("Getting excercise config: trying web.");
        let resp = reqwest::blocking::get(path)?;
        let result: ExcConfig = resp.json()?;
        Ok(result)
    }
}

/// Sentences
/// get_sentences finds the most relevant source for practice sentences.
///
pub fn get_sentences(filename: &String) -> Result<Vec<String>, Error> {
    debug!("Getting sentences.");
    let config = PerhabsConfig::new();
    let path_disk = config.sentences_path_disk + &filename;
    let path_web = config.sentences_path_web + &filename;
    if let Ok(res) = get_sentences_disk(&path_disk) {
        return Ok(res);
    }
    if let Ok(res) = get_sentences_web(&path_web) {
        return Ok(res);
    } else {
        return Ok(default_sentences());
    }
}

fn get_sentences_disk(path: &String) -> io::Result<Vec<String>> {
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

fn get_sentences_web(path: &String) -> Result<Vec<String>, reqwest::Error> {
    debug!("Getting sentences: trying web.");
    let resp = reqwest::blocking::get(path)?;
    let result = BufReader::new(resp);
    let mut contents = vec![];
    for line in result.lines() {
        if let Ok(ip) = line {
            contents.push(ip);
        }
    }
    Ok(contents)
}

fn default_sentences() -> Vec<String> {
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
