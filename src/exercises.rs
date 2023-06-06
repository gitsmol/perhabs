pub mod cog_numbers;
pub mod cog_words;
pub mod episodic_memory;
pub mod seq_numbers;
pub mod spatial_drawing;
pub mod vergence;
pub mod visual_recognition;
pub mod visual_saccades;

//  **********
// Some basic supporting stuff
// ***********

#[derive(Debug, PartialEq)]
enum SessionStatus {
    None,
    Answer,
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
