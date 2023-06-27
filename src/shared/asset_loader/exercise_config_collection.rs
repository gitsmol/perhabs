use crate::{
    exercises::spatial_drawing::painters::SpatialPuzzle,
    shared::asset_loader::exercise_config::vergence::Level,
};
use ehttp::{Response, Result};
use log::debug;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use std::{
    fs::File,
    io::{self, BufReader},
};

use super::{
    exercise_config::{
        depth_perception::DepthPerceptionConfig, vergence::VergenceConfig,
        visual_recognition::VisRecognitionConfig, visual_saccades::VisSaccadesConfig,
    },
    AssetSource,
};

/// ExcerciseConfigCollection
/// The ExcerciseConfigCollection struct finds the most relevant config source using new().
///
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExerciseConfigCollection {
    pub source: AssetSource,
    pub vergence: Vec<VergenceConfig>,
    pub spatial_drawing: Vec<SpatialPuzzle>,
    pub visual_recognition: Vec<VisRecognitionConfig>,
    pub visual_saccades: Vec<VisSaccadesConfig>,
    pub depth_perception: Vec<DepthPerceptionConfig>,
}

impl Default for ExerciseConfigCollection {
    fn default() -> Self {
        debug!("Getting excercise config: falling back to default.");
        ExerciseConfigCollection {
            source: AssetSource::Default,
            vergence: vec![
                VergenceConfig {
                    name: String::from("Convergence"),
                    levels: vec![Level {
                        name: String::from("Easy"),
                        step: 1,
                        pixel_size: 3,
                    }],
                },
                VergenceConfig {
                    name: String::from("Divergence"),
                    levels: vec![Level {
                        name: String::from("Easy"),
                        step: -1,
                        pixel_size: 3,
                    }],
                },
            ],
            spatial_drawing: vec![SpatialPuzzle::new(5)],
            visual_recognition: vec![VisRecognitionConfig::default()],
            visual_saccades: vec![VisSaccadesConfig::default()],
            depth_perception: vec![DepthPerceptionConfig::default()],
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
