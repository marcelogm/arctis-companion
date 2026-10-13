# Desired-state reconciliation with adoption per dongle identity, events as intent, echoes filtered

Status: accepted (2026-10-11), revised 2026-10-12

Headset settings exist twice: desired (persisted in GSettings, owned by `Companion`) and observed (read by `Session`). `Session` starts with no desired settings and never writes until `Change::Desired { settings, policy }` arrives; policy and settings travel in one message. When a dongle whose identity (PID plus `HID_UNIQ`) differs from `adopted-identity` reaches Ready, the policy is `Adopt`: desired adopts observed, and the observed curve maps to an equivalent library preset (`core::eq::curve::equivalent`, tolerant of the device's quantization) or becomes the custom preset "Headset Curve". Until then, headset settings are not editable and schema defaults are never shown. After adoption the policy is `Enforce`: on every connect only drifted fields are written and verified. An on-headset change event updates desired; an event that only echoes our own recent write is consumed by `Session` and never becomes intent. Persistence of settings across a headset power cycle is unknown, and this rule is correct in both cases.

## Considered options

- GSettings always wins, no adoption: destroys the user's existing headset configuration on first launch.
- A single global "adopted" flag: a replaced dongle would silently receive the old headset's settings.
- Firmware strings in the identity: a firmware update would re-adopt, possibly factory values.
- Tracking fields edited before adoption: more state than hiding the controls until the headset is read.
- Three-way merge with a last-synced copy: would adopt factory values if the headset does not persist settings.
- Headset always wins: changes made while the headset was off would be lost.

## Consequences

- Changes made from the phone app while this app is not running are overwritten on reconnect. Revisit if the hardware probe proves persistence.
- A real button press that lands on a value written within `ECHO_WINDOW` is treated as an echo; the readback still shows it.
- Code: `crates/core/src/companion/adoption.rs`, `crates/core/src/session/reconcile.rs`, `crates/core/src/session/echo.rs`.
