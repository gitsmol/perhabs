use serde::{Deserialize, Serialize};

use crate::modules::asset_loader::exercise_config::ExerciseConfig;

/// Params for a visual recognition exercise.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VisSaccadesExercise {
    pub name: String,
    pub arrow_size: usize,
    pub answer_timeout: i64, // The number of milliseconds the answer is shown
}

impl Default for VisSaccadesExercise {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            arrow_size: 3,
            answer_timeout: 500, // The number of milliseconds the answer is shown
        }
    }
}

impl ExerciseConfig for VisSaccadesExercise {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
