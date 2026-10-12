# Review log: second adversarial review (2026-10-12)

Decisions on each finding against the design of 2026-10-11. "Fixed" names where the fix lives; "merged" points to
the finding whose fix covers it. Spec parts: design, device, audio, ui, quality (all in
`docs/superpowers/specs/2026-10-11-arctis-companion-*.md`).

| # | Issue | Decision | Rationale |
|---|---|---|---|
| 1 | Our own write echoed as a setting event is taken as intent | Fixed: device §3 `echo.rs` with `ECHO_WINDOW`; SP0 item; Simulation scenarios for a drag echo and a real press | an echo must never move desired back |
| 2 | Exact EQ equality fails if the device quantizes | Fixed: `core::eq::curve::equivalent` (Q ±1 %, freq ±1 Hz, disabled bands equal) used by verify, "already matches" and adoption; SP0 measures and may widen; Flat is the factory curve | one tolerant comparison works before and after SP0, a canonicalize rule needs SP0's exact quantization first |
| 3 | The ~20 s flap comes from fwupd; 3 s settle unjustified | Fixed: ADR 0005, device §2, `hid-protocol.md` §7.4 note; `REBIND_SETTLE` 100 ms; frames during settles dropped; SP0 stops fwupd; 300 ms check in SP14 "when not Rebinding" | the evidence is strong and the design must hold on every fwupd host |
| 4 | EACCES while Rebinding is undefined | Fixed (with #21): retried in Rebinding until `REBIND_GIVE_UP`; PermissionDenied only from Absent | give-up into Absent, then discovery, covers a persistent EACCES without a second 2 s timer |
| 5 | Route chosen from the existence of a bluez sink | Fixed: audio §1 route from the default sink's terminal node; EQ sink only replaces a Nova 5 bluez default, restored exactly; design §1 states dongle-only scope; SP0 sends `0x22` | both links can be up while one plays |
| 6 | Power-on detection waits for the 5 min poll | Fixed: `HEADSET_OFF_POLL` 10 s only in HeadsetOff, ADR 0006, scenario | no 2.4 GHz audio flows while the headset is off, so no glitch risk |
| 7 | Pedantic clippy demands doc comments | Fixed (same as #15) | – |
| 8 | Single readback at 50 ms; `35` may drop the link | Fixed: `VERIFY_BACKOFF` 50/150/400 ms, `WIRELESS_MODE_GRACE`; SP0 items | latency is unknown until measured |
| 9 | Portal app id is on the client for native clients | Fixed: audio §4 reads node props, then the Client named by `client.id`; table tests for both shapes | matches PipeWire's module-access behaviour |
| 10 | `Cargo.lock` comment header | Fixed (same as #16) | – |
| 11 | bindgen needs libclang | Fixed: `org.freedesktop.Sdk.Extension.llvm22` (exists for branch 25.08, the GNOME 50 base), `clang-devel pipewire-devel` in CI, SP1 builds `crates/audio` in Flatpak | – |
| 12 | udev rule misses `0x2255` | Fixed: rule covers every non-bootloader PID; `crates/core/tests/udev_rule.rs` asserts it | a test is simpler than generating the rule |
| 13 | "Plays identically on either engine" is unverified | Fixed: headset curve called nominal; preamp difference stated (audio §1, ADR 0008) | the firmware's filter math is unknown |
| 14 | App rules miss streams behind EasyEffects | Fixed: `links.rs` follows chains up to 4 hops; the EQ sink is not inserted behind EasyEffects | – |
| 15 | Blocking: clippy doc lints vs no-comments | Fixed: lints allowed and guarded by `check-rules`; no `pub unsafe fn`; SP1 fixture | – |
| 16 | Blocking: comment check rejects generated files | Fixed: one list in `xtask/src/check_rules/generated.rs` for both checks; SP1 checks a fresh `Cargo.lock` and `pt_BR.po` | – |
| 17 | Repo makes the 400-line limit advisory, spec makes it hard | Resolved by the user: the limit is a general directive. `xtask` warns in CI and in the hook and never fails | user decision, 2026-10-12 |
| 18 | `Bridge::run` takes 6 arguments | Fixed: `Bridge::new(companion, adapters).run()` (design §3.1) | – |
| 19 | `Companion` inputs, actions and timers unlisted | Fixed: design §3.2 tables, `deadlines.rs`, `companion/timing.rs`, `crates/app/tests/bridge.rs` | – |
| 20 | Lifecycle misses flap in Settling/Probing, unplug, give-up, cadences | Fixed: device §2 diagram with a Present composite state, `REBIND_GIVE_UP`, cadence per phase, scenarios | – |
| 21 | EACCES during reopen flips to Permission Required | Fixed (merged #4) | – |
| 22 | EQ equality in verify and adoption | Fixed (merged #2) | – |
| 23 | 2.4 GHz preset depends on PipeWire seeing the USB sink | Fixed: the headset engine runs whenever the dongle is Ready; the USB sink (matched through its Device) only feeds the route | `0x33` is plain HID; the route still needs to know where this computer plays |
| 24 | Manual pick vs route preset undefined | Fixed (with #38): audio §4 "User picks" table, CONTEXT.md | – |
| 25 | Acceptance criteria need sub-projects in the same wave | Fixed: messages frozen in SP2; end-to-end scenario in SP8; hardware and PipeWire checks in SP14; SP2 → SP7 edge | – |
| 26 | Sync policy can arrive late; pre-adoption edits lost | Fixed: one `Change::Desired { settings, policy }`; no write without it; pre-adoption edits prevented (#41) | tracking touched fields was rejected as more state than hiding the controls |
| 27 | Attribution can land on `main` via server-side merges | Fixed: ruleset allows only rebase merging and requires `rules`; PR title checked; `rules` also on push to `main` | the user strictly forbids co-authorship in commits and PRs |
| 28 | "Started most recently" flips with corking | Fixed: ruled set with `RULE_ENTER`/`RULE_LEAVE`, recency by join time, 2 s budget on Bluetooth | – |
| 29 | DSP acceptance wrong for shelves and pass filters | Fixed: per-kind criteria in quality §1; pass filters store 0 dB | – |
| 30 | Where violation fixtures live | Fixed: built in tempdirs by `xtask/tests` | – |
| 31 | Doc check stops at paths; proposals ignored | Partly fixed: `path.rs::Item` symbols checked, ADR "Code:" lines from SP14. Proposals part rejected | `git check-ignore` shows the proposals are not ignored; `.gitignore` has 7 lines and the cited lines 8-10 came from a concatenated listing |
| 32 | RT mailbox and crossfade untested | Fixed: `triple_buffer`, `Equalizer::retarget` in core with allocation and continuity tests, `crates/audio` forbids `unsafe` unless `pw_filter` is chosen | – |
| 33 | Late echo on iface 3 overwrites desired | Fixed (merged #1) | – |
| 34 | Global `settings-adopted` flag | Fixed: `adopted-identity` = PID + `HID_UNIQ` | a firmware string was rejected as identity: a firmware update would re-adopt |
| 35 | Blocking: Bluetooth use case unreachable without the dongle | Fixed: ui §1 Phase × Route, Bluetooth-only page, reworded status page, `State` `bluetooth-only`, `core::view` test | – |
| 36 | Bluetooth preset is not the official Dual EQ | Fixed: SP0 `0x22`/`0x34`, metainfo gap (ADR 0013), row renamed "Bluetooth from This Computer", shown after a bluez node is seen | – |
| 37 | EQ sink takes over the default; volume keys hit the virtual sink | Fixed (merged #5): insertion and restore rules, `volume.rs` forwards volume and mute. Hiding the raw bluez node rejected | `node.hidden` is unverified and only cosmetic |
| 38 | Combo row vs manual pick | Fixed: combos always set and show the route preset; "Now Playing" row shows overrides (ui §2) | – |
| 39 | Toggle titled with the device name | Fixed: "App Presets", `AppRuleCount`, header carries model and battery (ui §5) | – |
| 40 | Extension cannot read the battery threshold | Fixed: `BatteryWarning` property from `alerts.rs` | – |
| 41 | First launch with the headset off shows schema defaults | Fixed: headset groups hidden and banner "Turn on the headset to read its settings" until adoption; `ActivatePreset` returns `NotReady` | – |
| 42 | "Other App…" binary names never match Flatpak apps | Fixed: rules match any identifier; seen apps persisted; identifier shown per rule | – |
| 43 | Scales announce numbers, custom drawings undescribed | Fixed: `value_text` per `ScaleRow`, accessibility walk extended | – |
| 44 | i18n has no arguments or plurals | Fixed: typed `Text` resolved in `crates/ui/src/text.rs`; one msgid per setting; extension uses `toLocaleString()` | – |
| 45 | No empty states | Fixed: ui §4 empty states, each a `core::view` state | – |
| 46 | Deleting a referenced preset | Fixed: unknown ids resolve to Flat or "Preset missing"; toast names the uses | – |
| 47 | Background switch dead after a denial | Fixed: re-request on user action; denial turns the switch off with a hint. No button to open Settings | opening GNOME Settings from the sandbox needs an extra D-Bus permission |
| 48 | Banner inside a group; competing banners | Fixed: one window banner with a priority order; group problems as warning rows. No "Open Sound Settings" button | same sandbox permission as #47 |
| 49 | 2.4 GHz preset lost without PipeWire | Fixed (merged #23) | – |
| 50 | Cannot audition a non-active preset; gain on pass filters | Fixed: "Preview" toggle makes a manual pick while the editor is open; gain insensitive for pass filters | – |
| 51 | Header link label implies the headset's audio mode | Fixed: the header names this computer's route; SP0 captures QuickSwitch with both links up | – |
| 52 | App curve stays on the headset after crash or suspend | Partly fixed: orderly quit restores the route preset, reconnect reconciles, residual risk shown on the page. Suspend hook rejected | logind's PrepareForSleep needs system-bus access the sandbox lacks |
| 53 | 90-day retention with only Day and Week views | Fixed: 8-day retention (ADR 0012) | YAGNI; a Month view can return later |
| 54 | Failed write leaves the control showing a rejected value | Fixed: row shows observed with "Not applied"; reconnect retry stated (device §4) | – |
