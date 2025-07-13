use crate::config::{BLINK_INTERVAL_MS, COUNTDOWN_INTERVAL_MS};
use crate::timer_logic::{toggle_blink, update_countdown};
use crate::timer_provider::{TimerHandle, TimerProvider};
use std::cell::RefCell;
use std::rc::Rc;
use sycamore::prelude::Signal;

/// Trigger blink timer if not already blinking, using provided TimerProvider
pub fn trigger_blink_timer(
    provider: Rc<dyn TimerProvider>,
    blink_timer_handle: &Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    is_blinking_signal: &Signal<bool>,
    is_blink_visible_signal: &Signal<bool>,
) {
    if !is_blinking_signal.get() {
        is_blinking_signal.set(true);
        is_blink_visible_signal.set(true);
        let blink_vis = is_blink_visible_signal.clone();
        let provider_clone = provider.clone();
        let handle = provider.set_interval(
            BLINK_INTERVAL_MS,
            Box::new(move || {
                // schedule blink toggle
                let vis = blink_vis.get();
                blink_vis.set(toggle_blink(vis));
                // ensure blinking continues with provider if needed
                let _ = provider_clone.clone();
            }),
        );
        *blink_timer_handle.borrow_mut() = Some(handle);
    }
}

/// Start or restart the main countdown timer using provided TimerProvider
pub fn start_countdown_timer(
    provider: Rc<dyn TimerProvider>,
    countdown_timer_handle: &Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    remaining_signal: &Signal<i32>,
    blink_timer_handle: &Rc<RefCell<Option<Box<dyn TimerHandle>>>>,
    is_blinking_signal: &Signal<bool>,
    is_blink_visible_signal: &Signal<bool>,
) {
    // Cancel existing timer if present
    if let Some(mut handle) = countdown_timer_handle.borrow_mut().take() {
        handle.cancel();
    }
    let timer_handle_clone = countdown_timer_handle.clone();
    let time_signal_clone = remaining_signal.clone();
    let blink_handle_clone = blink_timer_handle.clone();
    let blinking_clone = is_blinking_signal.clone();
    let visible_clone = is_blink_visible_signal.clone();

    let provider_clone = provider.clone();
    let handle = provider.set_interval(
        COUNTDOWN_INTERVAL_MS,
        Box::new(move || {
            let current = time_signal_clone.get();
            let result = update_countdown(current);
            time_signal_clone.set(result.remaining);

            if result.should_stop {
                // Stop the countdown timer when limit (e.g. -59:59) is reached
                if let Some(mut h) = timer_handle_clone.borrow_mut().take() {
                    h.cancel();
                }
                return;
            }

            if result.should_blink && !blinking_clone.get() {
                // start blink timer via provider
                trigger_blink_timer(
                    provider_clone.clone(),
                    &blink_handle_clone,
                    &blinking_clone,
                    &visible_clone,
                );
            }
        }),
    );

    *countdown_timer_handle.borrow_mut() = Some(handle);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BLINK_INTERVAL_MS, COUNTDOWN_INTERVAL_MS};
    use crate::timer_provider::tests::FakeProvider;
    use std::cell::RefCell;
    use std::rc::Rc;
    use sycamore::prelude::create_signal;
    use sycamore::reactive::create_root;

    #[test]
    fn test_start_countdown_no_blink() {
        let _ = create_root(|| {
            let fake = FakeProvider::new();
            let calls = fake.calls.clone();
            let provider: Rc<dyn TimerProvider> = Rc::new(fake);
            let countdown_handle = Rc::new(RefCell::new(None));
            let blink_handle = Rc::new(RefCell::new(None));
            let remaining = create_signal(5);
            let blinking = create_signal(false);
            let visible = create_signal(true);

            start_countdown_timer(
                provider.clone(),
                &countdown_handle,
                &remaining,
                &blink_handle,
                &blinking,
                &visible,
            );

            // FakeProvider triggers callback immediately: remaining should decrement
            assert_eq!(remaining.get(), 4);
            // Only countdown interval scheduled
            assert_eq!(&*calls.borrow(), &[COUNTDOWN_INTERVAL_MS]);
            // No blink timer scheduled
            assert!(blink_handle.borrow().is_none());
        });
    }

    #[test]
    fn test_start_countdown_with_blink() {
        let _ = create_root(|| {
            let fake = FakeProvider::new();
            let calls = fake.calls.clone();
            let provider: Rc<dyn TimerProvider> = Rc::new(fake);
            let countdown_handle = Rc::new(RefCell::new(None));
            let blink_handle = Rc::new(RefCell::new(None));
            let remaining = create_signal(0);
            let blinking = create_signal(false);
            let visible = create_signal(true);

            start_countdown_timer(
                provider.clone(),
                &countdown_handle,
                &remaining,
                &blink_handle,
                &blinking,
                &visible,
            );

            // FakeProvider triggers countdown then blink callbacks
            assert_eq!(remaining.get(), -1);
            assert!(blinking.get());
            // two intervals scheduled: countdown and blink
            assert_eq!(
                &*calls.borrow(),
                &[COUNTDOWN_INTERVAL_MS, BLINK_INTERVAL_MS]
            );
            // Blink handle is Some
            assert!(blink_handle.borrow().is_some());
        });
    }

    #[test]
    fn test_trigger_blink_timer_only_once() {
        let _ = create_root(|| {
            let fake = FakeProvider::new();
            let calls = fake.calls.clone();
            let provider: Rc<dyn TimerProvider> = Rc::new(fake);
            let blink_handle = Rc::new(RefCell::new(None));
            let blinking = create_signal(false);
            let visible = create_signal(false);

            // First call schedules blink
            trigger_blink_timer(provider.clone(), &blink_handle, &blinking, &visible);
            // blinking has started; fake provider callback toggled visibility off immediately
            assert!(blinking.get());
            assert!(!visible.get());
            assert_eq!(&*calls.borrow(), &[BLINK_INTERVAL_MS]);
            assert!(blink_handle.borrow().is_some());

            // Second call does nothing: no new schedule and handle remains Some
            trigger_blink_timer(provider.clone(), &blink_handle, &blinking, &visible);
            assert_eq!(&*calls.borrow(), &[BLINK_INTERVAL_MS]);
            assert!(blink_handle.borrow().is_some());
        });
    }
}
