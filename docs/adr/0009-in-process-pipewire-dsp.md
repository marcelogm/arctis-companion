# In-process PipeWire DSP instead of filter-chain

Status: accepted (2026-10-11), revised 2026-10-12

`module-filter-chain` is missing from the GNOME Flatpak runtimes and cannot be loaded from the sandbox. The system engine is therefore our own client nodes, EasyEffects style: an `Audio/Sink` "Arctis Nova 5 (Equalizer)" whose audio is processed by `core::eq::Equalizer` in the realtime callback and played to the Nova 5 bluez node. Coefficients are designed off the RT thread and handed over through the `triple_buffer` crate; `Equalizer::retarget` crossfades over 10 ms without allocating.

The EQ sink is inserted only when the default sink already is the Nova 5 bluez node, and it takes that default through the `default` metadata, which needs `media.category = Manager`. On removal the bluez node is restored only if our sink is still the default; a default the user chose is never changed. The sink's volume and mute are mirrored to the bluez node, so GNOME's slider and media keys keep controlling the headset.

## Considered options

- Host drop-in config in `~/.config/pipewire`: widens the sandbox and needs a PipeWire restart.
- Bundling filter-chain into the Flatpak: ABI coupling with the runtime's libpipewire, built from source in the manifest.
- A hand-written lock-free mailbox: concurrency code without a vetted implementation.

## Consequences

- The nodes die with the process, so a crash cannot leave a broken audio graph.
- If WirePlumber denies the Manager role, a warning row asks the user to select the EQ sink in Sound settings.
- `crates/audio` forbids `unsafe` unless SP0 picks `pw_filter`, which `pipewire` 0.10 does not wrap; then only `raw_filter.rs` may use it, behind safe functions.
- Flatpak needs `--filesystem=xdg-run/pipewire-0` and the `llvm22` SDK extension, since the `pipewire` crate runs bindgen.
- Code: `crates/audio/src/eq_sink.rs`, `crates/audio/src/outputs.rs`, `crates/audio/src/volume.rs`, `crates/core/src/eq/equalizer.rs`.
