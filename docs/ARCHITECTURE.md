# Architecture

`src/main.rs` owns the egui application and gilrs adapter. `src/core/mapping.rs` contains pure display transforms. `src/core/profiles.rs` owns versioned local JSON storage and validation. `src/core/mod.rs` defines the output boundary; its only current implementation reports unavailable and releases no resources.

The UI drains gilrs events every 50 ms and reads the current state. This preserves v0.1 behavior, but hardware polling is still on the UI thread. Native Windows discovery, stable per-unit identity, background input collection, and deduplication remain future work. Model UUID plus device name forms the profile key, which can be shared by identical units.

## UI

`src/ui.rs` owns color, spacing, surfaces, empty states, and tester widgets. `src/main.rs` owns navigation, selected-device state, device listing, and profile actions. The UI is optimized for a 940×700 desktop window and supports resizing down to 650×500 with vertical scrolling.
