use crate::config::INITIAL_SECONDS;

/// Result of a click event on the timer display
pub struct ClickResult {
    /// Remaining time after reset
    pub reset_remaining: i32,
    /// Whether the blink timer should be canceled
    pub should_cancel_blink: bool,
    /// New blinking state
    pub is_blinking: bool,
    /// New visibility state
    pub is_visible: bool,
}

/// Handle click: always reset timer; if currently blinking, cancel blinking
pub fn handle_click(
    _current_remaining: i32,
    is_blinking: bool,
    reset_time: Option<i32>,
) -> ClickResult {
    ClickResult {
        reset_remaining: reset_time.unwrap_or(INITIAL_SECONDS),
        should_cancel_blink: is_blinking,
        is_blinking: false,
        is_visible: true,
    }
}

/// Format time input with automatic colon insertion
pub fn format_time_input(value: &str) -> String {
    // Remove any non-digit characters
    let digits: String = value.chars().filter(|c| c.is_digit(10)).collect();

    match digits.len() {
        0 => String::new(),
        1 => digits,
        2 => digits,
        3 => format!("{}:{}", &digits[0..1], &digits[1..3]),
        4 => format!("{}:{}", &digits[0..2], &digits[2..4]),
        _ => {
            // If more than 4 digits, truncate to keep only the first 4
            let truncated = &digits[0..4];
            format!("{}:{}", &truncated[0..2], &truncated[2..4])
        }
    }
}

// Function to validate time format and convert to seconds
pub fn parse_time_input(input: &str) -> Result<i32, String> {
    // Split on colon
    let parts: Vec<&str> = input.trim().split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid format. Please use MM:SS".to_string());
    }

    // Parse minutes and seconds
    let minutes = match parts[0].parse::<i32>() {
        Ok(m) => m,
        Err(_) => return Err("Minutes must be a valid number".to_string()),
    };

    let seconds = match parts[1].parse::<i32>() {
        Ok(s) => s,
        Err(_) => return Err("Seconds must be a valid number".to_string()),
    };

    // Validate ranges
    if seconds < 0 || seconds > 59 {
        return Err("Seconds must be greater than or equal to 0 and less than 60".to_string());
    }

    // Limit total time to reasonable range (e.g., 59:59 max)
    if minutes < 0 || minutes > 59 {
        return Err("Minutes must be greater than or equal to 0 and less than 60".to_string());
    }

    // Convert to total seconds
    Ok(minutes * 60 + seconds)
}

#[cfg(test)]
mod tests {
    use super::{format_time_input, handle_click, parse_time_input};
    use crate::config::INITIAL_SECONDS;

    #[test]
    fn click_when_not_blinking() {
        let result = handle_click(42, false, None);
        assert_eq!(result.reset_remaining, INITIAL_SECONDS);
        assert_eq!(result.should_cancel_blink, false);
        assert_eq!(result.is_blinking, false);
        assert_eq!(result.is_visible, true);
    }

    #[test]
    fn click_when_blinking() {
        let result = handle_click(0, true, None);
        assert_eq!(result.reset_remaining, INITIAL_SECONDS);
        assert_eq!(result.should_cancel_blink, true);
        assert_eq!(result.is_blinking, false);
        assert_eq!(result.is_visible, true);
    }

    #[test]
    fn test_parse_time_input_valid_zero() {
        assert_eq!(parse_time_input("00:00"), Ok(0));
    }

    #[test]
    fn test_parse_time_input_valid_regular() {
        assert_eq!(parse_time_input("12:34"), Ok(12 * 60 + 34));
    }

    #[test]
    fn test_parse_time_input_valid_leading_zeros() {
        assert_eq!(parse_time_input("05:07"), Ok(5 * 60 + 7));
    }

    #[test]
    fn test_parse_time_input_valid_with_whitespace() {
        assert_eq!(parse_time_input("  07:08  "), Ok(7 * 60 + 8));
    }

    #[test]
    fn test_parse_time_input_err_missing_colon() {
        assert_eq!(
            parse_time_input("1234"),
            Err("Invalid format. Please use MM:SS".to_string())
        );
    }

    #[test]
    fn test_parse_time_input_err_empty() {
        assert_eq!(
            parse_time_input(""),
            Err("Invalid format. Please use MM:SS".to_string())
        );
    }

    #[test]
    fn test_parse_time_input_err_multiple_colons() {
        assert_eq!(
            parse_time_input("01:02:03"),
            Err("Invalid format. Please use MM:SS".to_string())
        );
    }

    #[test]
    fn test_parse_time_input_err_non_numeric_minutes() {
        assert_eq!(
            parse_time_input("ab:12"),
            Err("Minutes must be a valid number".to_string())
        );
    }

    #[test]
    fn test_parse_time_input_err_non_numeric_seconds() {
        assert_eq!(
            parse_time_input("12:xy"),
            Err("Seconds must be a valid number".to_string())
        );
    }

    #[test]
    fn test_parse_time_input_err_seconds_too_large() {
        assert_eq!(
            parse_time_input("10:60"),
            Err("Seconds must be greater than or equal to 0 and less than 60".to_string())
        );
        assert_eq!(
            parse_time_input("10:99"),
            Err("Seconds must be greater than or equal to 0 and less than 60".to_string())
        );
    }

    #[test]
    fn test_parse_time_input_err_minutes_too_large() {
        assert_eq!(
            parse_time_input("60:00"),
            Err("Minutes must be greater than or equal to 0 and less than 60".to_string())
        );
        assert_eq!(
            parse_time_input("99:59"),
            Err("Minutes must be greater than or equal to 0 and less than 60".to_string())
        );
    }

    #[test]
    fn test_parse_time_input_negative_seconds() {
        assert_eq!(
            parse_time_input("-01:30"),
            Err("Minutes must be greater than or equal to 0 and less than 60".to_string())
        );
        assert_eq!(
            parse_time_input("01:-30"),
            Err("Seconds must be greater than or equal to 0 and less than 60".to_string())
        );
    }

    #[test]
    fn test_format_time_input_empty() {
        assert_eq!(format_time_input(""), "");
    }

    #[test]
    fn test_format_time_input_single_digit() {
        assert_eq!(format_time_input("5"), "5");
    }

    #[test]
    fn test_format_time_input_two_digits() {
        assert_eq!(format_time_input("25"), "25");
    }

    #[test]
    fn test_format_time_input_three_digits() {
        assert_eq!(format_time_input("123"), "1:23");
    }

    #[test]
    fn test_format_time_input_four_digits() {
        assert_eq!(format_time_input("2530"), "25:30");
    }

    #[test]
    fn test_format_time_input_more_than_four_digits() {
        assert_eq!(format_time_input("123456"), "12:34");
    }

    #[test]
    fn test_format_time_input_already_has_colon() {
        assert_eq!(format_time_input("25:30"), "25:30");
    }

    #[test]
    fn test_format_time_input_ignores_non_digits() {
        assert_eq!(format_time_input("2a5:3b0"), "25:30");
    }
}
