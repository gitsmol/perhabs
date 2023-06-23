use chrono::{Duration, Local, NaiveTime};

/// A simple timer based on local time differences.
pub struct Timer {
    start_time: NaiveTime,
    end_time: Option<NaiveTime>,
    duration: Duration,
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "Start time: {}\nEnd time: {}\nDuration:{}",
            self.start_time.to_string(),
            self.end_time.unwrap_or_default().to_string(),
            self.duration.to_string()
        );
        write!(f, "{}", s)
    }
}

impl Timer {
    /// Create a timer. Needs to be set to be used.
    pub fn new() -> Self {
        Self {
            start_time: Local::now().time(),
            end_time: None,
            duration: Duration::zero(),
        }
    }

    /// Set a timer for a given duration, starting now.
    pub fn set(&mut self, duration: Duration) {
        let now = Local::now().time();
        self.start_time = now;
        self.end_time = Some(now + duration);
        self.duration = duration;
    }

    /// Stop a running timer
    pub fn reset(&mut self) {
        self.end_time = None;
    }

    /// Returns true when timer is finished.
    pub fn is_finished(&self) -> bool {
        let now = Local::now().time();
        match self.end_time {
            Some(end_time) => {
                if end_time < now {
                    return true;
                } else {
                    return false;
                }
            }
            None => true,
        }
    }

    /// Return the time remaining until the timer is finished.
    pub fn remaining(&self) -> Duration {
        if let Some(end_time) = self.end_time {
            let now = Local::now().time();
            if end_time > now {
                return end_time - now;
            }
        }
        // return zero by default
        chrono::Duration::zero()
    }

    /// Return the time passed since starting the timer.
    /// Returns the full duration of the timer if the timer is finished.
    pub fn time_passed(&self) -> Duration {
        if self.is_finished() {
            self.duration
        } else {
            let now = Local::now().time();
            now - self.start_time
        }
    }
}
