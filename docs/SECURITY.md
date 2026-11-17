# Security

PadBridge uses no telemetry, cloud account, or remote service. Profiles remain under `%LOCALAPPDATA%`. Invalid profile files are preserved while defaults load. The Sony HID input path reads controller reports; it does not send feature, rumble, or lightbar reports.

Virtual output launches a bundled VIIPER server locally and communicates with it over loopback. PadBridge also bundles the signed usbip-win2 installer. Both bundled executables have pinned SHA-256 checks and are verified before use. Installing the Windows kernel driver requires an explicit user action and Windows approval; it is not installed silently. The installer can restart USB hubs.

The distribution ZIP includes the third party license notices and the corresponding VIIPER source archive. See [Game compatibility](GAME_COMPATIBILITY.md#bundled-components) for upstream component versions. No code or binary from the referenced DS4Windows archives was copied into PadBridge.
