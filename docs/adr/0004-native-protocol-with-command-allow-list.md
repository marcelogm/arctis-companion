# Native Rust protocol; the Command type is the allow-list of frames

Status: accepted (2026-10-11), revised 2026-10-12

We implement the Nova 5 HID protocol in `crates/core/src/protocol/` instead of depending on HeadsetControl. `Command` has a variant only for frames we trust: the queries `B0 10 20 A0 36` and the absolute setters `37 39 27 33 AE A3 35`. No variant can produce HeadsetControl's `09` then `35 01` "save" (which most likely forces wireless speed mode), `A5`, `A7`, `0x22`, `0x34`, or any frame for a bootloader PID. A golden test asserts it.

`0x22` is a read-only query, so SP0 sends it once from its throwaway script to learn whether the headset keeps a Bluetooth curve. `Command` gains `0x22` and `0x34` only if SP0 proves both work (ADR 0013).

## Considered options

- HeadsetControl CLI or library: no events, no extended range, a suspect save sequence, C++ in the Flatpak, and no seam for fakes.

## Consequences

- A forbidden frame cannot be sent by mistake; enabling one later is a visible change to the type and its tests.
- Every setter writes an absolute value, so retrying after a transport error is always safe.
- Hardware facts that are still unverified are confirmed by SP0 before code relies on them.
- Code: `crates/core/src/protocol/command.rs`.
