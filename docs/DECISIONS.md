# Decisions

## 2026-11-17 — Retain gilrs

Preserve confirmed controller detection. Native Windows APIs need separate integration and hardware testing. Do not equate Bluetooth pairing with usable input.

## 2026-11-17 — Local profiles

Use serde and serde_json for versioned, validated local files. Invalid files are preserved. Profile transforms currently feed the tester only.

## 2026-11-17 — Virtual output

ViGEmBus is archived. No maintained, signed, suitably licensed driver and installation path has been verified in this development environment. Xbox emulation remains unavailable; the output interface is explicit and no driver is installed. A future backend needs provenance, signing, licensing, admin and cleanup review plus game testing before enabling it.

Sources: https://github.com/nefarius/ViGEmBus (archived 2023-11-02); https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/buttons (VHF requires a kernel-mode HID source driver). VHF is a framework, not a ready-to-use user-mode virtual Xbox controller backend.

