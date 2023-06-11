use chrono::Duration;

/// Format a duration:
/// 1 minute and 9 seconds = "1:09"
pub fn format_min_secs(duration: &Duration) -> String {
    let mins = duration.num_minutes();
    let secs = duration.num_seconds() - (mins * 60);
    format!("{}:{:02}", mins, secs)
}
