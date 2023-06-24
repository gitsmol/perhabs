use crate::shared::timer::Timer;
use chrono::{Duration, Local};

/// Manage a performance evaluation by keeping track of time and reps and
/// storing results.
///
/// Note: comparing two Evaluation structs only compares the duration and
/// repetitions fields.
pub struct Evaluation<T> {
    start_time: chrono::DateTime<Local>,
    end_time: Option<chrono::DateTime<Local>>,
    pub duration: Duration,
    pub repetitions: usize,
    timer: Timer,
    results: Vec<T>,
}

impl<T> PartialEq for Evaluation<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.duration == other.duration && self.repetitions == other.repetitions {
            true
        } else {
            false
        }
    }
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

    // ***********
    // Controls
    // ***********

    /// Start evaluation
    pub fn start(&mut self) {
        self.end_time = None;
        self.start_time = chrono::Local::now();
        self.timer.set(self.duration);
    }

    /// Set duration from number of seconds.
    pub fn set_duration_secs(&mut self, secs: i64) {
        self.duration = Duration::seconds(secs);
    }

    /// Set required number of repetitions.
    pub fn set_reps(&mut self, reps: usize) {
        self.repetitions = reps;
    }

    /// Add result of type T
    pub fn add_result(&mut self, result: T) {
        self.results.push(result);
    }

    /// Return a vec of all results.
    pub fn show_results(&self) -> &Vec<T> {
        &self.results
    }

    /// Are we done?
    pub fn is_finished(&mut self) -> bool {
        if let Some(_) = self.end_time {
            return true;
        }
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

    // ***********
    // Reporting
    // ***********

    /// Format a duration:
    /// 1 minute and 9 seconds = "1:09"
    fn format_min_secs(&self, duration: &Duration) -> String {
        let mins = duration.num_minutes();
        let secs = duration.num_seconds() - (mins * 60);
        format!("{}:{:02}", mins, secs)
    }

    /// What is the remaining duration?
    pub fn time_remaining(&mut self) -> Duration {
        if self.is_finished() {
            Duration::zero()
        } else {
            self.timer.remaining()
        }
    }

    /// How much time did we take?
    /// Returns a formatted string:
    /// 1 minute and 9 seconds = "1:09"
    /// Returns string "NaN" if not yet finished.
    pub fn time_remaining_as_string(&self) -> String {
        self.format_min_secs(&self.timer.remaining())
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
            self.format_min_secs(&duration)
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
        self.repetitions.saturating_sub(self.reps_done())
    }
}

// ***********
// Calculating scores
// ***********

impl Evaluation<bool> {
    pub fn average_score(&self) -> f32 {
        // Calculate average score.
        let mut total_score = 0.0;
        for r in self.show_results() {
            match r {
                true => total_score += 1.0,
                false => (),
            }
        }
        total_score / self.show_results().len() as f32
    }
}

impl Evaluation<f32> {
    pub fn average_score(&self) -> f32 {
        // Calculate average score.
        let mut total_score = 0.0;
        for r in self.show_results() {
            total_score += r;
        }
        total_score / self.show_results().len() as f32
    }
}

#[cfg(test)]
mod tests {
    use crate::shared::evaluation::Evaluation;
    use chrono::Duration;

    #[test]
    fn average_score_f32() {
        let mut evaluation: Evaluation<f32> = Evaluation::new(Duration::seconds(10), 10);
        evaluation.add_result(0.3);
        evaluation.add_result(1.0);
        evaluation.add_result(0.5);
        let score = evaluation.average_score();
        assert_eq!(score, 0.59999996);
    }

    #[test]
    fn average_score_bool() {
        let mut evaluation: Evaluation<bool> = Evaluation::new(Duration::seconds(10), 10);
        evaluation.add_result(true);
        evaluation.add_result(false);
        evaluation.add_result(false);
        let score = evaluation.average_score();
        assert_eq!(score, 0.33333334);
    }
}
