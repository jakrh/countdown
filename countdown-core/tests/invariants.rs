//! Property tests: every model reachable from `init` through `update`
//! calls at non-decreasing times, checked at the time of each call and
//! at a later time `u`.

mod support;

use countdown_core::*;
use proptest::prelude::*;
use support::*;

use Input::*;

const MAX_TIME: i64 = 1_000_000_000_000;
/// Traces start and end in the first half of the time domain; `later()`
/// may then jump up to another half, so long spans stay in contract.
const HALF_TIME: i64 = MAX_TIME / 2;
const MAX_STEPS: usize = 40;
/// Longer than the overdue half of the display range (3_599_000 ms), so
/// traces reach `-59:59`.
const MAX_JUMP: i64 = 4_000_000;

#[derive(Clone, Debug)]
struct Trace {
    stored: Option<String>,
    t0: i64,
    /// Each input with the time elapsed since the previous call.
    steps: Vec<(Input, i64)>,
}

struct Step {
    before: Model,
    input: Input,
    at: i64,
    after: Model,
    effect: Option<Effect>,
}

fn input() -> impl Strategy<Value = Input> {
    prop_oneof![
        1 => Just(Pause),
        1 => Just(Edit),
        1 => Just(Click),
        3 => digit().prop_map(Digit),
        1 => Just(Left),
        1 => Just(Right),
        1 => Just(Backspace),
        1 => Just(Confirm),
        1 => Just(Cancel),
        1 => Just(Blur),
    ]
}

/// Biased toward the edges: 0 (for `00:00`), 5 and 9 (for `59:59`), and
/// the invalid 10..=12.
fn digit() -> impl Strategy<Value = u8> {
    prop_oneof![
        2 => 0u8..=12,
        1 => prop::sample::select(vec![0u8, 5, 9]),
    ]
}

fn editor_input() -> impl Strategy<Value = Input> {
    input().prop_filter("not an editor exit", |i| {
        !matches!(i, Edit | Confirm | Cancel | Blur)
    })
}

fn gap() -> impl Strategy<Value = i64> {
    prop_oneof![8 => 0..=2_000i64, 1 => 0..=MAX_JUMP]
}

/// The offset of a check after the last call. Rarely spans most of the
/// time domain, to catch narrow integer types.
fn later() -> impl Strategy<Value = i64> {
    prop_oneof![9 => 0..=MAX_JUMP, 1 => 0..=HALF_TIME]
}

/// Includes 0..=3 s, which reaches the zero second (shows `00:00` with
/// `r < 0`) early, and 3599, the edge of INV-R-BOUND.
fn stored() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        (0u32..=4000).prop_map(|n| Some(n.to_string())),
        (0u32..=3).prop_map(|n| Some(n.to_string())),
        Just(Some("3599".to_owned())),
        ".{0,6}".prop_map(Some),
    ]
}

/// Numbers dressed with signs, padding and whitespace, plus arbitrary text.
/// The fixed edge cases are too rare to arise by chance; `"-0"` tells
/// `u32` parsing apart from signed parsing.
fn stored_text() -> impl Strategy<Value = String> {
    let affix = |options: &[&'static str]| prop::sample::select(options.to_vec());
    let edges = ["-0", "+0", "0", "3599", "3600", "-3599", " 0", "0 "];
    prop_oneof![
        affix(&edges).prop_map(str::to_owned),
        (
            affix(&["", "+", "-", " ", "0", "00"]),
            0u32..=5000,
            affix(&["", " ", "\n", ".0", "x"])
        )
            .prop_map(|(pre, n, post)| format!("{pre}{n}{post}")),
        ".{0,8}",
    ]
}

fn trace() -> impl Strategy<Value = Trace> {
    let latest_t0 = HALF_TIME - (MAX_STEPS as i64 + 2) * MAX_JUMP;
    (
        stored(),
        0..=latest_t0,
        prop::collection::vec((input(), gap()), 0..=MAX_STEPS),
    )
        .prop_map(|(stored, t0, steps)| Trace { stored, t0, steps })
}

fn run(tr: &Trace) -> Vec<Step> {
    let mut m = init(tr.stored.as_deref(), tr.t0);
    let mut t = tr.t0;
    tr.steps
        .iter()
        .map(|&(input, dt)| {
            t += dt;
            let (after, effect) = update(m.clone(), input, t);
            let step = Step {
                before: m.clone(),
                input,
                at: t,
                after: after.clone(),
                effect,
            };
            m = after;
            step
        })
        .collect()
}

/// Every reachable model with the time of the call that produced it.
fn states(tr: &Trace) -> Vec<(Model, i64)> {
    let first = (init(tr.stored.as_deref(), tr.t0), tr.t0);
    std::iter::once(first)
        .chain(run(tr).into_iter().map(|s| (s.after, s.at)))
        .collect()
}

fn is_timer(m: &Model, t: i64) -> bool {
    matches!(view(m, t).kind, ViewKind::Timer { .. })
}

proptest! {
    #[test]
    fn p_load(stored in stored_text()) {
        let expected = stored
            .parse::<u32>()
            .ok()
            .filter(|&n| n <= 3599)
            .unwrap_or(DEFAULT_START_FROM_SECS);
        prop_assert_eq!(init(Some(&stored), 0).start_from_secs, expected, "{:?}", stored);
    }

    #[test]
    fn inv_start_range(tr in trace()) {
        for (m, _) in states(&tr) {
            prop_assert!(m.start_from_secs <= 3599, "{m:?}");
        }
    }

    #[test]
    fn inv_r_bound(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            for u in [t, t + later] {
                prop_assert!(remaining(&m, u) <= 3_599_000, "{m:?} at {u}");
            }
        }
    }

    #[test]
    fn inv_display_range(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            for u in [t, t + later] {
                if is_timer(&m, u) {
                    let s = shown_at(&m, u);
                    prop_assert!((-3599..=3599).contains(&s), "{m:?} at {u}: {s}");
                }
            }
        }
    }

    #[test]
    fn inv_draft_valid(tr in trace()) {
        for (m, _) in states(&tr) {
            if let Phase::Editing { draft, .. } = &m.phase {
                let [m1, m2, s1, s2] = draft.digits;
                prop_assert!(m1 <= 5 && m2 <= 9 && s1 <= 5 && s2 <= 9, "{draft:?}");
                prop_assert!(draft.cursor <= 3, "{draft:?}");
            }
        }
    }

    #[test]
    fn inv_noop(tr in trace(), later in later()) {
        let digits = (0u8..=12).map(Digit);
        let inputs: Vec<Input> = ALL_INPUTS.into_iter().chain(digits).collect();
        for (m, t) in states(&tr) {
            for u in [t, t + later] {
                let md = mode(&m, u);
                for &input in inputs.iter().filter(|&&i| is_noop(md, i)) {
                    prop_assert_eq!(
                        update(m.clone(), input, u),
                        (m.clone(), None),
                        "{:?} × {:?} at {}", md, input, u
                    );
                }
            }
        }
    }

    #[test]
    fn inv_effect_only_confirm(tr in trace()) {
        for s in run(&tr) {
            let applies_confirm = mode(&s.before, s.at) == Mode::Editing && s.input == Confirm;
            prop_assert_eq!(s.effect.is_some(), applies_confirm, "{:?} at {}", s.input, s.at);
        }
    }

    #[test]
    fn inv_pause_static(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            if mode(&m, t) == Mode::Paused {
                prop_assert_eq!(view(&m, t), view(&m, t + later));
            }
        }
    }

    #[test]
    fn inv_edit_static(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            if mode(&m, t) == Mode::Editing {
                prop_assert_eq!(view(&m, t), view(&m, t + later));
            }
        }
    }

    #[test]
    fn inv_pause_roundtrip(tr in trace(), wait in 0..=MAX_JUMP, held in 0..=MAX_JUMP) {
        for (m, t) in states(&tr) {
            let t1 = t + wait;
            let t2 = t1 + held;
            let Phase::Running { deadline } = m.phase else { continue };
            if mode(&m, t1) != Mode::Running {
                continue;
            }
            let (after, _) = apply(m.clone(), &[(Pause, t1), (Pause, t2)]);
            prop_assert_eq!(after.phase, Phase::Running { deadline: deadline + (t2 - t1) });
            prop_assert_eq!(after.start_from_secs, m.start_from_secs);
        }
    }

    #[test]
    fn inv_edit_roundtrip(
        tr in trace(),
        wait in 0..=MAX_JUMP,
        held in 0..=MAX_JUMP,
        inner in prop::collection::vec((editor_input(), 0..=1000i64), 0..=10),
        exit in prop_oneof![Just(Edit), Just(Cancel), Just(Blur)],
    ) {
        let mut inner = inner;
        inner.sort_by_key(|&(_, permille)| permille);
        for (m, t) in states(&tr) {
            let t1 = t + wait;
            let t2 = t1 + held;
            let Phase::Running { deadline } = m.phase else { continue };
            if mode(&m, t1) != Mode::Running {
                continue;
            }
            let steps: Vec<(Input, i64)> = std::iter::once((Edit, t1))
                .chain(inner.iter().map(|&(i, permille)| (i, t1 + held * permille / 1000)))
                .chain(std::iter::once((exit, t2)))
                .collect();
            let (after, _) = apply(m.clone(), &steps);
            prop_assert_eq!(after.phase, Phase::Running { deadline: deadline + (t2 - t1) });
            prop_assert_eq!(after.start_from_secs, m.start_from_secs);
        }
    }

    #[test]
    fn inv_confirm_fresh(tr in trace()) {
        for s in run(&tr) {
            if let Some(Effect::SaveStartFrom(n)) = s.effect {
                prop_assert_eq!(remaining(&s.after, s.at), i64::from(n) * 1000);
            }
        }
    }

    #[test]
    fn inv_wake_none(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            for u in [t, t + later] {
                let md = mode(&m, u);
                let static_view = matches!(md, Mode::Paused | Mode::Editing);
                let at_floor = !static_view && shown_at(&m, u) == -3599;
                prop_assert_eq!(
                    next_wakeup(&m, u).is_none(),
                    static_view || at_floor,
                    "{:?} at {}", m, u
                );
            }
        }
    }

    #[test]
    fn inv_wake_exact(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            for u in [t, t + later] {
                let Some(d) = next_wakeup(&m, u) else { continue };
                let d = i64::from(d);
                let s = shown_at(&m, u);
                prop_assert_eq!(shown_at(&m, u + d), s - 1, "{:?} at {} + {}", m, u, d);
                prop_assert_eq!(shown_at(&m, u + d - 1), s, "{:?} at {} + {} - 1", m, u, d);
            }
        }
    }

    #[test]
    fn f_wake_range(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            for u in [t, t + later] {
                if let Some(d) = next_wakeup(&m, u) {
                    prop_assert!((1..=1000).contains(&d), "{m:?} at {u}: {d}");
                }
            }
        }
    }

    #[test]
    fn inv_look(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            for u in [t, t + later] {
                let expected = match mode(&m, u) {
                    Mode::Running => Some(Look::Normal),
                    Mode::Paused => Some(Look::Frozen),
                    Mode::Overdue => Some(Look::Overdue),
                    Mode::Editing => None,
                };
                prop_assert_eq!(look_at(&m, u), expected, "{:?} at {}", m, u);
            }
        }
    }

    #[test]
    fn inv_clickable(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            for u in [t, t + later] {
                let expected = matches!(mode(&m, u), Mode::Running | Mode::Overdue);
                prop_assert_eq!(view(&m, u).clickable, expected, "{:?} at {}", m, u);
            }
        }
    }

    /// Finds the next change of the displayed number by bisection, so the
    /// check does not depend on `next_wakeup`.
    #[test]
    fn inv_blink_aligned(tr in trace(), later in later()) {
        for (m, t) in states(&tr) {
            let u = t + later;
            if mode(&m, u) != Mode::Overdue {
                continue;
            }
            let s = shown_at(&m, u);
            if shown_at(&m, u + 1000) == s {
                prop_assert_eq!(s, -3599, "{:?} stalled at {}", m, u);
                continue;
            }
            // Invariant: shown(lo) == s, shown(hi) != s.
            let (mut lo, mut hi) = (u, u + 1000);
            while hi - lo > 1 {
                let mid = lo + (hi - lo) / 2;
                if shown_at(&m, mid) == s { lo = mid } else { hi = mid }
            }
            let onset = onset_at(&m, hi);
            prop_assert!(onset.is_some(), "{m:?} at {hi}");
            let phase = (hi - onset.unwrap()).rem_euclid(OVERDUE_BLINK_PERIOD_MS);
            prop_assert_eq!(phase, 0, "{:?}: change at {}", m, hi);
        }
    }
}
