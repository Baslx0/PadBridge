# QA

Automated gates: `cargo fmt --all -- --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`.

Observed on this Windows host: Windows PnP exposes a present DualSense Bluetooth HID game-controller interface (VID 054C, PID 0CE6). Gilrs enumerated zero controllers. hidapi opened the interface and read 78-byte `0x01` reports. The updated release app showed one connected DualSense through Sony HID and rendered the idle tester. This confirms detection, not full button or game compatibility.

Still required manually: press every button and move each stick/trigger in tester; DualSense USB; DualShock 4 USB/Bluetooth; unplug and reconnect; two identical Sony devices; battery unavailable state; saved profile restart; UI responsiveness under sustained input; and non-Sony gilrs regression. No virtual output is present, so game compatibility and virtual cleanup are inapplicable.

On the connected DualSense, a guided L1/L2/D-pad Up test over both Bluetooth and USB produced no changes in those raw HID fields. Other controls changed in the same capture windows. The parser has a synthetic USB fixture proving it maps left-field values when present. The user subsequently confirmed the fault was in that controller. No software fix can synthesize missing raw input without misleading the user.
