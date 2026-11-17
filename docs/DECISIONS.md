# Decision log

## 2026-11-17 — Project identity
Name: PadBridge. The repository was renamed from `Controller.PC` to `PadBridge` on 2026-11-17.

## 2026-11-17 — Native implementation
Start with Rust + egui + gilrs. Benefits: native executable and a small, inspectable application. Trade-off: gilrs may not discover every generic Bluetooth HID device.

## 2026-11-17 — Security-first scope
Do not package unsigned third-party controller binaries or quietly install drivers. Xbox emulation deferred until a maintained driver is reviewed.

## 2026-11-17 — Battery semantics
Only display values from an actual device/backend report; never infer a percent from connection type or elapsed time.
