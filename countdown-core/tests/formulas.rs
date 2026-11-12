//! F-BLINK has no core output beyond `overdue_onset`; it is covered by
//! INV-BLINK-ALIGNED. F-REM for `Editing` is only observable on leaving
//! the editor; T-EDT-CANCEL covers it. F-WAKE's range over reachable
//! states is checked in `invariants.rs`.

mod support;

use countdown_core::*;
use support::*;

use Input::*;

#[test]
fn f_rem_running_counts_down_from_deadline() {
    let m = running(10_000, T0);
    assert_eq!(text_at(&m, T0 + 4000), "00:06");
}

#[test]
fn f_rem_paused_ignores_now() {
    let m = paused(1_452_700);
    assert_eq!(text_at(&m, T0), "24:13");
    assert_eq!(text_at(&m, T0 + 50_000), "24:13");
}

#[test]
fn f_sec() {
    let cases = [
        (3_599_000, "59:59"),
        (1001, "00:02"),
        (1000, "00:01"),
        (1, "00:01"),
        (0, "00:00"),
        (-999, "00:00"),
        (-1000, "-00:01"),
        (-1001, "-00:01"),
        (-3_598_001, "-59:58"),
        (-3_599_000, "-59:59"),
        (-3_599_999, "-59:59"),
    ];
    for (r, text) in cases {
        assert_eq!(text_at(&running(r, T0), T0), text, "r = {r}");
    }
}

#[test]
fn f_sec_clamps_at_lower_bound() {
    // -10^12 overflows `i32`: the overdue blink runs for days on end.
    for r in [-3_600_000, -3_600_001, -1_000_000_000_000] {
        assert_eq!(text_at(&running(r, T0), T0), "-59:59", "r = {r}");
    }
}

/// Unreachable per INV-R-BOUND, but F-SEC defines it.
#[test]
fn f_sec_clamps_at_upper_bound() {
    for r in [3_599_001, 3_600_000, 1_000_000_000_000] {
        assert_eq!(text_at(&running(r, T0), T0), "59:59", "r = {r}");
    }
}

#[test]
fn f_mode() {
    let cases = [
        (running(3_599_000, T0), Mode::Running),
        (running(0, T0), Mode::Running),
        (running(-999, T0), Mode::Running),
        (running(-1000, T0), Mode::Overdue),
        (running(-3_600_000, T0), Mode::Overdue),
        (paused(-300), Mode::Paused),
        (paused(60_000), Mode::Paused),
        (editing(-300, "00:00", 0), Mode::Editing),
    ];
    for (m, expected) in cases {
        assert_eq!(mode(&m, T0), expected, "{m:?}");
    }
}

#[test]
fn f_text() {
    let cases = [
        (3_599_000, "59:59"),
        (61_000, "01:01"),
        (9_000, "00:09"),
        (0, "00:00"),
        (-61_000, "-01:01"),
    ];
    for (r, text) in cases {
        assert_eq!(text_at(&running(r, T0), T0), text, "r = {r}");
    }
}

#[test]
fn f_text_editing_shows_draft() {
    assert_eq!(text_at(&editing(1_452_700, "09:05", 2), T0), "09:05");
}

#[test]
fn f_draft_secs() {
    let cases = [("00:00", 0), ("10:01", 601), ("59:59", 3599)];
    for (text, secs) in cases {
        let (_, effect) = update(editing(60_000, text, 0), Confirm, T0);
        assert_eq!(effect, Some(Effect::SaveStartFrom(secs)), "{text}");
    }
}

#[test]
fn f_draft_from() {
    let cases = [
        (3_599_000, "59:59"),
        (61_000, "01:01"),
        (600_500, "10:01"),
        (1, "00:01"),
        (-500, "00:00"),
    ];
    for (r, text) in cases {
        let (m, _) = update(running(r, T0), Edit, T0);
        assert_eq!(text_at(&m, T0), text, "r = {r}");
        assert_eq!(cursor_at(&m, T0), Some(0), "r = {r}");
    }
}

#[test]
fn f_accepts() {
    // Every rejected digit is at least 6, so starting from zeros, the
    // slot holds `d` afterwards exactly when `d` was accepted.
    for slot in 0..4u8 {
        for d in 0..=12u8 {
            let accepts = d <= 9 && (slot % 2 == 1 || d <= 5);
            let (after, _) = update(editing(60_000, "00:00", slot), Digit(d), T0);
            let Phase::Editing { draft, .. } = &after.phase else {
                panic!("left Editing: {after:?}");
            };
            assert_eq!(
                draft.digits[usize::from(slot)] == d,
                accepts,
                "slot {slot}, d {d}"
            );
        }
    }
}

#[test]
fn f_look() {
    let cases = [
        (running(60_000, T0), Some(Look::Normal)),
        (running(-999, T0), Some(Look::Normal)),
        (running(-1000, T0), Some(Look::Overdue)),
        (paused(60_000), Some(Look::Frozen)),
        (paused(-300), Some(Look::Frozen)),
        (editing(60_000, "01:00", 0), None),
    ];
    for (m, expected) in cases {
        assert_eq!(look_at(&m, T0), expected, "{m:?}");
    }
}

#[test]
fn f_clickable() {
    let cases = [
        (running(60_000, T0), true),
        (running(-5000, T0), true),
        (running(-3_600_000, T0), true),
        (paused(60_000), false),
        (editing(60_000, "01:00", 0), false),
    ];
    for (m, expected) in cases {
        assert_eq!(view(&m, T0).clickable, expected, "{m:?}");
    }
}

#[test]
fn f_onset() {
    // deadline = T0 - 1000, so the onset is T0.
    assert_eq!(onset_at(&running(-1000, T0), T0), Some(T0));
    // deadline = T0 - 5500, so the onset is T0 - 4500.
    assert_eq!(onset_at(&running(-5500, T0), T0), Some(T0 - 4500));
}

#[test]
fn f_onset_none_outside_overdue() {
    for m in [running(-999, T0), running(60_000, T0), paused(-300)] {
        assert_eq!(onset_at(&m, T0), None, "{m:?}");
    }
}

#[test]
fn f_wake() {
    let cases = [
        (3_599_000, 1000),
        (1001, 1),
        (1000, 1000),
        (1, 1),
        (0, 1000),
        (-1, 999),
        (-999, 1),
        (-1000, 1000),
        (-3_598_000, 1000),
        (-3_598_001, 999),
        (-3_598_999, 1),
    ];
    for (r, wake) in cases {
        assert_eq!(next_wakeup(&running(r, T0), T0), Some(wake), "r = {r}");
    }
}

#[test]
fn f_wake_none() {
    let cases = [
        running(-3_599_000, T0),
        running(-3_600_000, T0),
        paused(60_000),
        editing(60_000, "01:00", 0),
    ];
    for m in cases {
        assert_eq!(next_wakeup(&m, T0), None, "{m:?}");
    }
}
