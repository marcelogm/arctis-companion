# Events first; status poll only while the headset is off, safety poll every 5 minutes

Status: accepted (2026-10-11), revised 2026-10-12

Battery, charging, link and on-headset changes come from unsolicited events, read on both iface 3 and iface 5 because where they arrive is unverified. `B0` is queried on probe, on a link event, every 10 s while the headset is off (`HEADSET_OFF_POLL`: no 2.4 GHz audio flows then, so it cannot glitch), and in Ready only as a safety net 5 minutes after the last battery event, never during a write. HeadsetControl #452 links frequent status polling to audio glitches on the Nova 5. If the hardware probe finds no events at all, the Ready cadence becomes what its glitch test allows, with a 60 s floor.

## Considered options

- Poll `B0` every 2 s (Arctis Sound Manager): the pattern reported to glitch audio.
- Events only, no poll: a missed event would leave battery and link stale forever.
- One 5-minute poll in every phase: without events, a headset turned on would wait up to 5 minutes, possibly running factory settings.

## Consequences

- Battery history has no dedicated sampling traffic.
- Without events, the headset turning on is detected within `HEADSET_OFF_POLL`.
- Code: `crates/core/src/session/timing.rs`.
