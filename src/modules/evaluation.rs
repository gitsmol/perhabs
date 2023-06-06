use super::timer::Timer;
use chrono::{Duration, Local};

/// Manage a performance evaluation by keeping track of time and/or reps
/// and storing results.
pub struct Evaluation<T> {
    start_time: chrono::DateTime<Local>,
    end_time: Option<chrono::DateTime<Local>>,
    duration: Duration,
    repetitions: usize,
    timer: Timer,
    results: Vec<T>,
}

impl<T> Evaluation<T> {
    pub fn new(duration: Duration, repetitions: usize) -> Self {
        Self {
            start_time: chrono::Local::now(),
            end_time: None,
            duration,
            repetitions,
            timer: Timer::new(),
            results: vec![],
        }
    }

    /// Start evaluation
    pub fn start(&mut self) {
        self.start_time = chrono::Local::now();
        self.timer.set(self.duration);
    }

    /// Add result of type T
    pub fn add_result(&mut self, result: T) {
        self.results.push(result);
    }

    /// What is the remaining duration?
    pub fn time_remaining(&self) -> Duration {
        self.timer.remaining()
    }

    /// How much time did we take?
    /// Returns None if not finished.
    pub fn time_taken(&self) -> Option<Duration> {
        if let Some(end_time) = self.end_time {
            Some(end_time - self.start_time)
        } else {
            None
        }
    }

    /// How much time did we take?
    /// Returns a formatted string:
    /// 1 minute and 9 seconds = "1:09"
    /// Returns string "NaN" if not yet finished.
    pub fn time_taken_as_string(&self) -> String {
        if let Some(duration) = self.time_taken() {
            String::from(format!(
                "{}:{:02}",
                duration.num_minutes(),
                duration.num_seconds()
            ))
        } else {
            String::from("NaN")
        }
    }

    /// How many reps have we done?
    pub fn reps_done(&self) -> usize {
        self.results.len()
    }

    /// Number of seconds taken per rep.
    /// Returns None if not finished.
    pub fn average_secs_per_rep(&self) -> Option<f32> {
        match self.time_taken() {
            Some(duration) => Some(duration.num_seconds() as f32 / self.reps_done() as f32),
            None => None,
        }
    }

    /// How many reps are remaining?
    pub fn reps_remaining(&self) -> usize {
        self.repetitions - self.results.len()
    }

    /// Return a vec of all results.
    pub fn show_results(&self) -> &Vec<T> {
        &self.results
    }

    /// Are we done?
    pub fn is_finished(&mut self) -> bool {
        // Amount of reps done?
        if self.results.len() >= self.repetitions {
            self.end_time = Some(chrono::Local::now());
            return true;
        };
        // Time up?
        if self.timer.is_finished() {
            self.end_time = Some(chrono::Local::now());
            return true;
        };
        // default: we are not finished
        false
    }
}
