use serde::{Deserialize, Serialize};

use super::ExerciseConfig;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VergenceConfig {
    pub name: String,
    pub step: isize,
    pub pixel_size: isize,
}

impl ExerciseConfig for VergenceConfig {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
