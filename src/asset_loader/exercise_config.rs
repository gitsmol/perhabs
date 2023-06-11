pub mod vergence;
pub mod visual_recognition;
pub mod visual_saccades;

pub trait ExerciseConfig {
    fn name(&self) -> &str;
}
