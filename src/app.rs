use crate::config::INITIAL_SECONDS;
use crate::event_logic::handle_click;
use crate::style_utils::compute_timer_style;
use crate::time_format::format_time;
use crate::timer_provider::{GlooTimerProvider, TimerHandle, TimerProvider};
use crate::timer_service::start_countdown_timer;
use std::cell::RefCell;
use std::rc::Rc;
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

#[component]
pub fn App() -> View {
    // instantiate timer provider as Rc<dyn TimerProvider> for sharing across closures
    let timer_provider: Rc<dyn TimerProvider> = Rc::new(GlooTimerProvider);
    // Clone provider for mount event to avoid moving original
    let timer_provider_for_mount = timer_provider.clone();
    // --- Countdown timer state ---
    // Remaining time signal (seconds)
    let remaining_time = create_signal(INITIAL_SECONDS);

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

    // --- Setup timer logic ---
    // Use on_mount to start the timer when the component mounts
    // Variable clones for the mount event
    let countdown_timer_handle_on_mount = countdown_timer_handle.clone();
    let remaining_time_on_mount = remaining_time.clone();
    let blink_timer_handle_on_mount = blink_timer_handle.clone();
    let is_blinking_signal_on_mount = is_blinking_signal.clone();
    let is_blink_visible_signal_on_mount = is_blink_visible_signal.clone();
    on_mount(move || {
        // start countdown using shared timer provider clone by value
        start_countdown_timer(
            timer_provider_for_mount.clone(),
            &countdown_timer_handle_on_mount,
            &remaining_time_on_mount,
            &blink_timer_handle_on_mount,
            &is_blinking_signal_on_mount,
            &is_blink_visible_signal_on_mount,
        );
        // Listen for 'p' key to toggle pause/resume
        let paused_signal_on_keydown = is_paused.clone();
        let provider_on_keydown = timer_provider_for_mount.clone();
        let countdown_handle_on_keydown = countdown_timer_handle_on_mount.clone();
        let blink_handle_on_keydown = blink_timer_handle_on_mount.clone();
        let remaining_time_on_keydown = remaining_time_on_mount.clone();
        let blinking_signal_on_keydown = is_blinking_signal_on_mount.clone();
        let visible_signal_on_keydown = is_blink_visible_signal_on_mount.clone();
        let key_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if event.key() == "p" {
                if paused_signal_on_keydown.get() {
                    // resume
                    paused_signal_on_keydown.set(false);
                    if !blinking_signal_on_keydown.get() {
                        start_countdown_timer(
                            provider_on_keydown.clone(),
                            &countdown_handle_on_keydown,
                            &remaining_time_on_keydown,
                            &blink_handle_on_keydown,
                            &blinking_signal_on_keydown,
                            &visible_signal_on_keydown,
                        );
                    }
                } else {
                    // pause
                    paused_signal_on_keydown.set(true);
                    if let Some(mut h) = countdown_handle_on_keydown.borrow_mut().take() {
                        h.cancel();
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keydown", key_closure.as_ref().unchecked_ref())
            .unwrap();
        key_closure.forget();
    });

    // --- Cleanup timer ---
    // Use on_cleanup to ensure timers are canceled on unmount to prevent memory leaks
    // Variable clones for the cleanup event
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

    // --- UI view ---
    // Clones for UI interaction in view
    let ui_timer = countdown_timer_handle.clone();
    let ui_time = remaining_time.clone();
    let ui_blink_active = is_blinking_signal.clone();
    let ui_blink_visible = is_blink_visible_signal.clone();
    let ui_blink_timer = blink_timer_handle.clone();
    let ui_paused = is_paused.clone();
    // Clone provider for click handler
    let timer_provider_click = timer_provider.clone();

    view! {
        div(class="timer-container") {
            p(
                class="timer-display",
                style=move || compute_timer_style(
                    ui_blink_active.get(),
                    ui_blink_visible.get(),
                    ui_paused.get(),
                ),
                on:click=move |_| {
                    // Handle click
                    let result = handle_click(
                        ui_time.get(),
                        ui_blink_active.get(),
                    );
                    // Apply click result
                    ui_time.set(result.reset_remaining);
                    if result.should_cancel_blink {
                        if let Some(mut handle) = ui_blink_timer.borrow_mut().take() {
                            handle.cancel();
                        }
                    }
                    ui_blink_active.set(result.is_blinking);
                    ui_blink_visible.set(result.is_visible);
                    // Restart countdown
                    start_countdown_timer(
                        timer_provider_click.clone(),
                        &ui_timer,
                        &ui_time,
                        &ui_blink_timer,
                        &ui_blink_active,
                        &ui_blink_visible,
                    );
                }
            ) {
                (formatted_time)
            }
        }
    }
}
