//! Pure countdown logic: no clock, no DOM, no dependencies.
//!
//! Time is whole milliseconds (`i64`) from the shell's monotonic clock.
//! The shell owns every side effect and calls these functions with the
//! current time.

pub const DEFAULT_START_FROM_SECS: u32 = 1500;

/// Must divide 1000, so that every change of the displayed number
/// starts a visible half of the blink.
pub const OVERDUE_BLINK_PERIOD_MS: i64 = 1000;

const MS_PER_SEC: i64 = 1000;

/// The largest value `MM:SS` can show, in either direction.
const MAX_SECS: u32 = 59 * 60 + 59;

const LAST_SLOT: u8 = 3;

const _: () = assert!(MS_PER_SEC % OVERDUE_BLINK_PERIOD_MS == 0);
const _: () = assert!(DEFAULT_START_FROM_SECS <= MAX_SECS);

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
    Timer {
        look: Look,
        overdue_onset: Option<i64>,
    },
    Editor {
        cursor: u8,
    },
}

/// P-LOAD, then T-INIT.
pub fn init(stored: Option<&str>, now: i64) -> Model {
    let start_from_secs = stored
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|&n| n <= MAX_SECS)
        .unwrap_or(DEFAULT_START_FROM_SECS);
    Model {
        phase: running_for(start_from_secs, now),
        start_from_secs,
    }
}

/// The transition table of spec §4. Every combination it does not list
/// returns the model unchanged.
pub fn update(m: Model, input: Input, now: i64) -> (Model, Option<Effect>) {
    use Input::*;
    let md = mode(&m, now);
    let r = remaining(&m.phase, now);
    let phase = match (input, &m.phase) {
        (Pause, Phase::Running { .. }) if md == Mode::Running => Phase::Paused { remaining_ms: r },
        (Edit, Phase::Running { .. }) if md == Mode::Running => {
            let secs = u32::try_from(displayed_secs(r))
                .expect("Mode::Running shows a non-negative number");
            Phase::Editing {
                remaining_ms: r,
                draft: Draft::showing(secs),
            }
        }
        (Click, Phase::Running { .. }) => running_for(m.start_from_secs, now),
        (Digit(_) | Left | Right | Backspace, Phase::Editing { draft, .. }) => Phase::Editing {
            remaining_ms: r,
            draft: draft.edited(input),
        },
        (Confirm, Phase::Editing { draft, .. }) => {
            let n = draft.secs();
            return (
                Model {
                    phase: running_for(n, now),
                    start_from_secs: n,
                },
                Some(Effect::SaveStartFrom(n)),
            );
        }
        // T-PAU-PAUSE and T-EDT-CANCEL: time resumes from where it stopped.
        (Pause, Phase::Paused { .. }) | (Cancel | Blur, Phase::Editing { .. }) => {
            Phase::Running { deadline: now + r }
        }
        _ => return (m, None),
    };
    (Model { phase, ..m }, None)
}

/// F-TEXT, F-LOOK, F-CLICKABLE and F-ONSET.
pub fn view(m: &Model, now: i64) -> View {
    let md = mode(m, now);
    let clickable = matches!(md, Mode::Running | Mode::Overdue);
    let text = match &m.phase {
        Phase::Editing { draft, .. } => {
            let [d0, d1, d2, d3] = draft.digits;
            format!("{d0}{d1}:{d2}{d3}")
        }
        phase => timer_text(displayed_secs(remaining(phase, now))),
    };
    let timer = |look, overdue_onset| ViewKind::Timer {
        look,
        overdue_onset,
    };
    let kind = match &m.phase {
        Phase::Paused { .. } => timer(Look::Frozen, None),
        &Phase::Running { deadline } if md == Mode::Overdue => {
            timer(Look::Overdue, Some(deadline + MS_PER_SEC))
        }
        Phase::Running { .. } => timer(Look::Normal, None),
        Phase::Editing { draft, .. } => ViewKind::Editor {
            cursor: draft.cursor,
        },
    };
    View {
        text,
        clickable,
        kind,
    }
}

/// F-WAKE.
pub fn next_wakeup(m: &Model, now: i64) -> Option<u32> {
    if !matches!(mode(m, now), Mode::Running | Mode::Overdue) {
        return None;
    }
    let r = remaining(&m.phase, now);
    let s = displayed_secs(r);
    if s == -i64::from(MAX_SECS) {
        return None;
    }
    let delay = r - (s - 1) * MS_PER_SEC;
    Some(u32::try_from(delay).expect("INV-R-BOUND keeps the delay in 1..=1000"))
}

/// F-MODE.
pub fn mode(m: &Model, now: i64) -> Mode {
    match m.phase {
        Phase::Running { .. } if displayed_secs(remaining(&m.phase, now)) < 0 => Mode::Overdue,
        Phase::Running { .. } => Mode::Running,
        Phase::Paused { .. } => Mode::Paused,
        Phase::Editing { .. } => Mode::Editing,
    }
}

fn running_for(secs: u32, now: i64) -> Phase {
    Phase::Running {
        deadline: now + i64::from(secs) * MS_PER_SEC,
    }
}

/// F-REM.
fn remaining(phase: &Phase, now: i64) -> i64 {
    match *phase {
        Phase::Running { deadline } => deadline - now,
        Phase::Paused { remaining_ms } | Phase::Editing { remaining_ms, .. } => remaining_ms,
    }
}

/// F-SEC.
fn displayed_secs(r: i64) -> i64 {
    let ceil = r.div_euclid(MS_PER_SEC) + i64::from(r.rem_euclid(MS_PER_SEC) != 0);
    let max = i64::from(MAX_SECS);
    ceil.clamp(-max, max)
}

/// F-TEXT for a timer.
fn timer_text(s: i64) -> String {
    let sign = if s < 0 { "-" } else { "" };
    let abs = s.unsigned_abs();
    format!("{sign}{:02}:{:02}", abs / 60, abs % 60)
}

impl Draft {
    /// F-DRAFT-FROM.
    fn showing(secs: u32) -> Draft {
        let (m, s) = (secs / 60, secs % 60);
        let digit = |n: u32| u8::try_from(n).expect("a single decimal digit");
        Draft {
            digits: [digit(m / 10), digit(m % 10), digit(s / 10), digit(s % 10)],
            cursor: 0,
        }
    }

    /// F-DRAFT-SECS.
    fn secs(&self) -> u32 {
        let [d0, d1, d2, d3] = self.digits.map(u32::from);
        (d0 * 10 + d1) * 60 + d2 * 10 + d3
    }

    /// T-EDT-DIGIT, T-EDT-LEFT, T-EDT-RIGHT and T-EDT-BACKSPACE. Other
    /// inputs leave the draft unchanged.
    fn edited(&self, input: Input) -> Draft {
        let mut next = self.clone();
        let slot = usize::from(self.cursor);
        match input {
            Input::Digit(d) if accepts(slot, d) => {
                next.digits[slot] = d;
                next.cursor = (self.cursor + 1).min(LAST_SLOT);
            }
            Input::Left => next.cursor = self.cursor.saturating_sub(1),
            Input::Right => next.cursor = (self.cursor + 1).min(LAST_SLOT),
            Input::Backspace => {
                next.digits[slot] = 0;
                next.cursor = self.cursor.saturating_sub(1);
            }
            _ => {}
        }
        next
    }
}

/// F-ACCEPTS: the tens slots (0 and 2) take at most 5, so a draft is
/// always a valid `MM:SS`.
fn accepts(slot: usize, d: u8) -> bool {
    d <= 9 && (slot % 2 == 1 || d <= 5)
}
