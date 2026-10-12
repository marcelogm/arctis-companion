# Hardware checklist

A person runs this list by hand on a computer with the SteelSeries Arctis Nova 5 dongle plugged in. Every box
starts unticked; only the person who ran a row ticks its `Done` box, and records measured values where the row says
so. No agent or script ticks a box.

Before you start:

- Scripts that talk to the dongle must keep the `Command` allow-list guard from SP0
  (`docs/research/hardware-findings.md` §0, "Scripts"): no other report goes to `/dev/hidraw*`.
- The sidetone may be one step away from your usual value: an earlier soak run stopped without a summary
  (`docs/research/hardware-findings.md` §11). Check it in the app before running H14 or A1.
- Install the release first (rows F1 and E9) unless a row says otherwise.

## H — Device (hardware-findings §7)

For every H row, "passes" means: the measured value is recorded in `docs/research/hardware-findings.md` §7 and, if it
differs from the designed default, an issue is opened against the named constant (`REBIND_SETTLE`,
`VERIFY_BACKOFF`, `ECHO_WINDOW`, `WIRELESS_MODE_GRACE`, `STATUS_SAFETY_POLL`, `AUTO_SWITCH_MIN_INTERVAL`, the
`crates/core/src/eq/curve.rs` tolerances or the sidetone marks).

| # | Check | How | Designed default meanwhile | Done |
|---|---|---|---|---|
| H1 | Persistence across a power cycle | record `20 A0 36`; write `39 03` and `37 0e`, verify; power the headset off and on; read again; restore | desired state re-applied on every Syncing (ADR 0007) | [ ] |
| H2 | Event interface for buttons and dial | listen on iface 3, 4, 5 while pressing mute, turning the volume and ChatMix dials, plugging the charger | read both iface 3 and 5 (ADR 0006) | [ ] |
| H3 | Echo of host writes | headset on: write each setter, log all interfaces for 1.5 s | `ECHO_WINDOW` 1 s | [ ] |
| H4 | Setter to readback, headset on | as hardware-findings §2.3 with the headset on | `VERIFY_BACKOFF` 50, 150, 400 ms | [ ] |
| H5 | `35 0` / `35 1` with the headset on | write, then `36` and `B0` every 100 ms for 10 s; note `B9`; check the official app's Extended range toggle after `35 00` | `WIRELESS_MODE_GRACE` 5 s; 0 = Extended range | [ ] |
| H6 | EQ quantization, headset on | write `cmd_33_odd.hex`, power-cycle the headset, read `20`, restore | ±1 Hz, ±1 % | [ ] |
| H7 | Sidetone Low/Medium/High | set each level in the official app, read `20[62]` | marks 0, 3, 6, 10 | [ ] |
| H8 | Audible glitches | play music; `B0` every 2 s for 1 min, then every 60 s for 5 min; one `33` write | 5 min safety poll, 30 s auto-switch interval | [ ] |
| H9 | Unreadable window without `92-viia.rules` | as root move the rule away, `udevadm control --reload`, measure `EACCES` time after a rebind, restore the rule | Rebinding retries `EACCES`, give up after 30 s | [ ] |
| H10 | Bluetooth curve `34` | phone paired, phone audio playing: `34` with a clearly audible curve, `22` readback, restore | known gap "Bluetooth preset for the phone" (ADR 0013) | [ ] |
| H11 | QuickSwitch | both links up, press QuickSwitch, log `B0[1]` and events | no "playing from Bluetooth" state | [ ] |
| H12 | Nova 5 bluez node | pair over Bluetooth, `pw-dump` the node and its Device | match by `device.api` and description prefix | [ ] |
| H13 | Optional flap confirmation | `systemctl stop fwupd` for 2 min, `journalctl -k` shows no `input3` rebind, start it again | fwupd is the cause (hardware-findings §4) | [ ] |
| H14 | SP5 soak with the headset on | turn the headset on, set Automatic power off to Never for the run, then `ARCTIS_HW=1 cargo test -p arctis-hid --test hardware -- --ignored --nocapture`; passes when the `soak:` line shows writes requested = confirmed and failed 0 | write then verify (device §4) | [ ] |

## A — Release acceptance on hardware

| # | Check | How | Passes when | Done |
|---|---|---|---|---|
| A1 | 2.4 GHz edit latency | headset on, run H14: it drives the real `crates/hid/src/worker.rs::Worker` and confirms each sidetone write by an `Observed` readback; writes that cross a Rebinding are counted as interrupted, not timed | the `soak:` line's "slowest confirmation" is at most 300 ms | [ ] |
| A2 | Ruled app | add a rule for a player (for example `org.gnome.Totem` or `firefox`) with a preset that is not the current one, start playback, then stop it; time it with a stopwatch or the debug log timestamps | the preset row changes 1 s after playback starts and returns 10 s after it stops | [ ] |
| A3 | Bluetooth system engine | on a Bluetooth-paired computer, play audio through "Arctis Nova 5 (Equalizer)" for 30 minutes at 48 kHz with quantum 256 (`pw-metadata -n settings 0 clock.force-quantum 256`); watch `pw-top` | `pw-top` shows ERR 0 for the EQ node at the end | [ ] |

## F — Flatpak and desktop

| # | Check | How | Passes when | Done |
|---|---|---|---|---|
| F1 | Install | `flatpak install --user arctis-companion.flatpak`; `sudo cp 70-arctis-companion.rules /etc/udev/rules.d/ && sudo udevadm control --reload-rules && sudo udevadm trigger`; launch from the app grid | the headset page appears | [ ] |
| F2 | Missing rule | on another computer, or with the rule moved away as root: launch the app | the "Permission Required" page and its dialog show the rule and the command; both copy buttons work | [ ] |
| F3 | Plug-in retry | with the rule installed, unplug the dongle and plug it back by hand | the permission page never flashes | [ ] |
| F4 | Background | turn Run in Background on and close the window; then log out and in | the process stays, GNOME's background apps list shows "Headset connected, NN %"; after login the app starts in the background | [ ] |
| F5 | Low battery | let the battery fall below the threshold, click the notification, then plug the charger | the notification appears, opens the battery page, and is withdrawn while charging | [ ] |
| F6 | pt_BR | if `flatpak info --user io.github.marcelogm.ArctisCompanion.Locale` does not list `pt`, run `flatpak config --user --set extra-languages pt_BR && flatpak update --user`; quit the running instance and start `LANG=pt_BR.UTF-8 LANGUAGE=pt_BR flatpak run io.github.marcelogm.ArctisCompanion` | every page, dialog, toast and the About dialog are in Portuguese, numbers use a decimal comma, nothing is truncated at 360 px wide | [ ] |

## U — UI by hand

| # | Check | How | Passes when | Done |
|---|---|---|---|---|
| U3 | Preset files | export a preset as JSON and as APO text, import both back, then import a published AutoEQ file | each import creates the expected preset; clamped values are reported in a toast with Details | [ ] |
| U4 | Rename and Details dialogs | open the Rename dialog and the Details dialog of an import toast and use them with the keyboard only | both open, take focus, accept and cancel with the keyboard | [ ] |

## B — Bluetooth audio

| # | Check | How | Passes when | Done |
|---|---|---|---|---|
| B1 | bluez node properties | `pw-dump` the Nova 5 Bluetooth node | `device.api` is `bluez5` and the description starts with "Arctis Nova 5", as `crates/audio/src/sinks.rs` expects | [ ] |
| B2 | Insert and remove | with the Flatpak app running, switch the default output between the Nova 5 Bluetooth sink and the speakers several times | the EQ sink is inserted and removed each time and the previous default comes back | [ ] |
| B3 | Manager role denied | inside the Flatpak, if WirePlumber denies the Manager role | the warning row "Choose “Arctis Nova 5 (Equalizer)” as output in Sound settings" appears | [ ] |

## E — Shell extension

Run each scenario in `extension/` with `npm run devkit -- <scenario>` and compare with the Expect column (from the
SP13 pull request).

| # | Scenario | Expect | Done |
|---|---|---|---|
| E1 | connected | toggle "App Presets" checked, subtitle "Flat"; header "Arctis Nova 5" / "73 %"; three presets, Flat checked; no panel icon | [ ] |
| E2 | low-battery | panel icon battery-caution-symbolic; header "14 %" | [ ] |
| E3 | charging | panel icon battery-level-40-charging-symbolic; header "Charging · 47 %" | [ ] |
| E4 | app-rule | subtitle "Footsteps · Firefox" | [ ] |
| E5 | manual-pick | subtitle "Bass Boost · Until Firefox stops" | [ ] |
| E6 | no-rules | subtitle "No app rules"; switch part insensitive; arrow still opens the menu | [ ] |
| E7 | connecting | preset items insensitive | [ ] |
| E8 | absent | no toggle, no panel icon | [ ] |
| E9 | installed extension | enable the installed extension yourself in the Extensions app, then start and quit the Flatpak app | the toggle appears while the app runs and hides when it quits | [ ] |
