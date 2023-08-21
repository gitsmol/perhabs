mod cog_numbers;
mod cog_words;
mod depth_perception;
mod episodic_memory;
mod seq_numbers;
pub mod spatial_drawing;
mod spatial_hearing;
mod vergence;
mod visual_recognition;
mod visual_saccades;

pub use cog_numbers::CogNumbers;
pub use cog_words::CogWords;
pub use depth_perception::DepthPerception;
pub use episodic_memory::EpisodicMemory;
pub use seq_numbers::NumSeq;
pub use spatial_drawing::SpatialDrawing;
pub use spatial_hearing::SpatialHearing;
pub use vergence::Vergence;
pub use visual_recognition::VisRecognition;
pub use visual_saccades::VisSaccades;

//  **********
// Some basic supporting stuff
// ***********

/// Enum to keep track of different stages of an exercise.
/// None,      There is no excercise going on.
/// Challenge  The challenge that prompts the user to do something
/// Response   The user has a change to respond
/// Result,    The user has responded, challenge response can be compared
/// Finished,  The exercise is over.
#[derive(Debug, PartialEq)]
enum ExerciseStatus {
    None,
    Challenge,
    Response,
    Result,
    Finished,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Turn a vector of numbers into a string, separating the numbers by a comma
pub fn numvec_to_string(seq: &Vec<u32>) -> String {
    let mut result = String::new();
    for i in seq {
        result += &i.to_string();
        result += ", ";
    }
    result.trim_end_matches(", ").to_string()
}
