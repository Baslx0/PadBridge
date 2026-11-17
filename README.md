# PadBridge

PadBridge is a local-first, open-source controller inspector for Windows, written in **Rust** with **egui** and **gilrs**.

## Preview v0.1

- Detect controllers recognized by Windows through the gilrs backend over supported USB/Bluetooth modes.
- Live buttons, analog sticks and trigger visualizer.
- Show charging state and battery percentage **only where the driver reports it**. Otherwise show Unavailable.
- Adjustable **visualization-only** deadzone.
- No cloud account, background service, bundled third-party driver or admin privileges.

**Limitations:** This is not yet a DS4Windows replacement: no Xbox emulation, actual remapping, Bluetooth pairing or adaptive triggers. Not all generic Bluetooth HID devices are recognized. Specific GameSir models and modes require testing.

## Run on Windows

Install the official Rust toolchain from https://rustup.rs and Microsoft C++ Build Tools.

```powershell
git clone https://github.com/Baslx0/PadBridge.git
cd PadBridge
git switch develop
cargo run
```

For a release build:

```powershell
cargo build --release
```

The executable is at `target\release\padbridge.exe`.

## Security

Source code is inspectable. Do not run unofficial prebuilt ZIPs. No elevated permissions or driver installs are performed by PadBridge. Review all dependencies before compiling.

## Roadmap

1. Validate DualSense and GameSir controller modes on real Windows hardware.
2. Add Windows HID and Bluetooth inventory to detect paired devices that gilrs does not expose as gamepads.
3. Add platform-specific battery probes with accurate unavailability states.
4. Local profiles and remapping.
5. Evaluate optional Xbox virtual controller integration with a maintained and verified driver.

See [architecture](docs/ARCHITECTURE.md), [decisions](docs/DECISIONS.md), and [QA plan](docs/QA.md).

**Status:** Prototype, not yet Windows build-tested or hardware-tested.
