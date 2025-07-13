/// Pure logic for countdown updates and blink toggling
use crate::config::MINIMUM_SECONDS;

/// Result of a countdown update step
pub struct CountdownUpdate {
    pub remaining: i32,
    pub should_blink: bool,

    /// Whether to stop the countdown at MINIMUM_SECONDS
    pub should_stop: bool,
}

/// Update countdown: decrease remaining, signal blink start when zero, stop at MINIMUM_SECONDS
pub fn update_countdown(remaining: i32) -> CountdownUpdate {
    if remaining > 0 {
        CountdownUpdate {
            remaining: remaining - 1,
            should_blink: false,
            should_stop: false,
        }
    } else if remaining == 0 {
        CountdownUpdate {
            remaining: -1, // Start negative countdown
            should_blink: true,
            should_stop: false,
        }
    } else if remaining > MINIMUM_SECONDS {
        CountdownUpdate {
            remaining: remaining - 1,
            should_blink: true,
            should_stop: false,
        }
    } else {
        // Stop at MINIMUM_SECONDS
        CountdownUpdate {
            remaining: MINIMUM_SECONDS,
            should_blink: true,
            should_stop: true,
        }
    }
}

/// Toggle blink visibility
pub fn toggle_blink(visible: bool) -> bool {
    !visible
}

#[cfg(test)]
mod tests {
    use super::{toggle_blink, update_countdown};

    #[test]
    fn test_update_countdown_decrements() {
        let result = update_countdown(10);
        assert_eq!(result.remaining, 9);
        assert!(!result.should_blink);
        assert!(!result.should_stop);
    }

    #[test]
    fn test_update_countdown_blink_signal() {
        let result = update_countdown(0);
        assert_eq!(result.remaining, -1);
        assert!(result.should_blink);
        assert!(!result.should_stop);
    }

    #[test]
    fn test_update_countdown_negative() {
        let result = update_countdown(-10);
        assert_eq!(result.remaining, -11);
        assert!(result.should_blink);
        assert!(!result.should_stop);
    }

    #[test]
    fn test_update_countdown_stop_at_limit() {
        let result = update_countdown(-3599);
        assert_eq!(result.remaining, -3599);
        assert!(result.should_blink);
        assert!(result.should_stop);
    }

    #[test]
    fn test_toggle_blink() {
        assert!(toggle_blink(false));
        assert!(!toggle_blink(true));
    }
}
