# QA

Automated gates: `cargo fmt --all -- --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`.

L3/R3 parser and virtual Xbox button mapping are covered by unit tests. The physical stick clicks still need confirmation on a known-good controller.

Observed on this Windows host: Windows PnP exposes a present DualSense Bluetooth HID game-controller interface (VID 054C, PID 0CE6). Gilrs enumerated zero controllers. hidapi opened the interface and read 78-byte `0x01` reports. The release app detected the DualSense through Sony HID. VIIPER output created a virtual Xbox 360 controller visible to XInput, and the user confirmed it works in GTA V through Epic/Rockstar.

Still required manually: full input sweep on known-good DualSense and DualShock 4 hardware; unplug and reconnect; two identical Sony devices; battery unavailable state; saved profile restart; UI responsiveness under sustained input; non-Sony gilrs regression; virtual device cleanup; and more game compatibility checks.

On the connected DualSense, a guided L1/L2/D-pad Up test over both Bluetooth and USB produced no changes in those raw HID fields. Other controls changed in the same capture windows. The parser has a synthetic USB fixture proving it maps left-field values when present. The user subsequently confirmed the fault was in that controller. No software fix can synthesize missing raw input without misleading the user.
