# Capability-driven rows; the audio-mode switch and the phone Bluetooth preset are known gaps

Status: accepted (2026-10-11), revised 2026-10-12

The official app has a 2.4 GHz / Bluetooth audio-mode switch, but no USB command for it is known and the dongle has no view of the Bluetooth link. The HIG says not to show a control that cannot work, so the header shows only how this computer's audio reaches the headset, read-only. Each model declares `Capabilities` in `crates/core/src/protocol/model.rs`, and `core::view` omits rows a model lacks, so a control appears the day its command is verified.

The official "Dual EQ" also gives the headset a Bluetooth preset that shapes phone audio. Our Bluetooth preset shapes only this computer's Bluetooth audio, through the system engine, and its row is named "Bluetooth from This Computer". SP0 sends the read-only `0x22`; only if it returns a band layout does SP0 try `0x34` with phone audio. A working pair becomes a capability with its own row in a later sub-project.

## Considered options

- Show the switch disabled: a control that never works.
- Guess a command from other models: an unverified write to the headset (ADR 0004).
- Call our Bluetooth preset parity: users would expect it to change their phone audio.

## Consequences

- Two known parity gaps, listed in the metainfo: the audio-mode switch and the Bluetooth preset for the phone.
- Code: `crates/core/src/protocol/model.rs`, `crates/core/src/view/headset_page.rs`.
