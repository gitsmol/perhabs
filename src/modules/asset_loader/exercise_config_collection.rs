use crate::exercises::spatial_drawing::painters::SpatialPuzzle;
use crate::exercises::visual_recognition::VisRecognitionExercise;
use ehttp::{Response, Result};
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use std::{
    fs::File,
    io::{self, BufReader},
};

use super::AssetSource;

/// ExcerciseConfigCollection
/// The ExcerciseConfigCollection struct finds the most relevant config source using new().
///
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExerciseConfigCollection {
    pub source: AssetSource,
    pub vergence: Vec<VergenceExercise>,
    pub spatial_drawing: Vec<SpatialPuzzle>,
    pub visual_recognition: Vec<VisRecognitionExercise>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VergenceExercise {
    pub name: String,
    pub levels: Vec<Level>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Level {
    pub name: String,
    pub step: isize,
    pub pixel_size: isize,
}

impl Default for ExerciseConfigCollection {
    fn default() -> Self {
        debug!("Getting excercise config: falling back to default.");
        ExerciseConfigCollection {
            source: AssetSource::Default,
            vergence: vec![
                VergenceExercise {
                    name: String::from("Convergence"),
                    levels: vec![Level {
                        name: String::from("Easy"),
                        step: 1,
                        pixel_size: 3,
                    }],
                },
                VergenceExercise {
                    name: String::from("Divergence"),
                    levels: vec![Level {
                        name: String::from("Easy"),
                        step: -1,
                        pixel_size: 3,
                    }],
                },
            ],
            spatial_drawing: vec![SpatialPuzzle::new(5)],
            visual_recognition: vec![VisRecognitionExercise::default()],
        }
    }
}

impl ExerciseConfigCollection {
    pub fn from_disk(path: &String) -> io::Result<ExerciseConfigCollection> {
        debug!("Getting excercise config: trying disk.");
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut de = serde_json::Deserializer::from_reader(reader);
        let config = ExerciseConfigCollection::deserialize(&mut de)?;
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
