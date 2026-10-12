# Own preset library, JSON user presets, Equalizer APO import

Status: accepted (2026-10-11)

SteelSeries presets are proprietary and are not shipped, not even the curves HeadsetControl copies. We ship our own CC0 curves as const data in `crates/core/src/eq/library.rs`, store user presets as one versioned JSON file each under `$XDG_DATA_HOME/arctis-companion/presets/`, and import Equalizer APO / AutoEQ parametric text, the de facto interchange format, clamped to the hardware envelope.

## Considered options

- Shipping HeadsetControl's Flat/Bass/Focus/Smiley values: copied from the proprietary app.
- TOML user presets: no advantage over JSON, which `serde_json` already gives `core`.

## Consequences

- Users can bring measured corrections; export is a file copy.
- Built-ins are read-only; editing starts a duplicate, so no reset logic exists.
