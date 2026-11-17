# QA

Automated gates: `cargo fmt --all -- --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`.

Manual Windows hardware tests still required: no pad; one and multiple pads; USB and Bluetooth reconnect; DualSense and GameSir modes; battery unavailable and charging reports; restart with saved profiles; two identical controllers; UI responsiveness under sustained input. Corrupt profile recovery is unit-covered only through validation and should be checked in UI. Virtual output cleanup is inapplicable while output is unsupported.
