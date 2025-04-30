use crate::event_logic::{format_time_input, handle_click, parse_time_input};
use crate::storage::save_remaining_seconds;
use crate::time_format::format_time;
use crate::timer_provider::{TimerHandle, TimerProvider};
use crate::timer_service::start_countdown_timer;
use std::{cell::RefCell, rc::Rc};
use sycamore::prelude::Signal;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};

/// Handle input events and format timer input with automatic colon insertion
pub fn handle_timer_input(event: web_sys::Event, input_value: Signal<String>) {
    if let Some(target) = event.target() {
        if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
            // Get the raw input value
            let raw_value = input.value();
            // Format the value with automatic colon insertion
            let formatted = format_time_input(&raw_value);
            // Set the formatted value back to the input field
            input_value.set(formatted);
        }
    }
}

/// Helper function to create key event handler for F key
pub fn create_key_handler(
    input_mode: Signal<bool>,
    input_value: Signal<String>,
    remaining_time: Signal<u32>,
) -> impl Fn(KeyboardEvent) + 'static {
    move |ev: KeyboardEvent| {
        handle_toggle_input_mode(
            ev,
            input_mode.clone(),
            input_value.clone(),
            remaining_time.clone(),
        );
    }
}

/// Handle toggle input mode via 'f' key
pub fn handle_toggle_input_mode(
    event: KeyboardEvent,
    input_mode: Signal<bool>,
    input_value: Signal<String>,
    remaining_time: Signal<u32>,
) -> bool {
    if !input_mode.get() && event.key() == "f" {
        event.prevent_default();
        let time_str = format_time(remaining_time.get());
        input_value.set(time_str);
        input_mode.set(true);
        focus_timer_input();
        true
    } else if input_mode.get() && event.key() == "f" {
        event.prevent_default();
        input_mode.set(false);
        true
    } else {
        false
    }
}

/// Handle click event and reset the timer
pub fn handle_click_and_reset(
    provider: Rc<dyn TimerProvider>,
    countdown_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    blink_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    remaining_time: Signal<u32>,
    blinking_signal: Signal<bool>,
    visible_signal: Signal<bool>,
    reset_time: Signal<Option<u32>>,
) {
    let result = handle_click(
        remaining_time.get(),
        blinking_signal.get(),
        reset_time.get(),
    );
    remaining_time.set(result.reset_remaining);
    if result.should_cancel_blink {
        if let Some(mut handle) = blink_handle.borrow_mut().take() {
            handle.cancel();
        }
    }
    blinking_signal.set(result.is_blinking);
    visible_signal.set(result.is_visible);
    start_countdown_timer(
        provider,
        &countdown_handle,
        &remaining_time,
        &blink_handle,
        &blinking_signal,
        &visible_signal,
    );
}

/// Setup pause/resume key handler on window
pub fn setup_pause_resume_listener(
    timer_provider: Rc<dyn TimerProvider>,
    paused_signal: Signal<bool>,
    countdown_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    blink_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    remaining_time: Signal<u32>,
    blinking_signal: Signal<bool>,
    visible_signal: Signal<bool>,
) {
    let window = web_sys::window().unwrap();
    let paused = paused_signal.clone();
    let provider = timer_provider.clone();
    let countdown_handle = countdown_handle.clone();
    let blink_handle = blink_handle.clone();
    let remaining_time = remaining_time.clone();
    let blinking = blinking_signal.clone();
    let visible = visible_signal.clone();
    let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        if event.key() == "p" {
            if paused.get() {
                paused.set(false);
                if !blinking.get() {
                    start_countdown_timer(
                        provider.clone(),
                        &countdown_handle,
                        &remaining_time,
                        &blink_handle,
                        &blinking,
                        &visible,
                    );
                }
            } else {
                if blinking.get() {
                    return;
                }
                paused.set(true);
                if let Some(mut h) = countdown_handle.borrow_mut().take() {
                    h.cancel();
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    window
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

// Function to focus the timer input field after a short delay
pub fn focus_timer_input() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Use setTimeout to focus after the DOM is updated
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        if let Some(element) = document.get_element_by_id("timer-input") {
            if let Some(input_element) = element.dyn_into::<HtmlInputElement>().ok() {
                let _ = input_element.focus();
                let _ = input_element.select(); // Select all text for easy editing
            }
        }
    }) as Box<dyn FnMut()>);

    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        50, // Short delay to ensure DOM is updated
    );
    closure.forget(); // Prevent closure from being dropped
}

/// Setup input mode key handler (Enter/Escape) on window
pub fn setup_input_mode_listener(
    input_mode: Signal<bool>,
    input_value: Signal<String>,
    input_error: Signal<Option<String>>,
    remaining_time: Signal<u32>,
    timer_provider: Rc<dyn TimerProvider>,
    countdown_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    blink_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    blinking_signal: Signal<bool>,
    visible_signal: Signal<bool>,
    paused_signal: Signal<bool>,
    reset_time: Signal<Option<u32>>,
) {
    let window = web_sys::window().unwrap();

    // Create a listener to track input mode changes
    let input_mode_track = input_mode.clone();

    // Set appropriate listeners whenever input mode changes
    let doc = window.document().unwrap();

    // Set up keyboard event handler
    let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        if input_mode.get() {
            match event.key().as_str() {
                "Enter" => {
                    event.prevent_default();
                    event.stop_propagation();

                    // Avoid reading and modifying signal values simultaneously, first get all needed values
                    let input_str = input_value.with(|v| v.clone());

                    // Use web_sys::console for debugging
                    web_sys::console::log_1(
                        &format!("Enter key pressed, value: {}", input_str).into(),
                    );

                    // Parse time string
                    match parse_time_input(&input_str) {
                        Ok(total_seconds) => {
                            // Clear error
                            input_error.set(None);

                            // Set remaining time and exit input mode
                            remaining_time.set(total_seconds);
                            // Also update reset time for next click reset
                            reset_time.set(Some(total_seconds));
                            input_mode.set(false);

                            // Save the time setting to LocalStorage
                            save_remaining_seconds(total_seconds);

                            // Cancel existing timers
                            let mut cancelled_handles = false;
                            if let Some(mut handle) = countdown_handle.borrow_mut().take() {
                                handle.cancel();
                                cancelled_handles = true;
                            }
                            if let Some(mut handle) = blink_handle.borrow_mut().take() {
                                handle.cancel();
                                cancelled_handles = true;
                            }

                            // Reset blink states
                            blinking_signal.set(false);
                            visible_signal.set(true);

                            // Reset pause state
                            paused_signal.set(false);

                            // Start new countdown
                            if cancelled_handles {
                                // Ensure not reading and modifying signals simultaneously
                                // Use a short delay to ensure all state updates are complete
                                let remaining = remaining_time.clone();
                                let provider = timer_provider.clone();
                                let countdown = countdown_handle.clone();
                                let blink = blink_handle.clone();
                                let is_blinking = blinking_signal.clone();
                                let is_visible = visible_signal.clone();

                                let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                    start_countdown_timer(
                                        provider.clone(),
                                        &countdown,
                                        &remaining,
                                        &blink,
                                        &is_blinking,
                                        &is_visible,
                                    );
                                })
                                    as Box<dyn FnMut()>);

                                let _ = web_sys::window()
                                    .unwrap()
                                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                                        cb.as_ref().unchecked_ref(),
                                        10, // Use 10ms delay to ensure state updates are complete
                                    );
                                cb.forget();
                            } else {
                                // Start timer directly
                                start_countdown_timer(
                                    timer_provider.clone(),
                                    &countdown_handle,
                                    &remaining_time,
                                    &blink_handle,
                                    &blinking_signal,
                                    &visible_signal,
                                );
                            }
                        }
                        Err(msg) => {
                            // Display validation error
                            input_error.set(Some(msg));
                        }
                    }
                }
                "Escape" => {
                    event.prevent_default();
                    event.stop_propagation();

                    // Use web_sys::console for debugging
                    web_sys::console::log_1(&"Escape key pressed!".into());

                    // Exit input mode
                    input_mode.set(false);
                }
                _ => {}
            }
        }
    }) as Box<dyn FnMut(_)>);

    // Use document directly for event listening to ensure it captures all events
    doc.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();

    // Monitor input mode changes, automatically select all text when entering input mode
    let input_mode_effect = input_mode_track.clone();
    let effect_closure = Closure::wrap(Box::new(move || {
        if input_mode_effect.get() {
            // Get input element and focus
            if let Some(doc) = web_sys::window().and_then(|win| win.document()) {
                if let Some(el) = doc.get_element_by_id("timer-input") {
                    if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                        let _ = input.focus();
                        input.select();
                    }
                }
            }
        }
    }) as Box<dyn FnMut()>);

    // Run setTimeout directly without intermediate function
    if input_mode_track.get() {
        let effect_fn = effect_closure.as_ref().unchecked_ref();
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            effect_fn, 50, // Run after 50 milliseconds
        );
    }

    // Keep closure alive
    effect_closure.forget();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{COUNTDOWN_INTERVAL_MS, INITIAL_SECONDS};
    use crate::timer_provider::tests::FakeProvider;
    use crate::timer_service::trigger_blink_timer;
    use std::{cell::RefCell, rc::Rc};
    use sycamore::prelude::*;
    use sycamore::reactive::create_root;

    #[test]
    fn test_handle_click_and_reset_not_blinking() {
        let _ = create_root(|| {
            let fake = FakeProvider::new();
            let calls = fake.calls.clone();
            let provider: Rc<dyn TimerProvider> = Rc::new(fake);
            let countdown_handle = Rc::new(RefCell::new(None));
            let blink_handle = Rc::new(RefCell::new(None));
            let remaining = create_signal(999);
            let blinking = create_signal(false);
            let visible = create_signal(false);

            handle_click_and_reset(
                provider.clone(),
                countdown_handle.clone(),
                blink_handle.clone(),
                remaining.clone(),
                blinking.clone(),
                visible.clone(),
                create_signal(None),
            );

            // Should schedule countdown interval exactly once
            assert_eq!(&*calls.borrow(), &[COUNTDOWN_INTERVAL_MS]);
            // FakeProvider triggers immediately: remaining decremented from INITIAL_SECONDS
            assert_eq!(remaining.get(), INITIAL_SECONDS - 1);
            // Blink state reset
            assert!(!blinking.get());
            assert!(visible.get());
            // No blink handle present
            assert!(blink_handle.borrow().is_none());
        });
    }

    #[test]
    fn test_handle_click_and_reset_blinking() {
        let _ = create_root(|| {
            let fake = FakeProvider::new();
            let calls = fake.calls.clone();
            let provider: Rc<dyn TimerProvider> = Rc::new(fake);
            let countdown_handle = Rc::new(RefCell::new(None));
            let blink_handle = Rc::new(RefCell::new(None));
            let remaining = create_signal(1);
            let blinking = create_signal(false);
            let visible = create_signal(true);
            // simulate blinking started
            trigger_blink_timer(provider.clone(), &blink_handle, &blinking, &visible);
            // reset call history
            calls.borrow_mut().clear();

            // Now clicking should cancel blink and restart countdown
            handle_click_and_reset(
                provider.clone(),
                countdown_handle.clone(),
                blink_handle.clone(),
                remaining.clone(),
                blinking.clone(),
                visible.clone(),
                create_signal(None),
            );

            // Blink cancelled: handle taken
            assert!(blink_handle.borrow().is_none());
            // Blink signal cleared
            assert!(!blinking.get());
            assert!(visible.get());
            // New countdown scheduled
            assert_eq!(&*calls.borrow(), &[COUNTDOWN_INTERVAL_MS]);
            // FakeProvider immediate callback: remaining from INITIAL_SECONDS
            assert_eq!(remaining.get(), INITIAL_SECONDS - 1);
        });
    }
}
