// src/time.rs

use chrono::{DateTime, Duration, Utc};

/// Parses a date string into a DateTime object.
/// Tries several common formats.
fn parse_date_string(date_str: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(date_str)
        .or_else(|_| DateTime::parse_from_rfc2822(date_str))
        .or_else(|_| DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S %z"))
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

/// Calculates the difference between two date strings and returns a human-readable string.
pub fn time_diff(start_str: &str, end_str: &str) -> String {
    if let (Some(start), Some(end)) = (parse_date_string(start_str), parse_date_string(end_str)) {
        let duration = end.signed_duration_since(start);
        format_duration(duration)
    } else {
        "Invalid date format".to_string()
    }
}

/// Formats a duration into a human-readable string.
fn format_duration(duration: Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }
    parts.join(" ")
}

/// Converts a date string into a human-readable "time ago" string.
pub fn human_date(date_str: &str) -> String {
    if let Some(date) = parse_date_string(date_str) {
        let now = Utc::now();
        let duration = now.signed_duration_since(date);
        let formatted_duration = format_duration(duration);
        if duration > Duration::zero() {
            format!("{} ago", formatted_duration)
        } else {
            format!("in {}", format_duration(duration.abs()))
        }
    } else {
        "Invalid date format".to_string()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_diff() {
        let start = "2025-01-01T12:00:00Z";
        let end = "2025-01-02T13:01:02Z";
        // 1d 1h 1m 2s
        let diff = time_diff(start, end);
        assert!(diff.contains("1d"));
        assert!(diff.contains("1h"));
        assert!(diff.contains("1m"));
        assert!(diff.contains("2s"));
    }

    #[test]
    fn test_human_date() {
        let now = Utc::now();
        let five_minutes_ago = now - Duration::minutes(5);
        let date_str = five_minutes_ago.to_rfc3339();
        let human = human_date(&date_str);
        assert!(human.contains("5m") && human.contains("ago"));
    }
}
