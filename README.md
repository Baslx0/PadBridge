# PadBridge

PadBridge v0.2.0 preview is a local Windows controller monitor built with Rust, egui, and gilrs.

## Working features

- Lists connected, usable gamepads exposed by gilrs and shows live buttons, axes, and device-reported power information.
- Dark navigation with Dashboard, Tester, Profiles, and Settings.
- Per-controller local profiles with create, duplicate, rename, select, delete, restore defaults, deadzone, Y inversion, and trigger threshold settings. Tester transformations are display-only.
- Versioned JSON profiles at `%LOCALAPPDATA%\PadBridge\profiles.json`; invalid files remain untouched and defaults load.

## Limitations

Discovery is limited to gilrs. Paired Bluetooth devices that are not usable gamepads are not listed. A device's gilrs UUID may identify a model rather than a unique physical unit, so identical controllers can share a profile. Battery values are shown only when gilrs reports them. Xbox virtual output, game remapping, native Windows inventory, and vendor features are unavailable. No driver is installed.

Run with `cargo run`; release build with `cargo build --release`. Windows 11 and a Rust toolchain with MSVC build tools are required. See [architecture](docs/ARCHITECTURE.md), [features](docs/FEATURES.md), [QA](docs/QA.md), and [security](docs/SECURITY.md).
