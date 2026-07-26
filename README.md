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
| Windows (x64) | `…-windows-x64.7z` |
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
| `p` | Pause / resume |
| `f` | Enter time-entry mode (pauses the countdown) |
| `Enter` | Confirm the new time and restart |
| `Esc` | Leave time-entry mode, changing nothing |
| Click the time | Reset to the most recently set time |
| `Cmd`+`Q` | Quit (macOS) |
| `Alt`+`F4` | Quit (Windows) |

In time-entry mode, typing digits inserts the colon automatically.
The range is `00:00`–`59:59`.

## Behaviour

- At `00:00` the display blinks and keeps counting into negative time,
  stopping at `-59:59`.
- The most recently set time is remembered across restarts; the first
  run starts at 25:00.
- There is no close button and no menu bar, so quitting is done from
  the keyboard.

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
