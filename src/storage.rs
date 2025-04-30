use crate::config::INITIAL_SECONDS;

const TIMER_REMAINING_SECONDS_KEY: &str = "countdown_timer_remaining_seconds";

/// Read remaining seconds from LocalStorage
/// If no stored value is found or an error occurs, returns the default value INITIAL_SECONDS
pub fn load_remaining_seconds() -> u32 {
    // Try to get the value from localStorage
    if let Some(storage) = get_local_storage() {
        if let Ok(Some(value)) = storage.get_item(TIMER_REMAINING_SECONDS_KEY) {
            if let Ok(seconds) = value.parse::<u32>() {
                return seconds;
            }
        }
    }

    // Return default if anything fails
    INITIAL_SECONDS
}

/// Save remaining seconds to LocalStorage
pub fn save_remaining_seconds(seconds: u32) {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item(TIMER_REMAINING_SECONDS_KEY, &seconds.to_string());
    }
}

/// Get LocalStorage instance
fn get_local_storage() -> Option<web_sys::Storage> {
    match web_sys::window() {
        Some(window) => match window.local_storage() {
            Ok(Some(storage)) => Some(storage),
            _ => None,
        },
        None => None,
    }
}
