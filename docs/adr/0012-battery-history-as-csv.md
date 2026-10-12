# Battery history as a change-only CSV

Status: accepted (2026-10-11), revised 2026-10-12

Samples come only from events and the existing status reads, and a row is appended only when percent, charging or link changes: about 100 rows a day. An append-only CSV under `$XDG_DATA_HOME/arctis-companion/battery.csv`, trimmed to 8 days at start-up, is enough for the Day and Week views, the only views offered.

## Considered options

- SQLite: indexes and WAL for a dataset of a few hundred rows, plus a bundled C library.
- 90-day retention: 83 days nobody can see; a Month view can bring it back later.

## Consequences

- Queries are a full read of a small file; `core::battery::series` does the work in pure code.
- Code: `crates/core/src/battery/`, `crates/app/src/platform/battery_store.rs`.
