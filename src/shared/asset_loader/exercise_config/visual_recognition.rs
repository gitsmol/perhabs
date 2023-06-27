use serde::{Deserialize, Serialize};

use crate::shared::asset_loader::exercise_config::ExerciseConfig;

/// Params for a visual recognition exercise.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VisRecognitionConfig {
    pub name: String,
    pub num_arrows: usize,
    pub arrow_size: usize,
    pub answer_timeout: i64, // The number of milliseconds the answer is shown
}

impl Default for VisRecognitionConfig {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            num_arrows: 3,
            arrow_size: 3,
            answer_timeout: 500, // The number of milliseconds the answer is shown
        }
    }
}

impl ExerciseConfig for VisRecognitionConfig {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
