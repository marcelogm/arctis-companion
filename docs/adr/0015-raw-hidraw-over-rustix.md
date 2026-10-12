# Raw hidraw over rustix, no hidapi

Status: accepted (2026-10-11)

We already discover nodes by reading sysfs ourselves, and the protocol needs only open, a 65-byte write and a 64-byte read on `/dev/hidrawN`. Raw `rustix` calls do that without a C library, libudev, or `/run/udev` in the sandbox, and a socketpair stands in for the node in tests.

## Considered options

- `hidapi` with `linux-native-basic-udev`: unverified in Flatpak, and its enumeration duplicates our sysfs discovery.
- `async-hid`: brings an async runtime.

## Consequences

- `unsafe` is allowed only in `crates/hid` and `crates/audio`.
- Code: `crates/hid/src/port.rs`.
