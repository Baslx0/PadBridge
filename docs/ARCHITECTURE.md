# Architecture

`src/main.rs` owns the egui app, navigation, selected-device state, and presentation of gilrs and Sony devices. `src/ui.rs` owns reusable layout and tester widgets. `src/core/mapping.rs` contains pure display transforms. `src/core/profiles.rs` owns versioned local JSON storage. `src/output.rs` sends selected Sony HID input to an optional ViGEmBus virtual Xbox 360 target. The target is unplugged on stop, disconnection, input timeout, or app shutdown.

`src/sony.rs` uses hidapi on a dedicated worker thread. It scans only known Sony game-controller HID interfaces, opens them by path, reads input reports with a 20 ms timeout, and sends bounded-rate snapshots to the UI. It never sends output reports. Report parsing supports DualSense simple and extended reports and DualShock 4 USB/Bluetooth reports. A device appears only after a valid input report. It is removed after a device-removal event or two seconds without valid reports. Scanning runs every two seconds. The worker is stopped and joined at shutdown.

Gilrs remains unchanged for other controllers. When gilrs reports a Sony device with the same display name, PadBridge hides a matching Sony HID entry to reduce duplicates. This is a name-based heuristic and needs multi-device hardware testing. Gilrs polling still occurs on the UI thread every 50 ms; Sony HID work does not.

Sony profile keys use product ID plus serial number where exposed, or HID path otherwise. A HID path can change across reconnections. Gilrs keys remain model-based and may be shared by identical controllers. Battery is unavailable on the Sony HID fallback until a reliable report parser is verified.
