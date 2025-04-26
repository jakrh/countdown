/// Pure logic for countdown updates and blink toggling

/// Result of a countdown update step
pub struct CountdownUpdate {
    pub remaining: u32,
    pub should_blink: bool,
}

/// Update countdown: decrease remaining or signal blink start when zero
pub fn update_countdown(remaining: u32) -> CountdownUpdate {
    if remaining > 0 {
        CountdownUpdate {
            remaining: remaining - 1,
            should_blink: false,
        }
    } else {
        CountdownUpdate {
            remaining: 0,
            should_blink: true,
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
    }

    #[test]
    fn test_update_countdown_blink_signal() {
        let result = update_countdown(0);
        assert_eq!(result.remaining, 0);
        assert!(result.should_blink);
    }

    #[test]
    fn test_toggle_blink() {
        assert!(toggle_blink(false));
        assert!(!toggle_blink(true));
    }
}
