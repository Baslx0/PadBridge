# PadBridge

PadBridge v0.2.0 preview is a local Windows controller monitor built with Rust, egui, gilrs, and a read-only Sony HID fallback.

## Working features

- Lists usable gamepads exposed by gilrs. A Sony HID fallback reads known DualSense and DualShock 4 USB/Bluetooth input reports when Windows Gaming Input does not expose them through gilrs.
- Live button, stick, and trigger tester. Battery is shown only when the backend reports it; Sony HID fallback currently shows Unavailable.
- Dark navigation with Controllers, Input tester, Profiles, and About & settings.
- Local profiles with create, duplicate, rename, select, delete, restore defaults, deadzone, Y inversion, and trigger threshold settings. These change the tester display only.
- Versioned JSON profiles at `%LOCALAPPDATA%\PadBridge\profiles.json`; invalid files remain untouched while defaults load.

## Run

Use `cargo run` or `cargo build --release` on Windows 11 with the Rust MSVC toolchain and C++ build tools. The release executable is `target\release\padbridge.exe`.

## Limitations

Sony HID support covers official Sony vendor ID `054C` with DualSense (`0CE6`), DualSense Edge (`0DF2`), and DualShock 4 (`05C4`, `09CC`) report layouts. The connected DualSense Bluetooth controller was detected locally; PS4 and USB paths have parser tests but still need hardware verification. Other vendors remain limited to gilrs. Virtual Xbox output requires ViGEmBus and has not yet been tested with the driver or a game on this PC. When missing, PadBridge downloads a hash-verified official installer for the user to run. No game remapping, automatic driver installation, Bluetooth pairing, or reliable per-unit identity for gilrs-only identical controllers. No cloud account or telemetry.

See [architecture](docs/ARCHITECTURE.md), [features](docs/FEATURES.md), [QA](docs/QA.md), and [security](docs/SECURITY.md).

Game use: PadBridge can create virtual Xbox 360 output from Sony HID input when ViGEmBus is installed separately. For GTA V through Epic/Rockstar over Bluetooth, see [game compatibility](docs/GAME_COMPATIBILITY.md) for setup and limits.
