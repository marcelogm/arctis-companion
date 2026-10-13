# Five crates around a pure core

Status: accepted (2026-10-11)

The workspace has `core` (pure: protocol, session, eq, companion, battery, view models, ports), `hid` (sysfs and hidraw), `audio` (PipeWire), `ui` (GTK widgets) and `app` (composition root). A crate boundary exists only where it enforces a dependency rule: `core` cannot do I/O because its dependency closure is limited to `serde` and `serde_json` (checked by `xtask check-pure`), adapters cannot see GTK, and widgets cannot reach channels, files or GSettings. Inside a crate, seams are types and modules, not manifests.

## Considered options

- Four crates with the UI inside `app`: UI tests and app platform code would share a crate, and the bridge and the widgets could not proceed in parallel without conflicts.
- Eleven fine-grained crates (testability proposal): each boundary compiler-enforced, but high navigation cost and constant cross-crate type moves for a single-device app.

## Consequences

- One pure module plus its tests explains each feature. Two reducers, `Session` and `Companion`, carry all behaviour; time enters them as a `Millis` argument.
- `core::testkit` (cargo feature `testkit`) provides `FakeHeadset`, `Simulation` and in-memory repositories to every crate's tests.
