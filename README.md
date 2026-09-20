# MirrorPy

Android screen mirroring and control for the desktop, built with Tauri and powered by scrcpy.

This is the Rust rewrite. The original Python implementation lives in [Rfannn/mirrorpy](https://github.com/Rfannn/mirrorpy) under `legacy/` and is no longer the primary target.

## What it does

- Finds Android devices over USB and ADB mDNS, no manual IP entry
- Pairs over wireless debugging and connects with one click
- Mirrors through scrcpy with four quality presets
- Screenshot, clipboard in both directions, file push, device info
- Remembers your device, quality and appearance between sessions

## Requirements

- Windows 10 or 11
- Rust 1.77 or newer
- Node.js is not required: the frontend is plain HTML, CSS and JavaScript, and Tauri serves it directly

adb and scrcpy ship with the app. You do not need the Android SDK.

## Build

```bash
cargo tauri build
```

The installer lands in `src-tauri/target/release/bundle/nsis/` and the bare executable in `src-tauri/target/release/`.

For a hot-reloading development window:

```bash
cargo tauri dev
```

### Binaries

`src-tauri/tauri.conf.json` lists the scrcpy and adb files that get bundled as resources. During development they are read from the neighbouring `mirrorpy` checkout, so keep the two projects side by side:

```
Documents/
├── mirrorpy/          # Python version, holds adb.exe, scrcpy.exe
└── mirror-rust/       # this project
```

To build without that checkout, copy `adb.exe`, `scrcpy.exe`, `scrcpy-server` and the scrcpy DLLs next to `tauri.conf.json` and update the resource paths.

## Architecture

```
src/                       Frontend, served directly by Tauri
├── index.html             Shell: title bar, sidebar, six sections
├── styles.css             Glass theme, real backdrop blur
└── app.js                 State, command calls, rendering

src-tauri/src/             Backend
├── lib.rs                 Command registration
├── tools.rs               Binary and data path resolution, process helpers
├── devices.rs             adb parsing, mDNS discovery, subnet scan
├── scrcpy.rs              Quality presets, mirror launch
├── actions.rs             Screenshot, clipboard, file push, QR codes
└── settings.rs            settings.json load and save
```

Settings live in the app config directory, not the install folder:

- Windows: `%APPDATA%\com.rfannn.mirrorpy\settings.json`
- Screenshots: `%APPDATA%\com.rfannn.mirrorpy\screenshots\`

## Quality presets

| Preset | Resolution | Frame rate | Bitrate |
|--------|-----------|------------|---------|
| Low | 720 | 15 fps | 2 Mbps |
| Medium | 720 | 30 fps | 4 Mbps |
| High | 1080 | 60 fps | 8 Mbps |
| Ultra | device native | device native | 50 Mbps |

## Keyboard

| Key | Action |
|-----|--------|
| Esc | Close |
| F11 | Toggle fullscreen |
| Ctrl+1 to Ctrl+6 | Switch section |

## Not implemented yet

Record, volume control and app management are shown disabled rather than pretending to work. Everything else on the Quick Actions page is functional.

## License

Apache 2.0.
