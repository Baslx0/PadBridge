# Security

PadBridge uses no telemetry, cloud account, driver installation, background service, or elevation. Profile data remains under `%LOCALAPPDATA%`. New dependencies are serde and serde_json, used only to serialize local profiles. An invalid profile file is preserved and defaults are loaded. Atomic replacement uses a temporary file in the same directory; external file locks may cause save to fail, which the UI reports.

Dependency and driver provenance should be reviewed before distribution. No virtual controller binary is bundled.
