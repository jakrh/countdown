/// Compute CSS styles for the timer display based on blinking and visibility state
pub fn compute_timer_style(is_blinking: bool, is_visible: bool) -> String {
    // Base style: pointer cursor and disable text selection
    let base = "cursor: pointer; user-select: none;";
    // Color: red when blinking
    let color = if is_blinking { "color: red;" } else { "" };
    // Visibility: hidden only during blink off
    let visibility = if is_blinking && !is_visible {
        "visibility: hidden;"
    } else {
        "visibility: visible;"
    };
    format!("{} {} {}", base, color, visibility)
}

#[cfg(test)]
mod tests {
    use super::compute_timer_style;

    #[test]
    fn test_default_style() {
        let style = compute_timer_style(false, true);
        assert!(style.contains("cursor: pointer;"));
        assert!(style.contains("user-select: none;"));
        assert!(style.contains("visibility: visible;"));
    }

    #[test]
    fn test_blink_visible() {
        let style = compute_timer_style(true, true);
        assert!(style.contains("color: red;"));
        assert!(style.contains("visibility: visible;"));
    }

    #[test]
    fn test_blink_hidden() {
        let style = compute_timer_style(true, false);
        assert!(style.contains("color: red;"));
        assert!(style.contains("visibility: hidden;"));
    }
}
