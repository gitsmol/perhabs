use serde::Deserialize;

// Exercise config model
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
