use crate::config::INITIAL_SECONDS;
use crate::event_ui::{create_key_handler, setup_input_mode_listener, setup_pause_resume_listener};
use crate::time_format::format_time;
use crate::timer_provider::{GlooTimerProvider, TimerHandle, TimerProvider};
use crate::timer_service::start_countdown_timer;
use crate::view_components::{create_timer_display_view, create_timer_input_view};
use std::cell::RefCell;
use std::rc::Rc;
use sycamore::prelude::*;

#[component]
pub fn App() -> View {
    // instantiate timer provider as Rc<dyn TimerProvider> for sharing across closures
    let timer_provider: Rc<dyn TimerProvider> = Rc::new(GlooTimerProvider);
    // Clone provider for mount event to avoid moving original
    let timer_provider_for_mount = timer_provider.clone();
    // --- Countdown timer state ---
    // Remaining time signal (seconds)
    let remaining_time = create_signal(INITIAL_SECONDS);

    // Custom reset time signal (None = use INITIAL_SECONDS)
    let reset_time = create_signal(None::<u32>);

    // Handle for the main countdown timer
    let countdown_timer_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>> =
        Rc::new(RefCell::new(None));

    // Blink state and timer handle after countdown ends
    let is_blinking_signal = create_signal(false);
    let is_blink_visible_signal = create_signal(true);
    let blink_timer_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>> = Rc::new(RefCell::new(None));

    // --- Format display (MM:SS) ---
    // Use create_memo to recompute only when remaining_time changes
    let formatted_time = create_memo(move || format_time(remaining_time.get()));

    // --- Pause state ---
    // Pause state signal
    let is_paused = create_signal(false);

    // --- Input mode ---
    // true = entering time, false = countdown
    let input_mode = create_signal(false);
    // user input string ("mm:ss")
    let input_value = create_signal("25:00".to_string());
    // validation error message
    let input_error = create_signal(None::<String>);

    // --- Setup timer logic ---
    // Use on_mount to start the timer when the component mounts
    let countdown_handle_for_mount = countdown_timer_handle.clone();
    let blink_handle_for_mount = blink_timer_handle.clone();
    on_mount(move || {
        // start countdown using shared timer provider clone by value
        start_countdown_timer(
            timer_provider_for_mount.clone(),
            &countdown_handle_for_mount,
            &remaining_time,
            &blink_handle_for_mount,
            &is_blinking_signal,
            &is_blink_visible_signal,
        );

        // simplified pause/resume listener setup
        setup_pause_resume_listener(
            timer_provider_for_mount.clone(),
            is_paused.clone(),
            countdown_handle_for_mount.clone(),
            blink_handle_for_mount.clone(),
            remaining_time.clone(),
            is_blinking_signal.clone(),
            is_blink_visible_signal.clone(),
        );

        // Register input mode Enter/Escape listener
        setup_input_mode_listener(
            input_mode.clone(),
            input_value.clone(),
            input_error.clone(),
            remaining_time.clone(),
            timer_provider_for_mount.clone(),
            countdown_handle_for_mount.clone(),
            blink_handle_for_mount.clone(),
            is_blinking_signal.clone(),
            is_blink_visible_signal.clone(),
            is_paused.clone(),
            reset_time.clone(),
        );
    });

    // --- Cleanup timer ---
    // Use on_cleanup to ensure timers are canceled on unmount to prevent memory leaks
    let countdown_timer_handle_on_cleanup = countdown_timer_handle.clone();
    let blink_timer_handle_on_cleanup = blink_timer_handle.clone();
    on_cleanup(move || {
        if let Some(handle) = countdown_timer_handle_on_cleanup.take() {
            drop(handle); // Cancel main countdown Interval
        }
        if let Some(handle) = blink_timer_handle_on_cleanup.borrow_mut().take() {
            drop(handle); // Cancel blink Interval
        }
    });

    // Create key event handler
    let key_handler = create_key_handler(
        input_mode.clone(),
        input_value.clone(),
        remaining_time.clone(),
    );

    view! {
        div(
            class="timer-container",
            tabindex="0",
            on:keydown=key_handler,
        ) {
            (if input_mode.get() {
                create_timer_input_view(
                    input_value.clone(),
                    input_error.clone(),
                    input_mode.clone()
                )
            } else {
                create_timer_display_view(
                    formatted_time.clone(),
                    is_blinking_signal.clone(),
                    is_blink_visible_signal.clone(),
                    is_paused.clone(),
                    timer_provider.clone(),
                    countdown_timer_handle.clone(),
                    blink_timer_handle.clone(),
                    remaining_time.clone(),
                    is_blinking_signal.clone(),
                    is_blink_visible_signal.clone(),
                    is_paused.clone(),
                    reset_time.clone()
                )
            })
        }
    }
}
