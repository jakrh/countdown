/// Application configuration constants

/// 25:00 in seconds for the initial countdown
pub const INITIAL_SECONDS: i32 = 25 * 60;

/// -59:59 in seconds for the stop condition
pub const MINIMUM_SECONDS: i32 = -3599;

/// 1 second in milliseconds for countdown updates
pub const COUNTDOWN_INTERVAL_MS: u32 = 1000;

/// 500 milliseconds in milliseconds for blink toggling
pub const BLINK_INTERVAL_MS: u32 = 500;
