/// Compute CSS styles for the timer display based on blinking and visibility state
pub fn compute_timer_style(is_blinking: bool, is_visible: bool, is_paused: bool) -> String {
    // Base style: pointer cursor and disable text selection
    let base: &str = "cursor: pointer; user-select: none;";
    // Color: darkturquoise when paused, red when blinking, white otherwise
    let color = if is_paused {
        "color: darkturquoise;"
    } else if is_blinking {
        "color: red;"
    } else {
        "color: white;"
    };
    // Opacity: invisible only during blink off, but still clickable
    let opacity = if is_blinking && !is_visible {
        "opacity: 0;" // Invisible
    } else {
        "opacity: 1;" // Visible
    };
    format!("{} {} {}", base, color, opacity)
}

#[cfg(test)]
mod tests {
    use super::compute_timer_style;

    #[test]
    fn test_default_style() {
        let style = compute_timer_style(false, true, false);
        assert!(style.contains("cursor: pointer;"));
        assert!(style.contains("user-select: none;"));
        assert!(style.contains("color: white;"));
        assert!(style.contains("opacity: 1;"));
    }

    #[test]
    fn test_blink_visible() {
        let style = compute_timer_style(true, true, false);
        assert!(style.contains("color: red;"));
        assert!(style.contains("opacity: 1;"));
    }

    #[test]
    fn test_blink_hidden() {
        let style = compute_timer_style(true, false, false);
        assert!(style.contains("color: red;"));
        assert!(style.contains("opacity: 0;"));
    }

    #[test]
    fn test_paused_style() {
        let style = compute_timer_style(false, true, true);
        assert!(style.contains("color: darkturquoise;"));
        assert!(style.contains("opacity: 1;"));
    }
}
