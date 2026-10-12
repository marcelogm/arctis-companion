# Single background-capable app process, no daemon

Status: accepted (2026-10-11), revised 2026-10-12

The Flatpak app itself owns the dongle, the PipeWire client, the D-Bus object, the battery log and the notifications. When the window closes it keeps running under a `gio::ApplicationHoldGuard`, granted by the Background portal (requested through `ashpd`, with autostart and `--gapplication-service`), while the GSettings key `run-in-background` is on. The user did not choose a daemon, and one owner inside the app means no IPC between owners.

## Considered options

- Separate host daemon (systemd user service, as Linux-Arctis-Manager): needs host installation beyond the udev rule and a second permission story.
- Second binary inside the Flatpak: two processes competing for the hidraw nodes.
- Hand-written portal calls through `gio::DBusProxy`: smaller dependency tree, but the Request/Response handshake (handle token, subscribe before calling) is subtle code we would own; `ashpd` is the Circle-app norm.

## Consequences

- If the user denies background, everything still works while the window is open. The portal is asked at first run and again only when the user turns "Run in Background" on; a denial turns the switch back off and its subtitle points to Settings → Apps.
- fwupd also reaches the dongle (it rebinds the command interface every ~20 s); the app absorbs that as Rebinding (ADR 0005).
- `ashpd` brings `zbus` for portal calls only; our own D-Bus object stays on GIO (ADR 0002).
- Code: `crates/app/src/application.rs`, `crates/app/src/platform/background.rs`.
