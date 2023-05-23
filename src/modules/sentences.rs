use super::asset_loader::SentenceFile;
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
