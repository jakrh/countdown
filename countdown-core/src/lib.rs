//! Pure countdown logic: no clock, no DOM, no dependencies.
//!
//! Time is whole milliseconds (`i64`) from the shell's monotonic clock.
//! The shell owns every side effect and calls these functions with the
//! current time.

// Bodies are stubs until the implementation lands; tests are written
// against this API first.
#![allow(unused_variables)]

pub const DEFAULT_START_FROM_SECS: u32 = 1500;

/// Must divide 1000, so that every change of the displayed number
/// starts a visible half of the blink.
pub const OVERDUE_BLINK_PERIOD_MS: i64 = 1000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub phase: Phase,
    pub start_from_secs: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    Running { deadline: i64 },
    Paused { remaining_ms: i64 },
    Editing { remaining_ms: i64, draft: Draft },
}

/// `digits` is `[minute tens, minute ones, second tens, second ones]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub digits: [u8; 4],
    pub cursor: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Running,
    Paused,
    Overdue,
    Editing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Pause,
    Edit,
    Click,
    Digit(u8),
    Left,
    Right,
    Backspace,
    Confirm,
    Cancel,
    Blur,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    Normal,
    Frozen,
    Overdue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    SaveStartFrom(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub text: String,
    pub clickable: bool,
    pub kind: ViewKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewKind {
    Timer { look: Look, overdue_onset: Option<i64> },
    Editor { cursor: u8 },
}

pub fn init(stored: Option<&str>, now: i64) -> Model {
    todo!()
}

pub fn update(m: Model, input: Input, now: i64) -> (Model, Option<Effect>) {
    todo!()
}

pub fn view(m: &Model, now: i64) -> View {
    todo!()
}

pub fn next_wakeup(m: &Model, now: i64) -> Option<u32> {
    todo!()
}

pub fn mode(m: &Model, now: i64) -> Mode {
    todo!()
}
