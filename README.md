# PadBridge

PadBridge is an open source Windows app for using a PlayStation controller in games that expect an Xbox 360 controller. It shows live input and sends it to a virtual Xbox controller through [VIIPER](https://github.com/Alia5/VIIPER) and [usbip-win2](https://github.com/vadimgrn/usbip-win2). Everything runs on your PC; there is no account, subscription, or remote server.

## Get the Windows app

1. Open the [PadBridge Releases](https://github.com/Baslx0/PadBridge/releases) page and download the latest `PadBridge-v*-windows-x64.zip`.
2. Extract the ZIP, then run `PadBridge.exe` from the extracted folder.
3. Connect a DualSense, DualSense Edge, or DualShock 4 by Bluetooth or USB. Select it on the **Controllers** page.
4. Open **Input tester** to check your buttons, sticks, triggers, and the live controller drawing.
5. Select **Start Xbox output** before opening your game. On first use, PadBridge may offer the bundled, verified USB/IP driver installer. Complete Windows setup, then start output again. Wait for **Active** and an XInput slot. Keep PadBridge open while playing.

For detailed setup and troubleshooting, read [Getting started](docs/GETTING_STARTED.md) and [Game compatibility](docs/GAME_COMPATIBILITY.md). The ZIP includes the third party licenses and VIIPER source archive.

## Build from source

On Windows 10/11 x64, install [Rust with the MSVC toolchain](https://rust-lang.org/tools/install/) and the Visual Studio C++ build tools. Clone this repository using its **Code** button, open PowerShell in the clone, and run:

```powershell
cargo build --release --locked
```

The executable is `target\release\padbridge.exe`. To make the same ZIP used by releases, run `powershell -NoProfile -File .\scripts\package.ps1 -SkipBuild`; it writes a ZIP and SHA-256 file under `dist\`.

## Features and limits

- Detects supported Sony HID controllers even when Windows Gaming Input does not expose them through gilrs. Other controllers use gilrs.
- Live controller view and tester, local profiles, deadzones, Y-axis inversion, and trigger thresholds.
- Virtual Xbox 360 output for selected Sony HID controllers. A signed Windows USB/IP kernel driver must be installed once with Windows approval. PadBridge bundles its installer and checks its SHA-256 hash.
- The user has confirmed GTA V through Epic/Rockstar working with the current virtual output setup. Other games and hardware configurations need testing.
- No automatic Bluetooth pairing, game-specific remapping, or reliable per-unit identity for identical gilrs-only controllers. Profile adjustments currently affect the tester display, not Xbox output.

See [Architecture](docs/ARCHITECTURE.md), [Features](docs/FEATURES.md), [QA](docs/QA.md), [Security](docs/SECURITY.md), and [Roadmap](docs/ROADMAP.md).
