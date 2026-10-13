# Arctis Companion

[![CI](https://github.com/marcelogm/arctis-companion/actions/workflows/ci.yml/badge.svg)](https://github.com/marcelogm/arctis-companion/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/marcelogm/arctis-companion?include_prereleases&sort=semver)](https://github.com/marcelogm/arctis-companion/releases)
[![License](https://img.shields.io/github/license/marcelogm/arctis-companion)](LICENSE)
![Rust](https://img.shields.io/badge/rust-2024-orange?logo=rust)
![GNOME](https://img.shields.io/badge/GNOME-50-4a86cf?logo=gnome)
![Flatpak](https://img.shields.io/badge/flatpak-io.github.marcelogm.ArctisCompanion-4a90d9?logo=flatpak)

GNOME app for the SteelSeries Arctis Nova 5 headsets on Linux. It changes the headset settings and equalizer,
switches presets by the app that is playing, and keeps a battery history.

It covers what the official mobile app does: sidetone, microphone volume, mute light, volume limiter, wireless
mode, automatic power off and equalizer presets. On top of that it adds a parametric equalizer editor with
Equalizer APO import, a computer-side equalizer through PipeWire, per-app presets, battery notifications and a
GNOME Shell Quick Settings extension.

The dongle is reached directly through hidraw. Only commands from a fixed allow-list are ever sent.

## Supported headsets

| Model | USB id | Tested |
|---|---|---|
| Arctis Nova 5 | 1038:2232 | yes |
| Arctis Nova 5X | 1038:2253 | yes |
| Arctis Nova 5X | 1038:2255 | no |
| Arctis Nova 5X | 1038:2264 | no |

Switching between 2.4 GHz and Bluetooth audio, and the Bluetooth equalizer preset, are not supported yet.

## Install

Download the assets from the [latest release](https://github.com/marcelogm/arctis-companion/releases).

```sh
flatpak install --user arctis-companion.flatpak

# dongle access, once per computer
sudo cp 70-arctis-companion.rules /etc/udev/rules.d/
sudo udevadm control --reload && sudo udevadm trigger

# optional: Quick Settings extension
gnome-extensions install arctis-companion@marcelogm.github.io.shell-extension.zip
gnome-extensions enable arctis-companion@marcelogm.github.io
```

Without the udev rule the app opens a page explaining how to install it.

## Build

Needs Rust 1.98, Meson 1.5, GTK 4, libadwaita 1.9, PipeWire, blueprint-compiler, gettext and clang.

```sh
flatpak-builder --user --install --force-clean build-dir build-aux/io.github.marcelogm.ArctisCompanion.json
```

or

```sh
meson setup build
meson compile -C build
meson install -C build
```

Tests and project checks:

```sh
cargo test --workspace
cargo xtask check-rules
cargo xtask check-doc-paths
cargo xtask check-pure
cargo xtask install-hooks
```

## Code

The workspace has five crates. `crates/core` holds the protocol, the session and the equalizer math with no I/O;
`crates/hid`, `crates/audio` and `crates/ui` are the hidraw, PipeWire and GTK adapters; `crates/app` wires them
into the binary. The Shell extension lives in `extension/`.

- [CONTEXT.md](CONTEXT.md): domain terms
- [docs/architecture](docs/architecture/README.md): how the code is laid out
- [docs/adr](docs/adr): design decisions
- [docs/research](docs/research): HID protocol and hardware notes

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

Not affiliated with SteelSeries. Arctis is a trademark of SteelSeries ApS.
