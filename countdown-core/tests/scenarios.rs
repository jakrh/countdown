mod support;

use countdown_core::*;
use support::*;

use Input::*;

#[test]
fn sc_launch_default() {
    let m = init(None, T0);
    assert_eq!(m.start_from_secs, DEFAULT_START_FROM_SECS);
    assert_eq!(mode(&m, T0), Mode::Running);
    assert_eq!(remaining(&m, T0), i64::from(DEFAULT_START_FROM_SECS) * 1000);
}

#[test]
fn sc_first_second() {
    let m = init(Some("600"), T0);
    assert_eq!(text_at(&m, T0 + 999), "10:00");
    assert_eq!(look_at(&m, T0 + 999), Some(Look::Normal));
    assert_eq!(text_at(&m, T0 + 1000), "09:59");
}

#[test]
fn sc_zero_then_overdue() {
    let m = running(1000, T0);
    assert_eq!(text_at(&m, T0), "00:01");
    assert_eq!(text_at(&m, T0 + 999), "00:01");
    assert_eq!(text_at(&m, T0 + 1000), "00:00");
    assert_eq!(look_at(&m, T0 + 1000), Some(Look::Normal));
    assert_eq!(text_at(&m, T0 + 1999), "00:00");
    assert_eq!(look_at(&m, T0 + 1999), Some(Look::Normal));
    assert_eq!(text_at(&m, T0 + 2000), "-00:01");
    assert_eq!(look_at(&m, T0 + 2000), Some(Look::Overdue));
}

#[test]
fn sc_clamp() {
    let m = running(-3_599_500, T0);
    assert_eq!(text_at(&m, T0), "-59:59");
    assert_eq!(next_wakeup(&m, T0), None);

    let later = T0 + 10_000_000;
    assert_eq!(text_at(&m, later), "-59:59");
    assert_eq!(look_at(&m, later), Some(Look::Overdue));

    let (m, _) = update(m, Click, later);
    assert_eq!(mode(&m, later), Mode::Running);
    assert_eq!(remaining(&m, later), i64::from(m.start_from_secs) * 1000);
}

#[test]
fn sc_pause_keeps_subsecond() {
    let m = running(1_452_700, T0);
    assert_eq!(text_at(&m, T0), "24:13");

    let (m, _) = apply(m, &[(Pause, T0), (Pause, T0 + 5000)]);
    assert_eq!(text_at(&m, T0 + 5699), "24:13");
    assert_eq!(look_at(&m, T0 + 5699), Some(Look::Normal));
    assert_eq!(text_at(&m, T0 + 5700), "24:12");
}

#[test]
fn sc_pause_during_zero() {
    let m = running(-300, T0);
    assert_eq!(text_at(&m, T0), "00:00");
    assert_eq!(mode(&m, T0), Mode::Running);

    let (m, _) = update(m, Pause, T0);
    assert_eq!(text_at(&m, T0), "00:00");
    assert_eq!(look_at(&m, T0), Some(Look::Frozen));

    let (m, _) = update(m, Pause, T1);
    assert_eq!(text_at(&m, T1 + 699), "00:00");
    assert_eq!(look_at(&m, T1 + 699), Some(Look::Normal));
    assert_eq!(text_at(&m, T1 + 700), "-00:01");
    assert_eq!(look_at(&m, T1 + 700), Some(Look::Overdue));
}

#[test]
fn sc_no_pause_when_overdue() {
    let m = running(-5000, T0);
    assert_eq!(text_at(&m, T0), "-00:05");
    assert_eq!(update(m.clone(), Pause, T0), (m, None));
}

#[test]
fn sc_edit_overwrite_all() {
    let m = running(1_452_700, T0);
    assert_eq!(text_at(&m, T0), "24:13");

    let (m, _) = apply(
        m,
        &[
            (Edit, T0),
            (Digit(0), T0),
            (Digit(5), T0),
            (Digit(3), T0),
            (Digit(0), T0),
        ],
    );
    assert_eq!(text_at(&m, T0), "05:30");
    assert_eq!(cursor_at(&m, T0), Some(3));
}

#[test]
fn sc_edit_one_slot() {
    let m = editing(60_000, "24:13", 0);
    let (m, _) = apply(m, &[(Right, T0), (Right, T0), (Digit(4), T0)]);
    assert_eq!(text_at(&m, T0), "24:43");
    assert_eq!(cursor_at(&m, T0), Some(3));
}

#[test]
fn sc_edit_reject_tens() {
    let m = editing(60_000, "24:13", 2);
    assert_eq!(update(m.clone(), Digit(6), T0), (m, None));
}

#[test]
fn sc_edit_last_slot() {
    let m = editing(60_000, "24:13", 3);
    let (m, _) = apply(m, &[(Digit(7), T0), (Digit(8), T0)]);
    assert_eq!(text_at(&m, T0), "24:18");
    assert_eq!(cursor_at(&m, T0), Some(3));
}

#[test]
fn sc_backspace() {
    let m = editing(60_000, "24:13", 3);

    let (m, _) = update(m, Backspace, T0);
    assert_eq!(text_at(&m, T0), "24:10");
    assert_eq!(cursor_at(&m, T0), Some(2));

    let (m, _) = apply(m, &[(Backspace, T0), (Backspace, T0), (Backspace, T0)]);
    assert_eq!(text_at(&m, T0), "00:00");
    assert_eq!(cursor_at(&m, T0), Some(0));
}

#[test]
fn sc_confirm() {
    let m = editing(1_452_700, "05:30", 3);
    let (m, effect) = update(m, Confirm, T1);
    assert_eq!(effect, Some(Effect::SaveStartFrom(330)));
    assert_eq!(m.start_from_secs, 330);
    assert_eq!(text_at(&m, T1 + 999), "05:30");
    assert_eq!(text_at(&m, T1 + 1000), "05:29");
}

#[test]
fn sc_confirm_unchanged() {
    let m = running(1_452_700, T0);
    assert_eq!(text_at(&m, T0), "24:13");

    let (m, effects) = apply(m, &[(Edit, T0), (Confirm, T0)]);
    assert_eq!(effects, [Effect::SaveStartFrom(1453)]);
    assert_eq!(remaining(&m, T0), 1_453_000);
}

#[test]
fn sc_confirm_zero() {
    let m = editing(60_000, "00:00", 0);
    let (m, effect) = update(m, Confirm, T1);
    assert_eq!(effect, Some(Effect::SaveStartFrom(0)));
    assert_eq!(text_at(&m, T1 + 1000), "-00:01");
    assert_eq!(look_at(&m, T1 + 1000), Some(Look::Overdue));
}

#[test]
fn sc_cancel_exact() {
    let m = running(-300, T0);
    assert_eq!(text_at(&m, T0), "00:00");

    let (m, _) = apply(m, &[(Edit, T0), (Digit(5), T0), (Cancel, T1)]);
    assert_eq!(text_at(&m, T1 + 699), "00:00");
    assert_eq!(text_at(&m, T1 + 700), "-00:01");
}

#[test]
fn sc_blur_cancels() {
    let m = running(60_000, T0);
    let (m, effects) = apply(
        m,
        &[
            (Edit, T0),
            (Digit(0), T0),
            (Digit(5), T0),
            (Digit(3), T0),
            (Digit(0), T0),
        ],
    );
    assert_eq!(text_at(&m, T0), "05:30");

    let (m, effect) = update(m, Blur, T1);
    assert_eq!(mode(&m, T1), Mode::Running);
    assert_eq!(remaining(&m, T1), 60_000);
    assert_eq!(effect, None);
    assert!(effects.is_empty());
}

#[test]
fn sc_load() {
    let accepted = [("0", 0), ("3599", 3599), ("+60", 60), ("0060", 60)];
    for (stored, secs) in accepted {
        assert_eq!(init(Some(stored), T0).start_from_secs, secs, "{stored:?}");
    }

    let rejected = [
        Some("3600"),
        Some("4294967296"),
        Some("-1"),
        Some("abc"),
        Some(""),
        Some(" 60"),
        None,
    ];
    for stored in rejected {
        assert_eq!(
            init(stored, T0).start_from_secs,
            DEFAULT_START_FROM_SECS,
            "{stored:?}"
        );
    }
}
