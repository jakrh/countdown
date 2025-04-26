use gloo_timers::callback::Interval;

/// Cancel handle returned by TimerProvider
pub trait TimerHandle {
    fn cancel(&mut self);
}

/// Abstraction over timer creation (interval scheduling)
pub trait TimerProvider {
    fn set_interval(&self, ms: u32, callback: Box<dyn FnMut()>) -> Box<dyn TimerHandle>;
}

/// Real provider using gloo_timers
pub struct GlooTimerProvider;

impl TimerProvider for GlooTimerProvider {
    fn set_interval(&self, ms: u32, mut callback: Box<dyn FnMut()>) -> Box<dyn TimerHandle> {
        let interval = Interval::new(ms, move || {
            // invoke the callback
            (callback)();
        });
        Box::new(GlooTimerHandle {
            interval: Some(interval),
        })
    }
}

/// Internal handle for Gloo-based timers
struct GlooTimerHandle {
    interval: Option<Interval>,
}

impl TimerHandle for GlooTimerHandle {
    fn cancel(&mut self) {
        // Dropping the interval stops it
        self.interval.take();
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Fake handle to track cancellation
    struct FakeHandle {
        pub cancelled: bool,
    }

    impl TimerHandle for FakeHandle {
        fn cancel(&mut self) {
            self.cancelled = true;
        }
    }

    /// Fake provider that records calls and invokes callback immediately
    pub struct FakeProvider {
        pub calls: Rc<RefCell<Vec<u32>>>,
    }

    impl FakeProvider {
        pub fn new() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl TimerProvider for FakeProvider {
        fn set_interval(&self, ms: u32, mut callback: Box<dyn FnMut()>) -> Box<dyn TimerHandle> {
            self.calls.borrow_mut().push(ms);
            // simulate immediate trigger
            (callback)();
            Box::new(FakeHandle { cancelled: false })
        }
    }

    #[test]
    fn test_fake_provider() {
        let provider = FakeProvider::new();
        let calls_ref = provider.calls.clone();
        // use `Box<dyn FnMut()>` to match signature, with `move` so closure is 'static
        let handle = provider.set_interval(
            123,
            Box::new(move || {
                calls_ref.borrow_mut().push(999);
            }),
        );
        assert_eq!(&*provider.calls.borrow(), &[123, 999]);
        // test cancellation
        let mut h = handle;
        h.cancel();
        // no panic means cancel worked
    }
}
