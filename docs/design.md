# Countdown Design

This document explains the few rules Countdown's behavior follows and
why the main decisions went the way they did. [spec.md](spec.md)
defines the exact behavior and wins on any conflict; IDs in parentheses
point into it. Usage is in the [README](../README.md).

## What it is

A small borderless, transparent window that floats above everything
and counts down from a start time. Past zero it keeps counting into
negative numbers and blinks, so the reminder is hard to miss. A click
resets it to the start time, `p` pauses, and `f` opens an editor that
sets a new start time.

It is a glanceable overlay, not an app to switch to, so it stays out of
the taskbar, and on macOS out of the Dock and Cmd+Tab (W-WINDOW,
W-ACTIVATION).

## Behavior principles

Behavior follows from a few general rules rather than special cases.
When a new case comes up, the default answer is that the input does
nothing, not a new rule.

- **Running is the center.** There are four modes: Running, Paused,
  Overdue and Editing. Running accepts pause, edit and reset. Every
  other mode accepts only its own small set of inputs, ignores all
  other input (T-NOOP), and leads only back to Running:
  - Paused: `p`.
  - Overdue: a click, which resets as it does in Running.
  - Editing: digits, arrows, Backspace, Enter, Esc, and losing focus.

  So `f` while paused does nothing rather than meaning "resume and
  edit". Routing every path through Running keeps the transition table
  small and every key predictable.
- **A mode exists only if some input behaves differently in it.**
  Overdue qualifies: `p` and `f` do nothing there. Sitting at `-59:59`
  does not, since every input behaves exactly as in Overdue; it is the
  edge of what `MM:SS` can show, handled by the display and wakeup rules
  (F-SEC, F-WAKE), not a mode.
- **Store what the user did; derive what time did.** Pausing, editing
  and confirming change the stored model. Becoming overdue and reaching
  `-59:59` are derived from the current time (F-MODE): an overdue timer
  is stored as plain `Running`, so it can never disagree with the
  clock.
- **Three looks.** Paused is frozen, a negative displayed number is
  overdue, and everything else is normal (F-LOOK). The editor has its
  own framed appearance.
- **The overdue reminder never weakens.** The blink runs for as long as
  the window is overdue, even hours later at `-59:59`
  (SH-OVERDUE-BLINK). The timer exists to interrupt; it is never traded
  for battery life.
- **Only the start time persists.** It is a setting, not progress: a
  relaunch begins again from the start time (T-INIT), and only Enter in
  the editor changes it (INV-EFFECT-ONLY-CONFIRM). Loading is lenient
  because the app only ever writes the canonical form; anything
  unusable falls back to the default (P-LOAD). Storage failures are
  silent, since the window has no room to show an error.
- **The editor is a four-slot overwrite field**, like setting a digital
  clock. It opens on the displayed time (F-DRAFT-FROM). A digit
  overwrites the slot under the cursor and moves right. A digit that
  would make the time invalid is rejected on the spot, so the draft is
  always valid and Enter always succeeds: there is no invalid state and
  no error message (INV-DRAFT-VALID). Enter applies the draft even when
  nothing changed, which makes the displayed time the new start time
  (SC-CONFIRM-UNCHANGED), and `00:00` is a valid draft; neither gets an
  exception.
- **Editing freezes time.** Esc resumes from exactly where it froze,
  sub-second included (INV-EDIT-ROUNDTRIP). Enter starts the new time
  on a whole second, discarding the frozen remainder
  (INV-CONFIRM-FRESH).

## Architecture

- **The dependency graph keeps the browser and Tauri out of the core**
  (A-CORE-PURE, W-DEPENDENCIES). `countdown-core` holds every timer
  rule. The frontend crate at the workspace root, the shell, depends on
  it, and Trunk builds it into the page; `src-tauri` depends on
  neither, so the native side carries no timer behavior. The graph, not
  discipline, enforces this, and the core's tests run natively.
- **Raw facts go in, effects come out.** The shell passes the core the
  time, the raw stored string and each input; parsing storage and
  deciding when to save are core logic too (P-LOAD,
  INV-EFFECT-ONLY-CONFIRM). Rules are therefore tested in the core,
  down to the overdue blink's onset. Manual testing is left with what
  needs a real window: the `SH-` and `W-` items, drawing the blink
  among them.
- **The shell filters by the event, never by the model**
  (SH-NO-MODE-LOGIC). Key repeat, modifiers, the mouse button and page
  visibility are facts about the event. What each mode ignores is
  already a core rule (T-NOOP); a check in the shell would be a second,
  untested copy that could disagree with it, so the shell does not even
  use `mode`.
- **No UI framework** (A-NO-FRAMEWORK). Every render rebuilds the time
  element's children, because both blinks must restart (see Timing
  model), even when the text is unchanged, as after a rejected digit. A
  framework's diffing exists to reuse unchanged nodes, so it would only
  get in the way, and it would add dependencies to build and audit.
- **A tunable lives in the core when the core's rules or tests use
  it.** `OVERDUE_BLINK_PERIOD_MS` must divide a second so that each new
  number starts lit. The core asserts this at compile time, and
  INV-BLINK-ALIGNED, which can see only the core, tests it, so the
  constant lives there although CSS does the animating. CSS cannot read
  Rust constants, so the shell publishes the values CSS needs as custom
  properties instead of repeating them.

## Timing model

- **Remaining time is derived, never counted down.** The model stores a
  deadline while running and derives the remaining time from the
  current clock (F-REM). A late timer only delays a repaint; it never
  loses time. 0.1.1 subtracted one per tick, so every late tick
  drifted, and a throttled or occluded window fell behind. A window
  that becomes visible again renders at once (SH-VISIBLE), since its
  timers may have been throttled while hidden.
- **Time is read once per event, in whole milliseconds** (SH-CLOCK).
  Floating-point time made about one reset in ten briefly show one
  second more than the start time. A single read keeps `view` and
  `next_wakeup` on the same second; a second boundary between rendering
  and scheduling would leave the old number up for an extra second and
  skip the next one.
- **The display rounds up** (F-SEC). The start time shows for the whole
  first second, and `00:00` lasts one second before `-00:01`.
- **Pause stores milliseconds.** Pausing and resuming neither gains nor
  loses the sub-second part (INV-PAUSE-ROUNDTRIP). 0.1.1 restarted the
  current second on resume, so repeated pausing pushed the time later.
- **The app wakes only when the visible text changes.** `next_wakeup`
  computes the exact instant (F-WAKE). Nothing is scheduled while
  paused, editing or held at `-59:59`; then there are no timers and no
  wasm work, and only CSS animations run. Every wakeup renders and
  reschedules from `now`, so firing early or late needs no special
  handling (SH-WAKE).
- **Every render starts from scratch.** The shell rebuilds the time
  element and keeps no memory of earlier renders, so both blinks restart
  in a known phase:
  - The overdue blink gets a delay computed from `now`, anchored to the
    instant the window became overdue (F-ONSET), so each new number
    appears in the lit half (INV-BLINK-ALIGNED, SH-OVERDUE-BLINK).
  - The editor's cursor blink restarts lit (SH-CURSOR-BLINK), so even a
    rejected key visibly registers.

## Key decisions

- **No `<input>` element.** The app must work while a CJK input method
  such as Cangjie or Zhuyin is on, and a text field lets the input
  method intercept keystrokes, so digits could not be typed. The editor
  is drawn by the app.
- **Keys work whenever the window has focus**, wherever the focus is
  inside it, because they are handled on `window` (SH-KEYMAP). In
  0.1.1, `f` needed the time element focused, and the only way to focus
  it was a click, which reset the timer. For the same reason only the
  time element resets; a click on the background just focuses the
  window (SH-CLICK, W-FOCUS).
- **Keys are matched by `event.code`**, the physical key (SH-KEYMAP), so
  `p` works with Caps Lock or an input method on. The cost is that on
  layouts like Dvorak, `p` and `f` sit at their QWERTY positions. On
  layouts like AZERTY, the digit row types digits without Shift, which
  the modifier rule below requires.
- **Any modifier, Shift included, blocks a key, and key repeat is
  ignored** (SH-KEY-FILTER). Shortcuts like Cmd+P belong to the system,
  and holding `p` must not toggle repeatedly. One rule covers every key
  instead of per-key exceptions. Only keys that are dispatched have
  their default action prevented (SH-KEY-PREVENT).
- **Losing focus while editing cancels the edit** (SH-BLUR,
  T-EDT-CANCEL). Otherwise a forgotten editor would freeze the timer
  indefinitely and silently disable the reminder.
- **The pointer cursor appears only where a click does something**
  (F-CLICKABLE), so a paused or editing timer does not invite a useless
  click.
- **One click from another app resets** (`acceptFirstMouse`, W-WINDOW).
  When the timer goes overdue the user is usually in another app, and a
  single click should be enough.
- **Mouse on the background:** dragging moves the window, a double
  click does nothing because the window is not resizable, and the page
  shows no context menu (SH-DRAG, W-CAPABILITIES, SH-CONTEXTMENU). On
  Windows the native window menu still appears there on purpose: it is
  the only mouse route to Move and Close on a borderless window.
- **One window config for every platform** (W-ONE-CONFIG). Tauri
  replaces arrays wholesale when merging a platform config, so a
  macOS-only file had to repeat the whole window entry. The width is
  the same everywhere and fits `-MM:SS`. `dragDropEnabled` keeps its
  default, under which Tauri intercepts file drops and the app ignores
  them; turning it off on macOS hands drops to WebKit, which may open
  the file over the whole app.
- **The config keeps only settings that do something** (W-WINDOW).
  `macOSPrivateApi` and its `macos-private-api` feature are left out:
  both have been no-ops since Tauri 2.12.1. `theme: "Dark"` stays for
  what the system draws around the page, the Windows 11 border of the
  borderless window. The theme does reach the webview's
  `prefers-color-scheme`; the page is unaffected only because it
  declares no `color-scheme` and never queries it.
- **The old storage key is not migrated** (P-NO-MIGRATION). It was
  named for the remaining time but held the start time. A one-time
  fallback to the default start time is cheaper than carrying migration
  code forever.

## Non-goals

- **System sleep** (A-NON-GOAL-SLEEP). Acting like a pause is the
  preferred outcome where the platform gives it, but nothing enforces
  it. Whether `performance.now()` advances during sleep differs by
  platform, and sleep cannot be told apart from an occluded window
  being throttled, so the app neither detects nor compensates for it.

## Open questions

- **Quitting from inside the app.** There is no quit control; on
  Windows the background's native menu offers Close. Whether to add one
  is a product question.
