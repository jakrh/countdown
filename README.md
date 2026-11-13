# Countdown timer

[English](README.md) | [繁體中文](README.zh-TW.md)

A small always-on-top countdown, meant as a reminder to take regular
eye breaks. The interval is configurable and defaults to 25 minutes.

The app is meant to stay on screen so the countdown can be checked at
any time, and apart from clicking the time to reset it, it needs
almost no interaction. To keep it from being switched to or clicked by
mistake, it stays out of the Dock and `Cmd`+`Tab` on macOS, and out of
the taskbar and `Alt`+`Tab` on Windows.

Time running out is signalled just as softly: the digits in the window
turn red and start blinking, and that is all — no notification, no
sound, nothing that pops up and takes focus. The count then continues
past zero into negative time, showing how far over it has run, so
whatever is in progress can be finished first.

## Install

The app is portable — there is no installer. Once the download is
unpacked, the app sits wherever the user keeps programs and is started
by hand. The Linux build is a single file that needs no unpacking at
all.

Get the latest build from [Releases](../../releases).

| Platform | File |
| --- | --- |
| macOS (Apple Silicon) | `…-macos-arm64.zip` |
| Windows (x64) | `…-windows-x64.zip` |
| Linux (x64) | `…-linux-x64.AppImage` |

One step is needed before the first launch:

- **macOS** — the build is ad-hoc signed but not notarized, so macOS
  refuses to open it until the quarantine flag is cleared with
  `xattr -dr com.apple.quarantine <path to countdown.app>`. Dragging
  the app onto the Terminal window fills in the path.
- **Windows** — the executable is unsigned, so SmartScreen blocks the
  first launch. Choose **More info**, then **Run anyway**.
- **Linux** — make the AppImage executable with `chmod +x`.

## Controls

| | |
| --- | --- |
| `p` | Pause / resume (not after time runs out) |
| `f` | Enter time-entry mode (not while paused or after time runs out; the countdown waits) |
| Click the time | Reset to the most recently set time (not while paused) |
| `Cmd`+`Q` | Quit (macOS) |
| `Alt`+`F4` | Quit (Windows) |

The app's keys are matched by their position on the keyboard, so they
work with Caps Lock or an input method on; with a modifier held, they
do nothing.

In time-entry mode:

| | |
| --- | --- |
| `0`–`9` | Overwrite the digit at the cursor and move right |
| `←` `→` | Move the cursor |
| `Backspace` | Clear the digit at the cursor and move left |
| `Enter` | Count down from the new time |
| `f` or `Esc` | Leave time-entry mode, discarding the entry; the countdown continues where it stopped |

Time entry works like setting a digital clock: four digit slots, where
the tens digits of minutes and seconds accept only 0–5. The range is
`00:00`–`59:59`. Switching to another app also leaves time-entry mode,
as `Esc` does.

## Behaviour

- `00:00` lasts one second; then the digits turn red, blink, and keep
  counting into negative time. The count stops at `-59:59`; the
  blinking does not.
- While paused, the digits freeze in cyan.
- The most recently set time is remembered across restarts; the first
  run starts at 25:00.
- There is no close button and no menu bar, so quitting is done from
  the keyboard. On Windows, right-clicking the background also opens
  the window menu with Close.

## Known issues

- On Linux the window opens at the wrong size, does not stay on top of
  other windows, and shows in the taskbar and `Alt`+`Tab`; the overdue
  blinking can also be irregular. Pausing and time entry work
  normally.

## Build from source

Needs Rust, [Trunk](https://trunkrs.dev) and the Tauri CLI:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk tauri-cli

cargo tauri dev      # run
cargo tauri build    # release bundle
```

## License

MIT — see [LICENSE](LICENSE).
