use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VergenceConfig {
    pub name: String,
    pub levels: Vec<Level>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Level {
    pub name: String,
    pub step: isize,
    pub pixel_size: isize,
}
