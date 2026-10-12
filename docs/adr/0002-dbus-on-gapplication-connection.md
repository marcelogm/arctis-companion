# Headset1 exported through GIO on GApplication's connection; the extension never starts the app

Status: accepted (2026-10-11)

`io.github.marcelogm.ArctisCompanion.Headset1` is exported with `gio::DBusConnection::register_object` from the `dbus_register` vfunc, on the connection and well-known name GApplication already owns (the app id, allowed in Flatpak without `--own-name`). The Shell extension builds its proxy with `DO_NOT_AUTO_START` and hides itself when the name has no owner, so it never revives a process the user put to rest.

## Considered options

- A zbus server on the app id: rejected, a second connection cannot own the name GApplication holds, so the extension would not find the object.
- A zbus server on a subname: works, but adds a second connection and executor next to the GLib loop for no gain.
- Letting the extension D-Bus-activate the app: would override the user's "Run in Background" choice.

## Consequences

- The interface XML in `data/dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml` is the single contract; the extension's copy is checked byte-equal by `xtask`.
- Properties only change through `PropertiesChanged`; the extension holds no state of its own.
- Code: `crates/app/src/dbus/headset_object.rs`, `extension/arctis-companion@marcelogm.github.io/lib/client.js`.
