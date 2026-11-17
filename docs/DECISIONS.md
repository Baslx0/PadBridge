# Decisions

## 2026-11-17 — Retain gilrs

Preserve confirmed controller detection. Native Windows APIs need separate integration and hardware testing. Do not equate Bluetooth pairing with usable input.

## 2026-11-17 — Local profiles

Use serde and serde_json for versioned, validated local files. Invalid files are preserved. Profile transforms currently feed the tester only.

## 2026-11-17 — Virtual output

ViGEmBus is archived. No maintained, signed, suitably licensed driver and installation path has been verified in this development environment. Xbox emulation remains unavailable; the output interface is explicit and no driver is installed. A future backend needs provenance, signing, licensing, admin and cleanup review plus game testing before enabling it.

Sources: https://github.com/nefarius/ViGEmBus (archived 2023-11-02); https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/buttons (VHF requires a kernel-mode HID source driver). VHF is a framework, not a ready-to-use user-mode virtual Xbox controller backend.


## 2026-11-17 — Sony HID fallback

Windows exposed a present DualSense HID interface and valid input reports while gilrs enumerated zero devices. Add read-only hidapi discovery for known Sony VID/PID values on a worker thread, retaining gilrs for other controllers. Parse only known report IDs and lengths; show no inferred battery values. SDL's PS4/PS5 HID drivers document the report offsets: https://github.com/libsdl-org/SDL/blob/main/src/joystick/hidapi/SDL_hidapi_ps4.c and https://github.com/libsdl-org/SDL/blob/main/src/joystick/hidapi/SDL_hidapi_ps5.c. hidapi API: https://docs.rs/hidapi/2.6.7/hidapi/. The provided DS4Windows archives were inspected as read-only references; no binaries or source were incorporated.

## 2026-11-17 — GTA V Bluetooth output

PadBridge remains input-only. Steam Input is a user-controlled workaround for non-Steam games but separate Epic/Rockstar launchers can prevent its overlay from attaching. Do not claim game compatibility from tester detection. The LuminalVGBus fork says its driver is unsigned and has never been loaded, so it is not integrated. See GAME_COMPATIBILITY.md.

## 2026-11-17 — Optional ViGEmBus output

Add opt-in virtual Xbox 360 output for selected Sony HID input through the established ViGEmBus client API. The driver must be installed separately from the original Nefarius release; PadBridge does not install it. The backend unplugs the virtual target on stop or stale input. This supersedes the earlier input-only decision, but game compatibility remains unverified until tested with a real driver and GTA V.

## 2026-11-17 — Verified driver download

On a ViGEmBus `BusNotFound` error, download the original Nefarius 1.22.0 installer in a background thread to the user's local app data. Pin the release asset URL and SHA-256 (`89220a7865076b342892f98865f3499fb7c4cfd673159e89d352c360fd014c6a`), reject oversized or mismatched downloads, and expose a separate button to run the verified installer. The downloaded file's Authenticode signature was valid with Nefarius as signer during development. PadBridge never silently installs a kernel driver.
