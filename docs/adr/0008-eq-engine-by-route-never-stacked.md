# One equalizer engine per path, never stacked; the route comes from the default sink

Status: accepted (2026-10-11), revised 2026-10-12

A preset is a curve inside the hardware envelope (10 filters, ±10 dB, Q 0.2..10). Audio reaches the headset over two paths, and each has its own engine and its own route preset. The 2.4 GHz path uses the headset's own `0x33` equalizer with the 2.4 GHz preset, kept on the headset for consoles whenever the dongle session is Ready, whatever PipeWire reports. The path from this computer over Bluetooth uses the in-process PipeWire DSP with the Bluetooth preset. Neither engine processes the other path's audio. The route, which path carries this computer's audio now, is decided from the default sink followed to its terminal node, never from which sinks exist: both links can be connected while only one carries audio.

The curve drawn is the nominal curve: on the system engine it is applied exactly, minus an automatic preamp of `−max(boost)` dB; on the headset engine the firmware's filter design is unknown.

## Considered options

- Route from the existence of a Nova 5 bluez sink: with both links up, it would move this computer's audio to Bluetooth and take over an output the user chose.
- Route from the Nova 5 USB sink: the 2.4 GHz preset would stop applying whenever PipeWire is unreachable or the card profile is off, although `0x33` is plain HID.
- An opt-in "computer equalizer" stacked on top of the headset EQ on every route: two pickers whose effects combine, double processing on 2.4 GHz, and the per-app feature depending on the unverified Manager role.
- A user-facing "Equalizer engine" preference: exposes implementation internals.
- One active preset for all connections: loses the official per-connection selection.

## Consequences

- On a dongle-only desktop the system engine never runs; its row appears only once a Nova 5 bluez node has been seen.
- The official Bluetooth preset applies to phone audio; ours applies only to this computer's Bluetooth audio. That gap is listed in the metainfo (ADR 0013).
- Per-app switching on the 2.4 GHz path writes `0x33`; ADR 0010 bounds that traffic.
- Code: `crates/core/src/companion/routing.rs`, `crates/audio/src/links.rs`.
