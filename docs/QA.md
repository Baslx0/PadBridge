# QA gates

## Not yet verified
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check` on Windows
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] DualSense wired and wireless smoke tests
- [ ] GameSir Bluetooth mode smoke tests
- [ ] Button and axis tester accuracy
- [ ] Battery reporting: accurate or Unavailable
- [ ] No network requests / unapproved install behaviors

## Manual functional tests

1. Run on Windows with no controller: see empty-state guidance.
2. Connect DualSense through USB: verify name and button/axis motion.
3. Connect DualSense through Bluetooth: verify behavior, note battery accuracy.
4. Connect GameSir through Bluetooth: record exact model, current pairing mode, and whether the device appears.
5. Disconnect each controller while the app is running; verify the list updates.
6. Confirm no admin prompt or additional driver installation.

## Release policy

Do not tag a stable release until Windows build, functional tests and security review pass.
