use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use ehttp::{Response, Result};
use poll_promise::Promise;
use rand::prelude::*;

// The sentences and all config go here
pub struct Sentences {
    pub promise: Option<Promise<Result<Response>>>,
    pub selected_file: Option<SentenceFile>,
    pub contents: Option<Vec<String>>,
}

impl Default for Sentences {
    fn default() -> Self {
        Self {
            promise: None,
            selected_file: None,
            contents: None,
        }
    }
}

impl Sentences {
    /// Shuffle the file contents vec using the Fisher-Yates shuffle algorithm.
    pub fn shuffle_contents(&mut self) {
        if let Some(contents) = &mut self.contents {
            let length = contents.len();
            let mut rng = thread_rng();
            for i in 0..length {
                let j = rng.gen_range(i..length);
                let tmp = contents[i].clone();
                contents[i] = contents[j].clone();
                contents[j] = tmp;
            }
        }
    }
}

/// Sentences
/// get_sentences finds the most relevant source for practice sentences.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct SentenceFile {
    pub filename: String,
    pub language: String,
}

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
