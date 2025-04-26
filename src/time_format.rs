/// Utility functions for the countdown app

/// Format total seconds into "MM:SS" string
pub fn format_time(total_secs: u32) -> String {
    if total_secs == 0 {
        return "00:00".to_string();
    }
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::format_time;

    #[test]
    fn test_zero() {
        assert_eq!(format_time(0), "00:00");
    }

    #[test]
    fn test_less_than_minute() {
        assert_eq!(format_time(5), "00:05");
        assert_eq!(format_time(59), "00:59");
    }

    #[test]
    fn test_exact_minutes() {
        assert_eq!(format_time(60), "01:00");
        assert_eq!(format_time(150), "02:30");
    }

    #[test]
    fn test_large() {
        assert_eq!(format_time(3600), "60:00");
    }
}
