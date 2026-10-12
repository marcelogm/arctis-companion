# Long-lived ports, silent Rebinding, sysfs rescan as the only discovery

Status: accepted (2026-10-11), revised 2026-10-12

iface 3 is unbound and rebound about every 20 s. The cause is fwupd's SteelSeries "fizz" plugin, which owns this receiver, detaches usbhid from interface 3 to poll the headset, and reattaches it within milliseconds: `udevadm monitor` shows the unbind and bind of `1-5.2.1:1.3` 5 ms apart with no USB reset, the period is exactly 20.000 s, and `fwupdmgr get-devices` lists the receiver and headset under the `steelseries` plugin. SP0 confirmed it without root: fwupd 2.1.8 polls this receiver every 20 000 ms through interface 3, its threads wake at each rebind, and the first reply after a rebind carries the tail of fwupd's `B0` (`docs/research/hardware-findings.md` §4). Every host with fwupd therefore sees the flap, so silent Rebinding is needed everywhere.

We keep iface 3 and iface 5 open while the dongle is present, read both in one `poll(2)` loop, and on `EIO`/`ENODEV` from iface 3 enter `Rebinding`: the snapshot is kept, no query is sent, desired changes queue, and the interrupted phase resumes after the node returns and `REBIND_SETTLE` (default 100 ms, since nothing re-enumerates) passes. Frames read during a settle are dropped, so stale tails left by fwupd never decode. A reopen that fails with EACCES (the node exists before udev applies the `uaccess` ACL) is retried like a missing node; Rebinding gives up into Absent after `REBIND_GIVE_UP` (30 s). Discovery is a sysfs rescan (`/sys/class/hidraw/*/device/uevent`): every 2 s while Absent or PermissionDenied, every 250 ms while Rebinding, never otherwise.

## Considered options

- Open iface 3 per transaction: hides the flap without state, but drops unsolicited frames on iface 3, makes the kernel start and stop interrupt polling on every command, and still needs a recovery path that re-queried the device.
- Full resync after every reopen: turns the flap into a 20 s poll, the pattern linked to audio glitches.
- The 3 s post-enumeration settle after every rebind: no re-enumeration happens, and it would make the command interface unusable about 15 % of the time.
- inotify on `/dev` or a libudev monitor: inotify under bwrap is unverified and netlink is blocked by `--unshare-net`; sysfs reads are exposed in the sandbox and touch no device.

## Consequences

- The UI shows "Reconnecting…" only after 5 s of Rebinding; a transient EACCES never shows "Permission Required".
- An in-flight write is redone after reopen and verified.
- Asking users to disable fwupd is never needed.
- Code: `crates/core/src/session/`, `crates/hid/src/discovery.rs`, `crates/hid/src/worker.rs`.
