use crate::event_ui::{handle_click_and_reset, handle_timer_input};
use crate::style_utils::compute_timer_style;
use crate::timer_provider::{TimerHandle, TimerProvider};
use std::cell::RefCell;
use std::rc::Rc;
use sycamore::prelude::*;
use web_sys::MouseEvent;

/// Helper function to create timer input view
pub fn create_timer_input_view(
    input_value: Signal<String>,
    input_error: Signal<Option<String>>,
    input_mode: Signal<bool>,
) -> View {
    view! {
        div(class="input-container") {
            input(
                bind:value=input_value,
                id="timer-input",
                class="timer-input",
                // Add input event handler for automatic colon insertion
                on:input={
                    let input_value_clone = input_value.clone();
                    move |ev| handle_timer_input(ev, input_value_clone.clone())
                },
                // Prevent right-click context menu in input mode
                on:contextmenu=|ev: MouseEvent| ev.prevent_default(),
                on:blur={
                    let _input_mode_clone = input_mode.clone();
                    move |_| {
                        // Optionally exit input mode when focus is lost
                        // Uncomment if you want this behavior
                        // _input_mode_clone.set(false);
                    }
                }
            )
            (input_error.with_untracked(|err_opt| {
                if let Some(msg) = err_opt {
                    let error_msg = msg.clone();
                    view! { p(class="error-message") { (error_msg) } }
                } else {
                    view! {}
                }
            }))
        }
    }
}

/// Helper function to create timer display view
pub fn create_timer_display_view(
    formatted_time: ReadSignal<String>,
    ui_blink_active: Signal<bool>,
    ui_blink_visible: Signal<bool>,
    ui_paused: Signal<bool>,
    provider: Rc<dyn TimerProvider>,
    countdown_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    blink_handle: Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    remaining_time: Signal<u32>,
    blinking_signal: Signal<bool>,
    visible_signal: Signal<bool>,
    paused_signal: Signal<bool>,
    reset_time: Signal<Option<u32>>,
) -> View {
    view! {
        p(
            class="timer-display",
            style=move || compute_timer_style(
                ui_blink_active.get(),
                ui_blink_visible.get(),
                ui_paused.get(),
            ),
            on:click=move |_| {
                if !paused_signal.get() {
                    handle_click_and_reset(
                        provider.clone(),
                        countdown_handle.clone(),
                        blink_handle.clone(),
                        remaining_time.clone(),
                        blinking_signal.clone(),
                        visible_signal.clone(),
                        reset_time.clone(),
                    );
                }
            }
        )
        {
            (formatted_time)
        }
    }
}
