//! The web-sys shell: it turns DOM events into core inputs, reads the
//! clock and storage, and draws the core's view. All behavior lives in
//! `countdown-core`; the shell never looks at the mode (SH-NO-MODE-LOGIC).

use std::cell::RefCell;
use std::rc::Rc;

use countdown_core::{
    Effect, Input, Look, Model, OVERDUE_BLINK_PERIOD_MS, View, ViewKind, init, next_wakeup, update,
    view,
};
use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{
    Document, Event, EventTarget, HtmlElement, KeyboardEvent, MouseEvent, Performance, Storage,
};

const CURSOR_BLINK_PERIOD_MS: u32 = 1000;
const COLOR_NORMAL: &str = "white";
const COLOR_FROZEN: &str = "darkturquoise";
const COLOR_OVERDUE: &str = "red";
const COLOR_CURSOR_TEXT: &str = "black";

/// P-KEY.
const STORAGE_KEY: &str = "countdown_start_from_seconds";

struct App {
    document: Document,
    time: HtmlElement,
    performance: Performance,
    storage: Option<Storage>,
    model: Model,
    /// The one pending wakeup. Dropping a `Timeout` cancels it, so
    /// replacing it is how a wakeup is rescheduled.
    wakeup: Option<Timeout>,
}

type Shared = Rc<RefCell<App>>;

fn main() {
    console_error_panic_hook::set_once();
    let window = web_sys::window().expect("runs in a browser window");
    let document = window.document().expect("the window has a document");
    let time = document
        .get_element_by_id("time")
        .expect("index.html has #time")
        .unchecked_into::<HtmlElement>();
    let performance = window.performance().expect("the window has performance");
    // P-LOAD: storage that cannot be read counts as a missing key.
    let storage = window.local_storage().ok().flatten();
    let stored = storage
        .as_ref()
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten());

    publish_tunables(&document);
    let now = clock(&performance);
    let app: Shared = Rc::new(RefCell::new(App {
        model: init(stored.as_deref(), now),
        document: document.clone(),
        time: time.clone(),
        performance,
        storage,
        wakeup: None,
    }));
    refresh(&app, now);

    let shared = app.clone();
    listen(&window, "keydown", move |e| {
        let e = e.unchecked_ref::<KeyboardEvent>();
        let modified = e.shift_key() || e.ctrl_key() || e.alt_key() || e.meta_key();
        if e.repeat() || modified {
            return;
        }
        if let Some(input) = key_input(&e.code()) {
            e.prevent_default();
            dispatch(&shared, input);
        }
    });
    let shared = app.clone();
    listen(&time, "click", move |e| {
        if e.unchecked_ref::<MouseEvent>().button() == 0 {
            dispatch(&shared, Input::Click);
        }
    });
    let shared = app.clone();
    listen(&window, "blur", move |_| dispatch(&shared, Input::Blur));
    let shared = app.clone();
    listen(&document, "visibilitychange", move |_| {
        if !shared.borrow().document.hidden() {
            let now = shared.borrow().now();
            refresh(&shared, now);
        }
    });
    listen(&document, "contextmenu", |e| e.prevent_default());
}

/// SH-KEYMAP.
fn key_input(code: &str) -> Option<Input> {
    Some(match code {
        "KeyP" => Input::Pause,
        "KeyF" => Input::Edit,
        "ArrowLeft" => Input::Left,
        "ArrowRight" => Input::Right,
        "Backspace" => Input::Backspace,
        "Enter" | "NumpadEnter" => Input::Confirm,
        "Escape" => Input::Cancel,
        _ => {
            let rest = code
                .strip_prefix("Digit")
                .or_else(|| code.strip_prefix("Numpad"))?;
            let &[b] = rest.as_bytes() else { return None };
            if !b.is_ascii_digit() {
                return None;
            }
            Input::Digit(b - b'0')
        }
    })
}

/// SH-CLOCK: whole milliseconds, so the core's arithmetic stays exact.
fn clock(performance: &Performance) -> i64 {
    performance.now().floor() as i64
}

/// SH-DISPATCH.
fn dispatch(app: &Shared, input: Input) {
    let now = app.borrow().now();
    let effect = {
        let mut a = app.borrow_mut();
        let (model, effect) = update(a.model.clone(), input, now);
        a.model = model;
        effect
    };
    if let Some(Effect::SaveStartFrom(n)) = effect {
        app.borrow().save(n);
    }
    refresh(app, now);
}

/// Renders and reschedules: the tail of SH-DISPATCH and SH-INIT, and all
/// of SH-WAKE and SH-VISIBLE.
fn refresh(app: &Shared, now: i64) {
    let mut a = app.borrow_mut();
    a.render(view(&a.model, now), now);
    a.wakeup = next_wakeup(&a.model, now).map(|ms| {
        let app = app.clone();
        Timeout::new(ms, move || {
            let now = app.borrow().now();
            refresh(&app, now);
        })
    });
}

impl App {
    fn now(&self) -> i64 {
        clock(&self.performance)
    }

    /// P-SAVE: a failed write is ignored.
    fn save(&self, secs: u32) {
        if let Some(storage) = &self.storage {
            let _ = storage.set_item(STORAGE_KEY, &secs.to_string());
        }
    }

    /// Rebuilds the time element's children on every render, which
    /// restarts their CSS animations (SH-OVERDUE-BLINK, SH-CURSOR-BLINK).
    fn render(&self, view: View, now: i64) {
        let classes = self.time.class_list();
        classes
            .toggle_with_force("clickable", view.clickable)
            .expect("a valid class name");
        classes
            .toggle_with_force("editor", matches!(view.kind, ViewKind::Editor { .. }))
            .expect("a valid class name");
        self.time.set_text_content(None);
        match view.kind {
            ViewKind::Timer {
                look,
                overdue_onset,
            } => {
                let class = match look {
                    Look::Normal => "normal",
                    Look::Frozen => "frozen",
                    Look::Overdue => "overdue",
                };
                let span = self.append_span(&view.text, class);
                if let Some(onset) = overdue_onset {
                    let elapsed = (now - onset).rem_euclid(OVERDUE_BLINK_PERIOD_MS);
                    set_style(&span, "animation-delay", &format!("-{elapsed}ms"));
                }
            }
            ViewKind::Editor { cursor } => {
                // Slots 0 and 1 precede the colon at index 2 of `MM:SS`.
                let cursor_index = usize::from(cursor + cursor / 2);
                for (i, ch) in view.text.chars().enumerate() {
                    let class = if i == cursor_index { "cursor" } else { "" };
                    self.append_span(ch.encode_utf8(&mut [0; 4]), class);
                }
            }
        }
    }

    fn append_span(&self, text: &str, class: &str) -> HtmlElement {
        let span = self
            .document
            .create_element("span")
            .expect("span is a valid tag")
            .unchecked_into::<HtmlElement>();
        span.set_class_name(class);
        span.set_text_content(Some(text));
        self.time.append_child(&span).expect("a span fits in a div");
        span
    }
}

/// Hands the tunables `styles.css` needs to it as custom properties, so
/// each value is written once, in Rust: the shell's here, and the core's
/// `OVERDUE_BLINK_PERIOD_MS`, which also sets the blink phase.
fn publish_tunables(document: &Document) {
    let root = document
        .document_element()
        .expect("the document has a root")
        .unchecked_into::<HtmlElement>();
    let properties = [
        ("--color-normal", COLOR_NORMAL.to_owned()),
        ("--color-frozen", COLOR_FROZEN.to_owned()),
        ("--color-overdue", COLOR_OVERDUE.to_owned()),
        ("--color-cursor-text", COLOR_CURSOR_TEXT.to_owned()),
        (
            "--overdue-blink-period",
            format!("{OVERDUE_BLINK_PERIOD_MS}ms"),
        ),
        (
            "--cursor-blink-period",
            format!("{CURSOR_BLINK_PERIOD_MS}ms"),
        ),
    ];
    for (name, value) in properties {
        set_style(&root, name, &value);
    }
}

fn set_style(element: &HtmlElement, property: &str, value: &str) {
    element
        .style()
        .set_property(property, value)
        .expect("a writable inline style");
}

/// Listeners live as long as the page, so their closures are leaked.
fn listen(target: &EventTarget, kind: &str, handler: impl FnMut(Event) + 'static) {
    let closure = Closure::<dyn FnMut(Event)>::new(handler);
    target
        .add_event_listener_with_callback(kind, closure.as_ref().unchecked_ref())
        .expect("a valid listener");
    closure.forget();
}
