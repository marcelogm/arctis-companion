# Arctis Companion

A GNOME companion for the SteelSeries Arctis Nova 5 headset family on Linux: it configures the headset, shapes its
sound, and reports its battery, matching the official mobile app and adding per-app presets, a computer-side
equalizer and battery history.

## Language

### Hardware

**Headset**:
The Arctis Nova 5, 5P or 5X worn by the user; it holds its own settings and equalizer.
_Avoid_: device (ambiguous with the dongle), headphones

**Dongle**:
The 2.4 GHz USB receiver plugged into the computer; the only way the app talks to the headset.
_Avoid_: receiver, transmitter, base station

**Model**:
The headset variant identified by the dongle's product id, with its support level and capabilities.
_Avoid_: PID (the number, not the concept), SKU

**Capability**:
A feature a model is known to support; controls exist only for capabilities.
_Avoid_: feature flag

**Link**:
The state of the wireless connection between dongle and headset: not paired, searching, headset off, connected.
_Avoid_: connection status, pairing

**Command interface** / **Event interface**:
The dongle's two vendor HID interfaces: one receives commands and answers queries, the other pushes events.
_Avoid_: iface 3 / iface 5 in prose, control endpoint

### Talking to the headset

**Command**:
One frame the app is allowed to send: a query or an absolute setter. Anything else is a forbidden frame.
_Avoid_: request, packet, message

**Reply**:
The dongle's answer to a query, read only up to its documented length.
_Avoid_: response, ACK (there is none)

**Stale tail**:
Leftover bytes from an earlier reply that trail a new reply and carry no meaning.

**Event**:
An unsolicited frame reporting something that changed on the headset (battery, charging, link, a button or dial).
_Avoid_: notification (reserved for desktop notifications), interrupt

**Readback**:
The query sent after a setter to confirm the headset took the value, since writes are never acknowledged.
_Avoid_: ACK, confirmation

**Safety poll**:
A rare status query that covers missed events; never a regular sampling mechanism.
_Avoid_: polling, refresh

**Settle**:
The quiet wait after the dongle appears or returns before the first command.

**Rebind**:
The command interface briefly disappearing and coming back while the dongle stays plugged in, caused every ~20 s by
fwupd polling the headset; the app rides through it without querying and without telling the user.
_Avoid_: disconnect, reconnect, flap (in user-facing text)

**Session**:
The life of one dongle from discovery to removal, moving through its phases.

**Phase**:
Where a session stands: Absent, Permission Denied, Settling, Probing, Headset Off, Syncing, Ready, Rebinding.
_Avoid_: state (too generic), status

**Echo**:
An event the dongle sends back for a value the app itself just wrote; it is consumed, never treated as the user's
intent.
_Avoid_: ACK, confirmation

**Dongle identity**:
The dongle's model id plus its USB serial when it reports one; adoption happens once per identity.
_Avoid_: device id, serial (alone)

### Settings

**Headset setting**:
A value stored and applied by the headset: mic volume, sidetone, mute light brightness, volume limiter, extended
range, automatic power off, headset curve.
_Avoid_: device setting, hardware setting

**App setting**:
A value only the app knows: presets, app rules, run in background, low battery warning level.
_Avoid_: preference (reserved for the Preferences dialog's contents)

**Desired settings**:
The headset settings the user wants, remembered by the app.

**Observed settings**:
The headset settings last read back from the headset.

**Drift**:
A difference between desired and observed settings found when the headset connects.

**Adoption**:
The one-time copy of observed into desired the first time a dongle identity connects, so the app never overwrites
what the user had already configured. Until it happens, headset settings are not editable.
_Avoid_: import, first sync

**Reconcile**:
Writing only the drifted headset settings and verifying each by readback.
_Avoid_: sync, push, restore

**Sidetone**:
How much of the user's own voice the headset plays back, eleven steps marked Off, Low, Medium and High.
_Avoid_: monitoring, retorno (pt_BR UI only)

**Extended range**:
The wireless mode that trades microphone quality for range.
_Avoid_: range mode, speed mode (the opposite setting)

**Mute light**:
The LED that glows while the microphone is muted; its brightness is a headset setting.
_Avoid_: mute LED (in UI text), mic indicator

### Sound

**Path**:
One of the two ways audio reaches the headset that the app can shape: the 2.4 GHz path (the dongle, any source,
consoles included) and the Bluetooth path from this computer. Each path has one engine and one route preset.
_Avoid_: channel, link (reserved for the dongle's wireless state)

**Route**:
Which path carries this computer's audio now, read from the default sound output: Dongle (2.4 GHz), Bluetooth, or
none. It never comes from which outputs merely exist.
_Avoid_: output, audio mode, connection

**Picked path**:
The path a pick from the Shell targets: the route's path, else the 2.4 GHz path when the dongle is present.

**Engine**:
What applies the equalizer on a path: the headset's own equalizer on the 2.4 GHz path, the system equalizer on the
Bluetooth path from this computer. An engine never processes another path's audio.
_Avoid_: backend, layer

**System equalizer**:
The equalizer the app runs on this computer's audio, used only on the Bluetooth path from this computer.
_Avoid_: computer EQ, software EQ, PipeWire EQ (in UI text)

**Envelope**:
The limits every preset must respect so it means the same on either engine: ten filters, ±10 dB, fixed kinds,
frequency and Q ranges, no gain on low-pass and high-pass.

**Filter**:
One band of a preset: kind, frequency, gain, Q, enabled.
_Avoid_: band (UI may say band), biquad (an implementation)

**Curve**:
The ordered filters of a preset as the headset or the system equalizer receives them.
_Avoid_: EQ settings, profile

**Equivalent curves**:
Two curves that differ only by the rounding the headset applies when it stores one; every curve comparison uses
this, never byte equality.

**Preset**:
A named curve the user can pick.
_Avoid_: profile, EQ, sound mode

**Measurement**:
The Nova 5's frequency response measured by RTINGS.com on a Brüel & Kjær 5128, shipped with the app and drawn as
"Headset".
_Avoid_: graph, FR (in UI text)

**Target**:
A frequency response for the same rig that a curve aims the headset at: Harman (LMG 5128 0.6) or diffuse field.
_Avoid_: target curve (ambiguous with Curve), house curve

**Built-in preset** / **Custom preset**:
A read-only preset shipped with the app / a preset the user created, duplicated or imported.

**Route preset**:
The persistent preset of a path when no app rule or manual pick applies: the 2.4 GHz preset and the Bluetooth
preset from this computer. The window's combo rows always set and show it.
_Avoid_: default preset, dual EQ

**Active preset**:
The preset currently applied on a path.

**App**:
A program identified by its app key, as seen in its audio streams.
_Avoid_: application (in code names), process, game

**App identifiers**:
What a stream says about its app: its Flatpak app id, its declared application id and its binary name.

**App key**:
The first present app identifier; it names an app among the seen apps.

**App rule**:
An association from one app identifier to a preset; it matches any stream that carries that identifier.
_Avoid_: profile, per-app EQ

**Ruled set**:
The apps with a rule that are playing on a path, joined after 1 s of playback and left after 10 s of silence; the
one that joined last chooses the active preset.

**Automatic app presets**:
The behaviour that lets app rules choose the active preset while ruled apps play.
_Avoid_: auto EQ, auto profiles

**Manual pick**:
A preset the user chose by hand while a ruled app plays; it holds until that path's ruled set changes, the route
changes, or the app restarts.
_Avoid_: override (in UI text)

**Preset reason**:
Why the active preset is active: a manual pick, the route preset, or a named app.

### Battery

**Battery sample**:
One recorded change of battery level, charging state or link, with its time.
_Avoid_: reading, measurement

**Battery history**:
The battery samples of the last 8 days.
_Avoid_: battery log (the file), stats

**Low battery warning** / **Critical battery warning**:
The two battery warning levels, reached at the user's threshold and at 5 %: each sends one desktop notification per
discharge, withdrawn when charging starts, and shows the Shell's panel icon.
_Avoid_: alert, alarm

### Surfaces

**Companion**:
The app as a whole, including when it runs in the background without a window.
_Avoid_: daemon, service

**Quick Settings toggle**:
The Shell extension's entry titled "App Presets"; it switches automatic app presets and lists presets.
_Avoid_: tray icon, indicator (the Shell class name)

## Relationships

- A **Dongle** serves exactly one **Headset**; a **Session** belongs to one **Dongle**.
- A **Headset setting** has **Desired** and **Observed** values; **Drift** is their difference; **Reconcile** removes it.
- Each **Path** has one **Route preset** and one **Engine**; the **Route** names the path this computer plays to.
- An **App rule** names one **Preset**; on each path the **Active preset** comes from a **Manual pick**, the newest app of the **Ruled set**, or the **Route preset**, recorded as its **Preset reason**.
- **Adoption** happens once per **Dongle identity**.

## Example dialogue

> **Dev:** "The headset rebound twice during a reconcile, did we resync?"
> **Domain expert:** "No. A **Rebind** keeps the **Session** in Rebinding, sends nothing, and resumes the
> **Reconcile** where it stopped; only a new **Link** to connected triggers Syncing."

## Flagged ambiguities

- "Bluetooth" means three things: the **Path** and **Route** from this computer, the headset's phone pairing, which
  the app cannot see, and the official app's Bluetooth preset, which shapes phone audio and is a known parity gap.
  Only the first is a domain term; UI text says "Bluetooth from This Computer".
- "Profile" was used for presets, app rules and the official app's EQ; it is not a term here.
