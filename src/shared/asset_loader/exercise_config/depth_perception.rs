use serde::{Deserialize, Serialize};

use super::ExerciseConfig;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DepthPerceptionConfig {
    pub name: String,
    pub circle_size: usize,
    pub offset_min: usize,
    pub offset_max: usize,
    pub offset_target_variance_min: usize,
    pub offset_target_variance_max: usize,
}

impl Default for DepthPerceptionConfig {
    fn default() -> Self {
        Self {
            name: String::from("Default"),
            circle_size: 3,
            offset_min: 1,
            offset_max: 3,
            offset_target_variance_min: 2,
            offset_target_variance_max: 5,
        }
    }
}

impl ExerciseConfig for DepthPerceptionConfig {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
