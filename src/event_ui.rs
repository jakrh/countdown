use crate::event_logic::handle_click;
use crate::timer_provider::{TimerHandle, TimerProvider};
use crate::timer_service::start_countdown_timer;
use std::{cell::RefCell, rc::Rc};
use sycamore::prelude::Signal;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

/// Handle click event and reset the timer
pub fn handle_click_and_reset(
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
