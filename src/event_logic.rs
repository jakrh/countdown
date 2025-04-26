use crate::config::INITIAL_SECONDS;

/// Result of a click event on the timer display
pub struct ClickResult {
    /// Remaining time after reset
    pub reset_remaining: u32,
    /// Whether the blink timer should be canceled
    pub should_cancel_blink: bool,
    /// New blinking state
    pub is_blinking: bool,
    /// New visibility state
    pub is_visible: bool,
}

/// Handle click: always reset timer; if currently blinking, cancel blinking
pub fn handle_click(_current_remaining: u32, is_blinking: bool) -> ClickResult {
    ClickResult {
        reset_remaining: INITIAL_SECONDS,
        should_cancel_blink: is_blinking,
        is_blinking: false,
        is_visible: true,
    }
}

#[cfg(test)]
mod tests {
    use super::handle_click;
    use crate::config::INITIAL_SECONDS;

    #[test]
    fn click_when_not_blinking() {
        let result = handle_click(42, false);
        assert_eq!(result.reset_remaining, INITIAL_SECONDS);
        assert_eq!(result.should_cancel_blink, false);
        assert_eq!(result.is_blinking, false);
        assert_eq!(result.is_visible, true);
    }

    #[test]
    fn click_when_blinking() {
        let result = handle_click(0, true);
        assert_eq!(result.reset_remaining, INITIAL_SECONDS);
        assert_eq!(result.should_cancel_blink, true);
        assert_eq!(result.is_blinking, false);
        assert_eq!(result.is_visible, true);
    }
}
