use chrono::{Duration, Local, NaiveTime};

/// A simple timer based on local time differences.
pub struct Timer {
    start_time: NaiveTime,
    end_time: NaiveTime,
    duration: Duration,
}

impl Timer {
    /// Set a timer for a given duration, starting now.
    pub fn new(duration: Duration) -> Self {
        let now = Local::now().time();
        Self {
            start_time: now,
            end_time: now + duration,
            duration,
        }
    }

    /// Returns true when timer is finished.
    pub fn is_finished(&self) -> bool {
        let now = Local::now().time();
        if self.end_time > now {
            true
        } else {
            false
        }
    }

    /// Return the time remaining until the timer is finished.
    pub fn remaining(&self) -> Duration {
        let now = Local::now().time();
        self.end_time - now
    }
}
