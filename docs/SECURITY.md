# Security

PadBridge uses no telemetry, cloud account, driver installation, background service, or elevation. Profile data remains under `%LOCALAPPDATA%`. Invalid profile files are preserved and defaults load. The Sony HID fallback is read-only: it sends no feature, output, rumble, or lightbar reports.

Dependencies: serde and serde_json serialize local profiles; hidapi 2.6.7 provides safe Rust access to the native HID API and builds its C wrapper from source. PadBridge contains no `unsafe` blocks or bundled controller executables. Review native dependencies and driver provenance before distribution. No code or binary from the referenced DS4Windows archives was executed or copied into PadBridge.
