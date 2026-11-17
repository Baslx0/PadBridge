# Game compatibility

PadBridge can create a virtual Xbox 360 controller from a selected Sony HID controller. It starts a bundled [VIIPER](https://github.com/Alia5/VIIPER) server on the local PC and uses the signed [usbip-win2](https://github.com/vadimgrn/usbip-win2) Windows driver. No paid service or remote server is involved. PadBridge includes both release files, but Windows driver installation still requires the user's action. The VIIPER server is GPL-3.0 and its corresponding source and license are included with the app; PadBridge communicates with it through an MIT-licensed Rust client.

## PadBridge Xbox output

1. Pair the PlayStation controller over Bluetooth. On PadBridge's Controllers page, select the Sony HID controller and click **Start Xbox output**.
2. If USB/IP is missing, click **Open verified driver installer** and complete its Windows setup. The installer may restart USB hubs, so save any active work first. Restart Windows if asked, then click **Start Xbox output** again.
3. The dashboard shows **Starting** while VIIPER attaches the device. It shows **Active** and an XInput slot once Windows detects the controller. Keep PadBridge open while playing.
4. Start GTA V after output is active. Click **Stop Xbox output** when finished. Output also stops when Sony input is lost.

The VIIPER server and USB/IP driver were observed creating an Xbox 360 device on this PC, and XInput slot 0 became available in an independent probe. GTA V has not yet been verified with the new PadBridge build. The physical controller may also be visible to games and could cause duplicate input.

## Alternative Steam Input path

Steam Input may translate a PlayStation controller for a non-Steam game. Add GTA V or its launcher to Steam, enable Steam Input for that shortcut, and launch from Steam. Epic/Rockstar's separate processes can prevent Steam's overlay from attaching, so this path is not guaranteed. See [Steam's non-Steam game guidance](https://help.steampowered.com/en/faqs/view/33E8-5EDF-24E6-4CFB).

## Bundled components

- VIIPER 0.9.0 server: [upstream release](https://github.com/Alia5/VIIPER/releases/tag/v0.9.0), GPL-3.0. Its license and corresponding source archive are in `third_party/VIIPER` and are extracted beside the runtime server.
- usbip-win2 0.9.8.1 x64 installer: [upstream release](https://github.com/vadimgrn/usbip-win2/releases/tag/v.0.9.8.1), BSD-2-Clause. Its license is in `third_party/usbip-win2`.

Both bundled executables were checked for valid Authenticode signatures and pinned SHA-256 hashes. They are verified again before extraction. PadBridge does not silently install a kernel driver.
