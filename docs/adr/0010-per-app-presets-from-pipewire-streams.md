# Per-app presets from PipeWire streams, with ruled-set hysteresis, per-path budgets and defined picks

Status: accepted (2026-10-11), revised 2026-10-12

The app that plays audio, not the focused window, chooses the preset. Flatpak has no focus portal, asking the Shell extension would make an optional surface load-bearing, and playback is what the user hears. A running output stream whose link chain (up to 4 hops, through filters such as EasyEffects) ends at the Nova 5 USB sink or at our EQ sink is a candidate on that path. A stream carries up to three identifiers: the portal app id (from the node, or from its Client for native PipeWire clients), `application.id` and the process binary; a rule matches when its stored identifier equals any of them.

Per path, an app joins the ruled set after 1 s of running and leaves it after 10 s without a running stream, so corking does not count. The app that joined most recently wins; an empty set gives the route preset. A window combo always sets the persistent route preset (and, while a ruled app plays, also becomes the manual pick); a Shell pick or `ActivatePreset` sets the route preset when no ruled app plays, else it is a temporary manual pick that ends when the ruled set or the route changes, or at restart. At most one automatic switch per 30 s on the 2.4 GHz path (each is a `0x33` write) and per 2 s on the Bluetooth path, none when the curve is already equivalent.

## Considered options

- Focused window reported by the Shell extension: makes an optional surface load-bearing and misses background audio such as a voice chat.
- "Started running most recently" by stream state: corking browsers and voice apps would flip the winner and cancel manual picks.
- Matching only the first present identifier: a rule typed as a binary name would never match Flatpak apps.
- A manual pick that turns automatic presets off globally: sticky and surprising.
- A plain 500 ms debounce: still flaps on short sounds and floods 0x33 writes.

## Consequences

- No HID storm on notification sounds or track gaps; switches are silent and explained by the "Now Playing" row and `ActivePresetReason`.
- An orderly quit restores the 2.4 GHz route preset on the headset; a crash or suspend while a ruled app plays can leave the app's curve, and the App Presets page says so.
- Steam AppID matching is left out (needs `/proc` outside the sandbox).
- Code: `crates/core/src/companion/selection.rs`, `crates/audio/src/streams.rs`, `crates/audio/src/links.rs`.
