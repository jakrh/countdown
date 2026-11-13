# Countdown Behavior Spec

This is the normative behavior spec: the implementation and the tests
follow it. [design.md](design.md) explains the principles behind it
and the reasons for the main decisions.

## 0. Conventions

- MUST, MUST NOT and MAY are used as in RFC 2119.
- Every requirement has a stable ID. Tests are named after the ID they
  verify (e.g. `T-RUN-PAUSE` → `t_run_pause`).
  - `F-`: formula
  - `T-`: transition
  - `INV-`: invariant (property test)
  - `SC-`: scenario (example test)
  - `P-`: persistence
  - `A-`: architecture constraint
  - `SH-`: shell requirement (manual smoke test)
  - `W-`: window and config requirement (manual smoke test)
- Time is whole milliseconds (`i64`). The shell floors
  `performance.now()` before passing it in (SH-CLOCK), so the core never
  sees fractional time: every formula is exact integer arithmetic and
  tests compare with `==`. Floating-point time is avoided because
  `(now + x) - now != x` for fractional `now`, which made the display
  show one second too many right after a reset about 10% of the time.
- `ceil(a / b)` is the mathematical ceiling of the exact quotient, not
  Rust's truncating `/`. `mod` is Euclidean (`rem_euclid`).
- Identifiers in `code` are the names used in the source.
- Concrete numbers in scenarios are examples. Rules and tests refer to
  tunables by name (§2), never by their current value.

## 1. Vocabulary

```rust
struct Model { phase: Phase, start_from_secs: u32 }

enum Phase {
    Running { deadline: i64 },
    Paused  { remaining_ms: i64 },
    Editing { remaining_ms: i64, draft: Draft },
}

/// `digits` is `[minute tens, minute ones, second tens, second ones]`.
struct Draft { digits: [u8; 4], cursor: u8 }

enum Mode  { Running, Paused, Overdue, Editing }   // derived, see F-MODE
enum Input { Pause, Edit, Click, Digit(u8), Left, Right, Backspace, Confirm, Cancel, Blur }
enum Look  { Normal, Frozen, Overdue }
enum Effect { SaveStartFrom(u32) }

struct View { text: String, clickable: bool, kind: ViewKind }
enum ViewKind {
    Timer  { look: Look, overdue_onset: Option<i64> },
    Editor { cursor: u8 },
}

fn init(stored: Option<&str>, now: i64) -> Model
fn update(m: Model, input: Input, now: i64) -> (Model, Option<Effect>)
fn view(m: &Model, now: i64) -> View
fn next_wakeup(m: &Model, now: i64) -> Option<u32>
fn mode(m: &Model, now: i64) -> Mode
```

- These items are the core's public API. All types derive `Clone`,
  `Debug` and `PartialEq`, and all struct fields are `pub`, so tests can
  build any model directly and compare results.

- **Start-from time** (`start_from_secs`): where the countdown starts.
  Used at launch and on reset; changed and saved only by `Confirm`.

## 2. Tunables

Each value is defined once, as one constant in code.

| Constant | Current value | Constraint |
|---|---|---|
| `DEFAULT_START_FROM_SECS` | 1500 (25:00) | MUST be in `0..=3599` |
| `OVERDUE_BLINK_PERIOD_MS` | 1000 | MUST divide 1000, so every number change starts a visible half |
| `CURSOR_BLINK_PERIOD_MS` | 1000 | none |
| `COLOR_NORMAL` | `white` | none |
| `COLOR_FROZEN` | `darkturquoise` | none |
| `COLOR_OVERDUE` | `red` | none |
| `COLOR_CURSOR_TEXT` | `black` | none |

Not tunable: the display range −3599..=3599 and the start-from range
0..=3599. Both are fixed by the `MM:SS` format.

## 3. Formulas

- **F-REM**: the remaining time `r`, in ms, is `deadline - now` for
  `Running`, and `remaining_ms` for `Paused` and `Editing`.
- **F-SEC**: the displayed seconds `s = clamp(ceil(r / 1000), -3599, 3599)`.
- **F-MODE**: `Phase::Running` with `s < 0` → `Mode::Overdue`;
  `Phase::Running` with `s ≥ 0` → `Mode::Running`; `Phase::Paused` →
  `Mode::Paused`; `Phase::Editing` → `Mode::Editing`. The mode is
  evaluated at the `now` of the call; it is never stored.
- **F-TEXT**: outside `Editing`, `text` is `MM:SS` for `s ≥ 0` and
  `-MM:SS` for `s < 0`, with `MM = |s| / 60` and `SS = |s| % 60`, both
  zero-padded to two digits. In `Editing`, `text` is the draft as
  `D0D1:D2D3`.
- **F-DRAFT-SECS**: a draft's value is
  `(d0 * 10 + d1) * 60 + d2 * 10 + d3` seconds.
- **F-DRAFT-FROM**: the draft for `s` in `0..=3599` has digits
  `[m / 10, m % 10, sec / 10, sec % 10]` with `m = s / 60` and
  `sec = s % 60`, and `cursor = 0`.
- **F-ACCEPTS**: slot `i` accepts digit `d` if `d ≤ 9`, and also `d ≤ 5`
  when `i` is 0 or 2.
- **F-LOOK**: `Mode::Paused` → `Look::Frozen`; `Mode::Overdue` →
  `Look::Overdue`; `Mode::Running` → `Look::Normal`. `Editing` has no
  look; its view is `ViewKind::Editor`.
- **F-CLICKABLE**: `clickable` is true if and only if the mode is
  `Running` or `Overdue`.
- **F-ONSET**: for `Phase::Running`, the overdue onset is
  `deadline + 1000`, the first instant at which `s < 0`.
  `overdue_onset` is `Some(onset)` in `Mode::Overdue` and `None`
  otherwise.
- **F-BLINK**: in `Mode::Overdue`, the text is visible at `now` if and
  only if `(now - onset) mod OVERDUE_BLINK_PERIOD_MS` is less than half
  the period. The shell realizes this in CSS (SH-OVERDUE-BLINK); the
  core only supplies `onset`.
- **F-WAKE**: in `Mode::Running` and `Mode::Overdue` with `s > -3599`,
  `next_wakeup` is `Some(r - (s - 1) * 1000)`, which lies in
  `1..=1000`. Otherwise (`Paused`, `Editing`, or `s == -3599`) it is
  `None`.

## 4. Transitions

- **T-INIT**: `init(stored, now)` returns `start_from_secs` per P-LOAD
  and phase `Running { deadline: now + start_from_secs * 1000 }`.

The mode is evaluated per F-MODE at the input's `now`. `—` is
**T-NOOP**: the model is returned unchanged and no effect is produced.

| Input | Running | Paused | Overdue | Editing |
|---|---|---|---|---|
| `Pause` | T-RUN-PAUSE | T-PAU-PAUSE | — | — |
| `Edit` | T-RUN-EDIT | — | — | — |
| `Click` | T-RESET | — | T-RESET | — |
| `Digit(d)` | — | — | — | T-EDT-DIGIT |
| `Left` | — | — | — | T-EDT-LEFT |
| `Right` | — | — | — | T-EDT-RIGHT |
| `Backspace` | — | — | — | T-EDT-BACKSPACE |
| `Confirm` | — | — | — | T-EDT-CONFIRM |
| `Cancel` | — | — | — | T-EDT-CANCEL |
| `Blur` | — | — | — | T-EDT-CANCEL |

Only T-EDT-CONFIRM produces an effect.

- **T-RUN-PAUSE**: phase ← `Paused { remaining_ms: deadline - now }`.
- **T-PAU-PAUSE**: phase ← `Running { deadline: now + remaining_ms }`.
- **T-RUN-EDIT**: phase ← `Editing { remaining_ms: deadline - now, draft }`,
  with the draft from F-DRAFT-FROM for the current `s`.
- **T-RESET**: phase ← `Running { deadline: now + start_from_secs * 1000 }`.
- **T-EDT-DIGIT**: if the cursor's slot accepts `d` (F-ACCEPTS),
  `digits[cursor] ← d` and `cursor ← min(cursor + 1, 3)`; otherwise
  nothing changes.
- **T-EDT-LEFT**: `cursor ← max(cursor - 1, 0)`. Here and in
  T-EDT-BACKSPACE the subtraction saturates at 0; it MUST NOT underflow
  `u8`.
- **T-EDT-RIGHT**: `cursor ← min(cursor + 1, 3)`.
- **T-EDT-BACKSPACE**: `digits[cursor] ← 0`, then
  `cursor ← max(cursor - 1, 0)`.
- **T-EDT-CONFIRM**: with `n` from F-DRAFT-SECS, phase ←
  `Running { deadline: now + n * 1000 }`, `start_from_secs ← n`, effect
  `SaveStartFrom(n)`. `remaining_ms` is discarded.
- **T-EDT-CANCEL**: phase ← `Running { deadline: now + remaining_ms }`.
  The draft is discarded.

## 5. Invariants

These hold for every model reachable from `init` through any sequence of
`update` calls with non-decreasing `now`, and for every `t` at or after
the last call. Property-test generators draw every time from
`0..=10^12` ms; values outside that range are out of contract.

- **INV-START-RANGE**: `start_from_secs` is in `0..=3599`.
- **INV-R-BOUND**: `r ≤ 3_599_000`, so the upper clamp of F-SEC never
  engages.
- **INV-DISPLAY-RANGE**: `s` is in `-3599..=3599`.
- **INV-DRAFT-VALID**: in `Editing`, every slot satisfies F-ACCEPTS and
  `cursor` is in `0..=3`. Hence the draft's value (F-DRAFT-SECS) is at
  most 3599, which INV-START-RANGE and INV-R-BOUND rely on.
- **INV-NOOP**: every `—` cell of §4 returns the model unchanged with no
  effect.
- **INV-EFFECT-ONLY-CONFIRM**: `update` returns `Some(_)` if and only if
  it applied T-EDT-CONFIRM.
- **INV-PAUSE-STATIC**: in `Mode::Paused`, `view(m, t)` is the same for
  every `t`.
- **INV-EDIT-STATIC**: in `Mode::Editing`, `view(m, t)` is the same for
  every `t`.
- **INV-PAUSE-ROUNDTRIP**: if `m.phase` is `Running { deadline }` in
  `Mode::Running` at `t1` and `t2 ≥ t1`, then `Pause` at `t1` and
  `Pause` at `t2` yield `Running { deadline: deadline + (t2 - t1) }`.
- **INV-EDIT-ROUNDTRIP**: if `m.phase` is `Running { deadline }` in
  `Mode::Running` at `t1` and `t2 ≥ t1`, then `Edit` at `t1`, followed
  by any inputs other than `Confirm`, `Cancel` and `Blur` at
  non-decreasing times in `[t1, t2]`, and finally `Cancel` or `Blur` at
  `t2`, yield `Running { deadline: deadline + (t2 - t1) }` with
  `start_from_secs` unchanged.
- **INV-CONFIRM-FRESH**: right after T-EDT-CONFIRM at `t` with value
  `n`, `r` at `t` is exactly `n * 1000`, whatever `remaining_ms` was.
- **INV-WAKE-NONE**: `next_wakeup` is `None` if and only if the mode is
  `Paused` or `Editing`, or `s == -3599`.
- **INV-WAKE-EXACT**: if `next_wakeup(m, t) == Some(d)`, then `s` at
  `t + d` is `s` at `t` minus 1, and `s` at `t + d - 1` equals `s` at
  `t`. The display changes exactly at the wakeup, never earlier.
- **INV-LOOK**: in a `ViewKind::Timer` view, `look` follows F-LOOK; in
  particular `Look::Frozen` appears if and only if the mode is `Paused`.
- **INV-CLICKABLE**: `view.clickable` follows F-CLICKABLE.
- **INV-BLINK-ALIGNED**: in `Mode::Overdue`, at every `t` where `s` at
  `t` differs from `s` at `t - 1`,
  `(t - onset) mod OVERDUE_BLINK_PERIOD_MS == 0`, with `onset` from
  `view(m, t).kind`'s `overdue_onset`. So every number change starts a
  visible half per F-BLINK.

## 6. Scenarios

Each starts from a model built for the test; `t0` is an arbitrary time
and `t1 ≥ t0`. Where a step gives no time, it happens at the previous
step's time.

**SC-LAUNCH-DEFAULT**
- Given `init(None, t0)`
- Then `start_from_secs == DEFAULT_START_FROM_SECS`, the mode is
  `Running`, and `r` at `t0` is `DEFAULT_START_FROM_SECS * 1000`

**SC-FIRST-SECOND**
- Given `init(Some("600"), t0)`
- Then at `t0 + 999` it shows `10:00` with `Look::Normal`
- And at `t0 + 1000` it shows `09:59`

**SC-ZERO-THEN-OVERDUE**
- Given `Running` with `r = 1000` ms at `t0` (shows `00:01`)
- Then at `t0 + 999` it still shows `00:01`
- And at `t0 + 1000` it shows `00:00` with `Look::Normal`
- And at `t0 + 1999` it still shows `00:00` with `Look::Normal`
- And at `t0 + 2000` it shows `-00:01` with `Look::Overdue`

**SC-CLAMP**
- Given `Running` with `r = -3_599_500` ms at `t0` (shows `-59:59`)
- Then `next_wakeup` is `None`, and at `t0 + 10_000_000` it still shows
  `-59:59` with `Look::Overdue`
- When `Click` at `t0 + 10_000_000`
- Then the mode is `Running` and `r` is `start_from_secs * 1000`

**SC-PAUSE-KEEPS-SUBSECOND**
- Given `Running` with `r = 1_452_700` ms at `t0` (shows `24:13`)
- When `Pause` at `t0`, then `Pause` again at `t0 + 5000`
- Then at `t0 + 5699` it shows `24:13` with `Look::Normal`
- And at `t0 + 5700` it shows `24:12`

**SC-PAUSE-DURING-ZERO**
- Given `Running` with `r = -300` ms at `t0` (shows `00:00`, `Mode::Running`)
- When `Pause` at `t0`
- Then it shows `00:00` with `Look::Frozen`
- When `Pause` again at `t1`
- Then at `t1 + 699` it shows `00:00` with `Look::Normal`
- And at `t1 + 700` it shows `-00:01` with `Look::Overdue`

**SC-NO-PAUSE-WHEN-OVERDUE**
- Given `Running` with `r = -5000` ms at `t0` (shows `-00:05`)
- When `Pause` at `t0`
- Then the model is unchanged

**SC-EDIT-OVERWRITE-ALL**
- Given `Running` with `r = 1_452_700` ms at `t0` (shows `24:13`)
- When `Edit` at `t0`, then `Digit(0)`, `Digit(5)`, `Digit(3)`, `Digit(0)`
- Then the text is `05:30` and `cursor == 3`

**SC-EDIT-ONE-SLOT**
- Given `Editing` with draft `24:13`, `cursor == 0`
- When `Right`, `Right`, `Digit(4)`
- Then the text is `24:43` and `cursor == 3`

**SC-EDIT-REJECT-TENS**
- Given `Editing` with draft `24:13`, `cursor == 2`
- When `Digit(6)`
- Then the draft and cursor are unchanged

**SC-EDIT-LAST-SLOT**
- Given `Editing` with draft `24:13`, `cursor == 3`
- When `Digit(7)`, then `Digit(8)`
- Then the text is `24:18` and `cursor == 3`

**SC-BACKSPACE**
- Given `Editing` with draft `24:13`, `cursor == 3`
- When `Backspace`
- Then the text is `24:10` and `cursor == 2`
- When `Backspace` three more times
- Then the text is `00:00` and `cursor == 0`

**SC-CONFIRM**
- Given `Editing` with draft `05:30` and `remaining_ms = 1_452_700`
- When `Confirm` at `t1`
- Then the effect is `SaveStartFrom(330)`, `start_from_secs == 330`
- And at `t1 + 999` it shows `05:30`, and at `t1 + 1000` it shows `05:29`

**SC-CONFIRM-UNCHANGED**
- Given `Running` with `r = 1_452_700` ms at `t0` (shows `24:13`)
- When `Edit` at `t0`, then `Confirm` at `t0`
- Then the effect is `SaveStartFrom(1453)` and `r` at `t0` is `1_453_000`

**SC-CONFIRM-ZERO**
- Given `Editing` with draft `00:00`
- When `Confirm` at `t1`
- Then the effect is `SaveStartFrom(0)`, and at `t1 + 1000` it shows
  `-00:01` with `Look::Overdue`

**SC-CANCEL-EXACT**
- Given `Running` with `r = -300` ms at `t0` (shows `00:00`)
- When `Edit` at `t0`, `Digit(5)`, then `Cancel` at `t1`
- Then at `t1 + 699` it shows `00:00`, and at `t1 + 700` it shows `-00:01`

**SC-BLUR-CANCELS**
- Given `Editing` entered at `t0` from `r = 60_000`, draft changed to `05:30`
- When `Blur` at `t1`
- Then the mode is `Running`, `r` at `t1` is `60_000`, and there is no effect

**SC-LOAD**
- `init(Some("0"), t0)` → `start_from_secs == 0`
- `init(Some("3599"), t0)` → `3599`
- `init(Some("+60"), t0)` and `init(Some("0060"), t0)` → `60`
- For each of `Some("3600")`, `Some("4294967296")` (`u32::MAX + 1`),
  `Some("-1")`, `Some("abc")`, `Some("")`, `Some(" 60")`, `None`:
  `start_from_secs == DEFAULT_START_FROM_SECS`

## 7. Persistence

- **P-KEY**: the start-from time is stored in `window.localStorage`
  under the key `countdown_start_from_seconds`, as a base-10 string.
- **P-LOAD**: at startup the shell reads the key and passes the result
  to `init` as `Some(value)`, or `None` when the key is missing or
  storage cannot be read. `init` accepts a value only if
  `value.parse::<u32>()` succeeds and the result is at most 3599;
  otherwise it uses `DEFAULT_START_FROM_SECS`. Non-canonical forms that
  `parse` accepts, such as `"+60"` and `"0060"`, load as their value;
  the app only ever writes canonical strings, so this is harmless.
- **P-SAVE**: on `Effect::SaveStartFrom(n)` the shell writes `n`. A
  failed write is ignored and not logged.
- **P-NO-MIGRATION**: the old key `countdown_timer_remaining_seconds`
  MUST NOT be read or written.

## 8. Shell

### Architecture

- **A-CORE-PURE**: the core (the §1 API, the §2 tunables it uses, and
  the §3–§5 logic; tunables only the shell uses live in the shell)
  is its own workspace crate, `countdown-core`, with an empty
  `[dependencies]` table and no access to any clock. The frontend crate depends on it. The
  dependency graph, not discipline, keeps the core free of `web-sys`
  and `wasm-bindgen`. Core tests run natively with
  `cargo test -p countdown-core`, without building the frontend or
  `dist/`. All `F-`, `T-`, `INV-`, `SC-` and `P-LOAD` tests live in
  this crate. `INV-` tests use `proptest`, which is only a
  `[dev-dependencies]` entry; it shrinks a failure to a minimal input
  sequence.
- **A-NO-FRAMEWORK**: the shell uses `web-sys` directly; no UI framework.
  `index.html` holds the static structure: the container
  (`id="container"`) and inside it the time element (`id="time"`). The
  time element's children are rebuilt from the view on every render
  (SH-OVERDUE-BLINK, SH-CURSOR-BLINK).
- **A-NON-GOAL-SLEEP**: behavior across system sleep is not specified.
  Whether `performance.now()` advances during sleep differs by platform,
  and the app neither detects nor compensates for it.

### Input

- **SH-KEYMAP**: the shell maps `keydown` on `window` by `event.code`:

  | `code` | `Input` |
  |---|---|
  | `KeyP` | `Pause` |
  | `KeyF` | `Edit` |
  | `Digit0`–`Digit9`, `Numpad0`–`Numpad9` | `Digit(0)`–`Digit(9)` |
  | `ArrowLeft` | `Left` |
  | `ArrowRight` | `Right` |
  | `Backspace` | `Backspace` |
  | `Enter`, `NumpadEnter` | `Confirm` |
  | `Escape` | `Cancel` |

- **SH-KEY-FILTER**: a `keydown` with `repeat == true`, or with any of
  `shiftKey`, `ctrlKey`, `altKey`, `metaKey` set, MUST NOT be
  dispatched.
- **SH-KEY-PREVENT**: a key that is dispatched MUST have
  `preventDefault()` called. Every other key, including mapped keys
  dropped by SH-KEY-FILTER, MUST be left alone.
- **SH-NO-MODE-LOGIC**: the shell MUST NOT look at the mode to decide
  whether to dispatch an input. Ignoring inputs is the core's job (§4).
- **SH-CLICK**: a primary-button `click` on the time element MUST
  dispatch `Input::Click`. Clicks elsewhere MUST NOT.
- **SH-BLUR**: a `blur` event on `window` MUST dispatch `Input::Blur`.
- **SH-CONTEXTMENU**: the page's context menu MUST be suppressed
  everywhere in the page. On Windows the drag region is non-client, so
  its native window menu (Move, Close) still appears; that is intended,
  being the only mouse route to Move and Close on an undecorated window.

### Dispatch and timing

- **SH-CLOCK**: for each event (load, input, timeout,
  `visibilitychange`) the shell reads `performance.now()` once, floors
  it to `i64`, and passes that one value to every core call it makes
  for the event.
- **SH-INIT**: on load the shell reads storage (P-LOAD), calls `init`,
  renders, and schedules the first wakeup.
- **SH-DISPATCH**: after every dispatch the shell MUST handle any effect
  (P-SAVE), render `view(m, now)`, cancel any pending timeout, and
  schedule at most one new timeout from `next_wakeup(m, now)`.
- **SH-WAKE**: when the timeout fires, the shell MUST render and
  reschedule as in SH-DISPATCH. Firing early or late needs no special
  handling, because both are recomputed from `now`.
- **SH-VISIBLE**: when `visibilitychange` reports the page visible, the
  shell MUST render and reschedule as in SH-DISPATCH.

### Rendering

- **SH-TEXT**: the time element shows `view.text`.
- **SH-LOOK**: in `ViewKind::Timer`, the text color is `COLOR_NORMAL`,
  `COLOR_FROZEN` or `COLOR_OVERDUE` for `Look::Normal`, `Look::Frozen`,
  `Look::Overdue`.
- **SH-OVERDUE-BLINK**: in `Look::Overdue`, the text blinks by a CSS
  animation: `opacity: 1` for the first half of
  `OVERDUE_BLINK_PERIOD_MS`, `opacity: 0` for the second, switching
  hard, repeating forever. Every render of an `Overdue` view MUST start
  a fresh animation with
  `animation-delay: -((now - overdue_onset) mod OVERDUE_BLINK_PERIOD_MS)`
  so that it matches F-BLINK, with no memory of the previous render.
  Rebuilding the element that carries the animation does this. Changing
  `animation-delay` on an animation already running does not restart it
  and would shift its phase. A phase error of up to one frame is
  acceptable. Only the text blinks; the background stays.
  The text MUST stay clickable while hidden, so `visibility` and
  `display` MUST NOT be used.
- **SH-POINTER**: the time element uses `cursor: pointer` while
  `view.clickable` is true, and the default cursor otherwise.
- **SH-EDITOR**: in `ViewKind::Editor`, the time element is drawn inside
  a frame (`1px solid white` border, `rgba(0, 0, 0, 0.7)` background,
  `4px` radius) with the same font and position as the timer. The colon
  is always shown. The text is `COLOR_NORMAL` with no shadow: the
  frame's dark background already keeps it readable, and the SH-STYLE
  shadow smears around the dark digit in the cursor block. The digit at
  `cursor` is an inverted block: background `COLOR_NORMAL`, text
  `COLOR_CURSOR_TEXT`.
- **SH-CURSOR-BLINK**: the inverted block blinks by a CSS animation that
  switches between inverted and normal every half of
  `CURSOR_BLINK_PERIOD_MS`; the digit itself never disappears. Every
  render of `ViewKind::Editor` MUST restart the animation from the
  inverted half, with no comparison against the previous render.
  Rebuilding the digit elements on each render does this, since a new
  element starts its animation from the beginning. Editing schedules no
  wakeups (INV-WAKE-NONE), so renders come only from SH-DISPATCH and
  SH-VISIBLE; a rejected digit or an arrow at an end therefore restarts
  the blink too, which is intended.
- **SH-STYLE**: the text is bold `'Courier New', Courier, monospace` at
  `1.8em` with a `0 0 5px black` shadow repeated four times, and cannot
  be selected. The container fills the window, centers the time, and has
  a `rgba(0, 0, 0, 0.2)` background with an `8px` radius; the page
  background is transparent.

### Window dragging

- **SH-DRAG**: the container MUST carry `data-tauri-drag-region="deep"`
  and `-webkit-app-region: drag`. The time element MUST carry
  `data-tauri-drag-region="false"` and `-webkit-app-region: no-drag`.
  - `-webkit-app-region` works only in Chromium (Windows); WKWebView
    ignores it, so macOS and Linux need the Tauri attribute.
  - On macOS a started drag swallows `mouseup`, so the time element must
    not start one or its click would never fire.
  - The two mechanisms do not conflict: where Chromium makes the region
    non-client, the page never sees the `mousedown`.

## 9. Window and config

- **W-WINDOW**: `tauri.conf.json` defines one window with exactly these
  settings: `title: "countdown"`, `center: true`, `width: 110`,
  `height: 52`, `acceptFirstMouse: true`, `transparent: true`,
  `decorations: false`, `alwaysOnTop: true`, `skipTaskbar: true`,
  `resizable: false`, `theme: "Dark"`. Settings with no effect and
  settings equal to Tauri's defaults are omitted.
- **W-ONE-CONFIG**: there are no platform config files, and
  `dragDropEnabled` keeps its default.
- **W-ACTIVATION**: on macOS, the app sets
  `ActivationPolicy::Accessory` in Tauri's `setup`, which hides it from
  the Dock and Cmd+Tab. It MUST be set there: tao re-applies the policy
  in `applicationDidFinishLaunching` and discards what `Info.plist` set.
  `Info.plist` keeps `LSUIElement = true` only to shrink the Dock-icon
  flash before that point.
- **W-CAPABILITIES**: the main window's capability grants exactly
  `core:window:allow-start-dragging`,
  `core:window:allow-internal-toggle-maximize` and
  `core:webview:allow-internal-toggle-devtools`. These are the only
  commands Tauri's injected scripts send:
  - The drag-region script sends `start_dragging` on press and
    `internal_toggle_maximize` on double-click (a no-op, since the
    window is not resizable).
  - In debug builds, the devtools hotkey (Cmd+Option+I on macOS,
    Ctrl+Shift+I elsewhere) sends `internal_toggle_devtools`. Release
    builds compile neither the script nor the command, so the grant is
    inert there and one static capability serves both builds.

  Names are checked against Tauri 2.12.5's scripts; re-check them on a
  Tauri upgrade.
- **W-FOCUS**: the window takes keyboard focus at launch (Tauri's
  default `focus: true`, hence omitted per W-WINDOW). Clicking the
  container gives it focus without resetting the timer, since only the
  time element dispatches `Click` (SH-CLICK). Both are on the to-verify
  list, especially for the macOS Accessory policy.
- **W-NO-TEMPLATE**: no `greet` command, no opener plugin, no
  `withGlobalTauri`. The frontend uses no Tauri JS API.
- **W-WINDOWS-SUBSYSTEM**: the binary keeps
  `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`,
  which prevents a console window on Windows in release.
- **W-PROFILE**: the release profile (`strip`, `lto`,
  `codegen-units = 1`) stays in the workspace root, since Cargo ignores
  `[profile.*]` in members.
- **W-DEPENDENCIES**: the dependency set is exactly:
  - `countdown-core`: no dependencies; dev-dependency `proptest`.
  - Frontend: `countdown-core`, `wasm-bindgen`, `web-sys`,
    `gloo-timers` (its `Timeout` cancels on drop, so replacing the one
    pending wakeup is an assignment) and `console_error_panic_hook`.
  - `src-tauri`: `tauri`, and build-dependency `tauri-build`, both with
    no features beyond their defaults.

  Every crate uses edition 2024. Each version requirement is the full
  version adopted, e.g. `"2.12.5"`, so the manifest records the tested
  floor and `cargo update` never moves below it.
