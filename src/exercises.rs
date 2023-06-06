pub mod cog_numbers;
pub mod cog_words;
pub mod episodic_memory;
pub mod seq_numbers;
pub mod spatial_drawing;
pub mod vergence;
pub mod visual_recognition;
pub mod visual_saccades;

#[derive(Debug, PartialEq)]
enum SessionStatus {
    None,
    Answer,
    Response,
    Result,
    Finished,
}
