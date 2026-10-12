# HID adapter

`crates/hid/` is the only code that reads `/sys/class/hidraw` and opens `/dev/hidraw*`. It runs one
`crates/core/src/session/reducer.rs::Session` on the device thread: it turns sysfs scans, port reads, port errors,
the poll timeout and desired settings from the main thread into `Input`s, and executes the `Effect`s that come back.
It decides nothing about the protocol: every frame it writes comes from the session, so only allow-listed commands
reach the dongle. Design rationale: device part §2 and §6 of the spec, ADR 0005 and ADR 0015.

## Modules

```mermaid
flowchart LR
    handle["handle.rs<br/>DeviceHandle: thread, channels"] --> worker["worker.rs<br/>Worker: turn, feed, execute"]
    wake["wake.rs<br/>DeviceChanges, Mailbox: mpsc + pipe"] --> worker
    waiter["waiter.rs<br/>Waiter, PollWaiter: clock + poll(2)"] --> worker
    rescan["rescan.rs<br/>Rescan: sysfs cadence"] --> worker
    discovery["discovery.rs<br/>scan, first_dongle, command_node"] --> worker
    uevent["uevent.rs<br/>parse"] --> discovery
    open["open_dongle.rs<br/>OpenDongle: iface 3 + iface 5"] --> worker
    port["port.rs<br/>Port, Opener, classify"] --> open
    worker --> session["core::session::Session"]
```

| Concern | Code |
|---|---|
| Public seam to the main thread | `crates/hid/src/handle.rs::DeviceHandle`, `crates/hid/src/handle.rs::spawn`, `crates/hid/src/wake.rs::DeviceChanges` |
| Desired settings in, wake the loop | `crates/hid/src/wake.rs::mailbox`, `crates/hid/src/wake.rs::Mailbox`, `crates/hid/src/wake.rs::take` |
| Clock and `poll(2)` | `crates/hid/src/waiter.rs::Waiter`, `crates/hid/src/waiter.rs::PollWaiter` |
| One loop iteration | `crates/hid/src/worker.rs::Worker`, `crates/hid/src/worker.rs::turn`, `crates/hid/src/worker.rs::feed`, `crates/hid/src/worker.rs::execute` |
| sysfs discovery | `crates/hid/src/discovery.rs::SysfsRoot`, `crates/hid/src/discovery.rs::scan`, `crates/hid/src/discovery.rs::first_dongle`, `crates/hid/src/discovery.rs::command_node`, `crates/hid/src/uevent.rs::parse` |
| Rescan cadence (`Absent` 2 s, `Rebinding` 250 ms, `Off`) | `crates/hid/src/rescan.rs::Rescan`, `crates/core/src/session/effect.rs::Cadence` |
| Raw hidraw: open, 65-byte write, read | `crates/hid/src/port.rs::Port`, `crates/hid/src/port.rs::HidrawOpener` |
| `EACCES`, `EPERM` and `ENOENT` mean denied; anything else means gone | `crates/hid/src/port.rs::classify`, `crates/hid/src/port.rs::OpenError` |
| The open pair; iface 5 error means `Lost`, iface 3 error means `CommandPortLost` | `crates/hid/src/open_dongle.rs::OpenDongle`, `crates/hid/src/worker.rs::port_gone` |

## One turn

```mermaid
flowchart LR
    W["wait()<br/>poll mailbox, iface 5, iface 3<br/>until WakeAt or next rescan"] --> M["mailbox: Change::Desired → Input::Desired"]
    W --> E["iface 5 frames → Unsolicited{Events}<br/>error → Lost"]
    W --> C["iface 3 frames → Reply<br/>error → CommandPortLost"]
    M & E & C --> T["run_timers(): rescan due → discover or reopen<br/>WakeAt due → Woke"]
```

`crates/hid/src/worker.rs::feed` hands each `Input` to the session with the waiter's clock and runs the returned
effects in order: `Write` goes to iface 3 (a failed write queues `CommandPortLost` for the same feed),
`WakeAt` sets the poll deadline, `Rescan` resets `crates/hid/src/rescan.rs::Rescan`, `Publish` goes to the
`async-channel` the bridge reads. When the session leaves the present phases (`Lost`, `REBIND_GIVE_UP`) the worker
closes both ports.

## Discovery and rebinding

```mermaid
sequenceDiagram
    participant R as Rescan
    participant W as Worker
    participant D as discovery
    participant O as Opener
    participant S as Session
    R->>W: due (Absent, every 2 s)
    W->>D: scan(/sys/class/hidraw)
    D-->>W: first_dongle: iface 3 + iface 5 of one HID_PHYS
    W->>O: open iface 5, then iface 3
    alt both open
        W->>S: Found{pid, uniq}
    else EACCES or ENOENT
        W->>S: Denied (PermissionDenied)
    end
    Note over W,S: fwupd unbinds iface 3: read fails
    W->>S: CommandPortLost
    S-->>W: Rescan(Rebinding)
    R->>W: due (every 250 ms)
    W->>D: command_node(same HID_PHYS)
    alt node back and readable
        W->>S: CommandPortBack
    else EACCES while udev applies the ACL
        W->>S: CommandPortDenied (stays Rebinding)
    end
```

| Rule | Code |
|---|---|
| Only supported, non-bootloader models of vendor `1038` are reported | `crates/hid/src/discovery.rs::scan` |
| A dongle needs both iface 3 and iface 5 on one physical path; the lowest `HID_PHYS` wins | `crates/hid/src/discovery.rs::first_dongle` |
| A denied dongle that disappears from sysfs is `Lost` | `crates/hid/src/worker.rs::discover` |
| The command node is looked up again by physical path, since its `hidrawN` may change | `crates/hid/src/worker.rs::reopen_command` |

## Tests

```mermaid
flowchart LR
    Cat["core testkit: scenarios::catalogue<br/>generic over Rig"] --> Sim["Simulation<br/>crates/core/tests/session/scenarios.rs"]
    Cat --> Sock["SocketRig<br/>crates/hid/tests/adapter/scenarios.rs"]
    Sock --> Wk["Worker"]
    Wk -- "socketpair" --> Bench["Bench: FakeHeadset,<br/>virtual clock, FakeSysfs"]
```

| Piece | Code |
|---|---|
| Scenario catalogue shared by both rigs | `crates/core/src/testkit/scenarios/mod.rs::catalogue`, `crates/core/src/testkit/rig.rs::Rig` |
| Worker over socketpairs on a virtual clock | `crates/hid/tests/adapter/rig.rs::SocketRig`, `crates/hid/tests/adapter/rig.rs::VirtualWaiter`, `crates/hid/tests/adapter/bench.rs::Bench` |
| sysfs fixture trees in tempdirs | `crates/hid/tests/adapter/sysfs.rs::FakeSysfs` |
| Discovery, ports, mailbox, waiter, rescan cadence, handle | `crates/hid/tests/adapter/discovery.rs`, `crates/hid/tests/adapter/port.rs`, `crates/hid/tests/adapter/wake.rs`, `crates/hid/tests/adapter/waiter.rs`, `crates/hid/tests/adapter/rescan.rs`, `crates/hid/tests/adapter/handle.rs` |
| Real dongle, `ARCTIS_HW=1` | `crates/hid/tests/hardware/discovery.rs`, `crates/hid/tests/hardware/soak.rs` |
