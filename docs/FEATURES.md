# Features

Implemented: gilrs gamepad listing and live tester; Sony DualSense Bluetooth HID detection and idle-input display verified on Windows; parser support for known DualSense/DualShock 4 USB and Bluetooth reports, including L3/R3 stick clicks; local profiles; display-only deadzone, stick inversion, and trigger thresholds.

Observed on this host: DualSense Bluetooth and USB discovery; working face and right-side button/trigger reports. The user confirmed the tested controller has a fault affecting its left controls. Still unverified on healthy hardware: left controls, DualShock 4 USB/Bluetooth, multiple Sony devices, disconnect/reconnect, and profile identity across connection modes.

Optional: Xbox 360 emulation from selected Sony HID input through the bundled VIIPER server and a separately installed usbip-win2 driver. Unavailable: game input remapping, paired Bluetooth inventory, vendor extras, and Sony HID battery reporting. The UI says Unavailable when no trustworthy battery data exists.
