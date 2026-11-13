mod support;

use countdown_core::*;
use support::*;

use Input::*;

#[test]
fn t_init() {
    assert_eq!(
        init(Some("90"), T0),
        Model {
            phase: Phase::Running {
                deadline: T0 + 90_000
            },
            start_from_secs: 90,
        }
    );
}

#[test]
fn t_init_uses_default_without_stored_value() {
    let start = DEFAULT_START_FROM_SECS;
    assert_eq!(
        init(None, T0),
        Model {
            phase: Phase::Running {
                deadline: T0 + i64::from(start) * 1000
            },
            start_from_secs: start,
        }
    );
}

#[test]
fn t_noop() {
    let cases = [
        (Mode::Running, running(60_000, T0)),
        (Mode::Paused, paused(60_000)),
        (Mode::Overdue, running(-5000, T0)),
        (Mode::Editing, editing(60_000, "01:00", 1)),
    ];
    for (expected_mode, m) in cases {
        assert_eq!(mode(&m, T0), expected_mode);
        for input in ALL_INPUTS
            .into_iter()
            .filter(|&i| is_noop(expected_mode, i))
        {
            assert_eq!(
                update(m.clone(), input, T0),
                (m.clone(), None),
                "{expected_mode:?} × {input:?}"
            );
        }
    }
}

#[test]
fn t_noop_mode_evaluated_at_input_time() {
    // Running when built, Overdue by the time the input arrives.
    let m = running(1000, T0);
    for input in [Pause, Edit] {
        assert_eq!(
            update(m.clone(), input, T0 + 2000),
            (m.clone(), None),
            "{input:?}"
        );
    }
}

#[test]
fn t_run_pause() {
    let m = running(1_452_700, T0);
    let (m, effect) = update(m, Pause, T0 + 300);
    assert_eq!(
        m.phase,
        Phase::Paused {
            remaining_ms: 1_452_400
        }
    );
    assert_eq!(m.start_from_secs, TEST_START);
    assert_eq!(effect, None);
}

#[test]
fn t_run_pause_during_zero_second() {
    let (m, _) = update(running(-300, T0), Pause, T0);
    assert_eq!(m.phase, Phase::Paused { remaining_ms: -300 });
}

#[test]
fn t_pau_pause() {
    let (m, effect) = update(paused(1_452_400), Pause, T1);
    assert_eq!(
        m.phase,
        Phase::Running {
            deadline: T1 + 1_452_400
        }
    );
    assert_eq!(effect, None);
}

#[test]
fn t_run_edit() {
    let (m, effect) = update(running(1_452_700, T0), Edit, T0);
    assert_eq!(
        m.phase,
        Phase::Editing {
            remaining_ms: 1_452_700,
            draft: draft("24:13", 0),
        }
    );
    assert_eq!(effect, None);
}

#[test]
fn t_run_edit_during_zero_second() {
    let (m, _) = update(running(-300, T0), Edit, T0);
    assert_eq!(
        m.phase,
        Phase::Editing {
            remaining_ms: -300,
            draft: draft("00:00", 0),
        }
    );
}

#[test]
fn t_reset_from_running() {
    assert_eq!(
        update(running(5000, T0), Click, T0),
        (
            Model {
                phase: Phase::Running {
                    deadline: T0 + i64::from(TEST_START) * 1000
                },
                start_from_secs: TEST_START,
            },
            None,
        )
    );
}

#[test]
fn t_reset_from_overdue() {
    assert_eq!(
        update(running(-5000, T0), Click, T0),
        (
            Model {
                phase: Phase::Running {
                    deadline: T0 + i64::from(TEST_START) * 1000
                },
                start_from_secs: TEST_START,
            },
            None,
        )
    );
}

#[test]
fn t_edt_digit_overwrites_and_moves_right() {
    let (m, effect) = update(editing(60_000, "24:13", 1), Digit(9), T0);
    assert_eq!(
        m.phase,
        Phase::Editing {
            remaining_ms: 60_000,
            draft: draft("29:13", 2),
        }
    );
    assert_eq!(effect, None);
}

#[test]
fn t_edt_digit_stays_on_last_slot() {
    let (m, _) = update(editing(60_000, "24:13", 3), Digit(9), T0);
    assert_eq!(
        m.phase,
        Phase::Editing {
            remaining_ms: 60_000,
            draft: draft("24:19", 3),
        }
    );
}

#[test]
fn t_edt_digit_rejected_changes_nothing() {
    let m = editing(60_000, "24:13", 0);
    assert_eq!(update(m.clone(), Digit(6), T0), (m, None));
}

#[test]
fn t_edt_left() {
    let (m, _) = update(editing(60_000, "24:13", 2), Left, T0);
    assert_eq!(cursor_at(&m, T0), Some(1));
    assert_eq!(text_at(&m, T0), "24:13");
}

#[test]
fn t_edt_left_stops_at_first_slot() {
    let m = editing(60_000, "24:13", 0);
    assert_eq!(update(m.clone(), Left, T0), (m, None));
}

#[test]
fn t_edt_right() {
    let (m, _) = update(editing(60_000, "24:13", 1), Right, T0);
    assert_eq!(cursor_at(&m, T0), Some(2));
    assert_eq!(text_at(&m, T0), "24:13");
}

#[test]
fn t_edt_right_stops_at_last_slot() {
    let m = editing(60_000, "24:13", 3);
    assert_eq!(update(m.clone(), Right, T0), (m, None));
}

#[test]
fn t_edt_backspace() {
    let (m, _) = update(editing(60_000, "24:13", 2), Backspace, T0);
    assert_eq!(
        m.phase,
        Phase::Editing {
            remaining_ms: 60_000,
            draft: draft("24:03", 1),
        }
    );
}

#[test]
fn t_edt_backspace_at_first_slot() {
    let (m, _) = update(editing(60_000, "24:13", 0), Backspace, T0);
    assert_eq!(
        m.phase,
        Phase::Editing {
            remaining_ms: 60_000,
            draft: draft("04:13", 0),
        }
    );
}

#[test]
fn t_edt_confirm() {
    let m = editing(777, "12:34", 1);
    assert_eq!(
        update(m, Confirm, T1),
        (
            Model {
                phase: Phase::Running {
                    deadline: T1 + 754_000
                },
                start_from_secs: 754,
            },
            Some(Effect::SaveStartFrom(754)),
        )
    );
}

#[test]
fn t_edt_cancel() {
    for input in [Cancel, Blur, Edit] {
        let m = editing(1_452_700, "05:30", 2);
        assert_eq!(
            update(m, input, T1),
            (
                Model {
                    phase: Phase::Running {
                        deadline: T1 + 1_452_700
                    },
                    start_from_secs: TEST_START,
                },
                None,
            ),
            "{input:?}"
        );
    }
}
