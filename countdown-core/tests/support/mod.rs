//! Builders and readers that let each test read like the spec's
//! Given/When/Then. Each test binary uses a different subset.
#![allow(dead_code)]

use countdown_core::*;

/// An arbitrary start time, deliberately not a multiple of 1000 so that
/// code wrongly aligned to `now % 1000` fails.
pub const T0: i64 = 1_234_567;

/// A later time, also off a whole number of seconds from `T0`.
pub const T1: i64 = T0 + 7_321;

/// The start-from time of every built model. It differs from the default
/// so that a test can tell a value taken from the model from the default.
pub const TEST_START: u32 = 600;
const _: () = assert!(TEST_START != DEFAULT_START_FROM_SECS);

pub fn running(r: i64, t0: i64) -> Model {
    Model {
        phase: Phase::Running { deadline: t0 + r },
        start_from_secs: TEST_START,
    }
}

pub fn paused(r: i64) -> Model {
    Model {
        phase: Phase::Paused { remaining_ms: r },
        start_from_secs: TEST_START,
    }
}

pub fn editing(r: i64, text: &str, cursor: u8) -> Model {
    Model {
        phase: Phase::Editing {
            remaining_ms: r,
            draft: draft(text, cursor),
        },
        start_from_secs: TEST_START,
    }
}

/// `text` is `"MM:SS"`.
pub fn draft(text: &str, cursor: u8) -> Draft {
    let b = text.as_bytes();
    assert!(
        b.len() == 5 && b[2] == b':',
        "draft text must be MM:SS: {text}"
    );
    Draft {
        digits: [b[0] - b'0', b[1] - b'0', b[3] - b'0', b[4] - b'0'],
        cursor,
    }
}

/// Applies `steps` in order and returns the final model with every
/// effect produced along the way.
pub fn apply(m: Model, steps: &[(Input, i64)]) -> (Model, Vec<Effect>) {
    steps
        .iter()
        .fold((m, Vec::new()), |(m, mut effects), &(input, t)| {
            let (m, effect) = update(m, input, t);
            effects.extend(effect);
            (m, effects)
        })
}

pub fn text_at(m: &Model, t: i64) -> String {
    view(m, t).text
}

/// `None` for an editor view.
pub fn look_at(m: &Model, t: i64) -> Option<Look> {
    match view(m, t).kind {
        ViewKind::Timer { look, .. } => Some(look),
        ViewKind::Editor { .. } => None,
    }
}

/// `None` for a timer view.
pub fn cursor_at(m: &Model, t: i64) -> Option<u8> {
    match view(m, t).kind {
        ViewKind::Editor { cursor } => Some(cursor),
        ViewKind::Timer { .. } => None,
    }
}

pub fn onset_at(m: &Model, t: i64) -> Option<i64> {
    match view(m, t).kind {
        ViewKind::Timer { overdue_onset, .. } => overdue_onset,
        ViewKind::Editor { .. } => None,
    }
}

/// F-REM, read straight from the public fields.
pub fn remaining(m: &Model, t: i64) -> i64 {
    match m.phase {
        Phase::Running { deadline } => deadline - t,
        Phase::Paused { remaining_ms } | Phase::Editing { remaining_ms, .. } => remaining_ms,
    }
}

/// The seconds a timer text shows. Accepts only F-TEXT's forms:
/// `"MM:SS"`, or `"-MM:SS"` for a nonzero value.
pub fn parse_shown(text: &str) -> i64 {
    let (sign, body) = match text.strip_prefix('-') {
        Some(body) => (-1, body),
        None => (1, text),
    };
    let b = body.as_bytes();
    let well_formed =
        b.len() == 5 && b[2] == b':' && [b[0], b[1], b[3], b[4]].iter().all(u8::is_ascii_digit);
    assert!(well_formed, "not MM:SS: {text:?}");
    let digit = |i: usize| i64::from(b[i] - b'0');
    let (mm, ss) = (digit(0) * 10 + digit(1), digit(3) * 10 + digit(4));
    assert!(ss < 60, "seconds out of range: {text:?}");
    let value = mm * 60 + ss;
    assert!(sign == 1 || value != 0, "negative zero: {text:?}");
    sign * value
}

pub fn shown_at(m: &Model, t: i64) -> i64 {
    parse_shown(&text_at(m, t))
}

pub const ALL_INPUTS: [Input; 10] = [
    Input::Pause,
    Input::Edit,
    Input::Click,
    Input::Digit(1),
    Input::Left,
    Input::Right,
    Input::Backspace,
    Input::Confirm,
    Input::Cancel,
    Input::Blur,
];

/// True for the `—` (no-op) cells of the spec's transition table.
pub fn is_noop(mode: Mode, input: Input) -> bool {
    use Input::*;
    !matches!(
        (mode, input),
        (Mode::Running, Pause | Edit | Click)
            | (Mode::Paused, Pause)
            | (Mode::Overdue, Click)
            | (
                Mode::Editing,
                Digit(_) | Left | Right | Backspace | Confirm | Cancel | Blur
            )
    )
}
