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

// Helper to handle click event and reset the timer
fn handle_click_and_reset(
    provider: Rc<dyn TimerProvider>,
    countdown_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    blink_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    remaining_time: Signal<u32>,
    blinking_signal: Signal<bool>,
    visible_signal: Signal<bool>,
) {
    let result = handle_click(remaining_time.get(), blinking_signal.get());
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

// Extracted function to setup pause/resume key handler
fn setup_pause_resume_listener(
    timer_provider: Rc<dyn TimerProvider>,
    paused_signal: Signal<bool>,
    countdown_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    blink_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    remaining_time: Signal<u32>,
    blinking_signal: Signal<bool>,
    visible_signal: Signal<bool>,
) {
    let window = web_sys::window().unwrap();
    // Clone inside closure
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
                // Resume countdown
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
                // Prevent pausing while blinking
                if blinking.get() {
                    return;
                }
                // Pause countdown
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
        // simplified pause/resume listener setup
        setup_pause_resume_listener(
            timer_provider_for_mount.clone(),
            is_paused.clone(),
            countdown_timer_handle_on_mount.clone(),
            blink_timer_handle_on_mount.clone(),
            remaining_time_on_mount.clone(),
            is_blinking_signal_on_mount.clone(),
            is_blink_visible_signal_on_mount.clone(),
        );
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

    // Named click handler using extracted helper
    let click_handler = {
        let provider = timer_provider_click.clone();
        let countdown = ui_timer.clone();
        let blink = ui_blink_timer.clone();
        let time = ui_time.clone();
        let blink_active = ui_blink_active.clone();
        let blink_visible = ui_blink_visible.clone();
        move |_| {
            handle_click_and_reset(
                provider.clone(),
                countdown.clone(),
                blink.clone(),
                time.clone(),
                blink_active.clone(),
                blink_visible.clone(),
            );
        }
    };

    view! {
        div(class="timer-container") {
            p(
                class="timer-display",
                style=move || compute_timer_style(
                    ui_blink_active.get(),
                    ui_blink_visible.get(),
                    ui_paused.get(),
                ),
                on:click=click_handler
            ) {
                (formatted_time)
            }
        }
    }
}
