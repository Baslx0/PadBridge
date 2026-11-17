# Architecture

## v0.1

- Rust 2021 application.
- `eframe/egui` native desktop UI.
- `gilrs` gamepad enumeration, normalized buttons/axes and power information.
- Device input is polled by the UI every 50 ms. No file storage or external services.

## Boundary and limitations

Enumerating all Bluetooth devices is different from enumerating input-compatible gamepads. v0.1 only lists gamepads recognized by gilrs. Power information is backend-dependent and may be absent even when Windows shows a battery indicator. Xbox emulation requires a separately evaluated driver and is not implemented.

## Proposed modules (future)

- `discovery`: Windows HID, XInput, Windows.Gaming.Input, BLE metadata.
- `capabilities`: per-device reports for battery, buttons, axes, rumble.
- `profiles`: local per-device settings.
- `output`: optional and explicitly enabled virtual controller.
- `ui`: controller dashboard.

No service or driver is installed by the app.
