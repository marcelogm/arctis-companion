# Shell extension

The Quick Settings toggle "App Presets" for GNOME Shell 50. It reads the app's `Headset1` object, holds no state
of its own and hides while the app's bus name has no owner (ADR 0002). Directory:
`extension/arctis-companion@marcelogm.github.io/`; contract copy
`extension/arctis-companion@marcelogm.github.io/dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml`, byte-equal
to `data/dbus/io.github.marcelogm.ArctisCompanion.Headset1.xml` (`xtask/src/check_rules/dbus_copy.rs::check`).

```mermaid
flowchart LR
    subgraph Shell["gnome-shell process"]
        EXT["extension.js<br/>ArctisCompanionExtension"]
        TOG["lib/toggle.js<br/>AppPresetsToggle"]
        IND["lib/indicator.js<br/>CompanionIndicator"]
        PRE["lib/presentation.js<br/>present"]
        CLI["lib/client.js<br/>HeadsetClient"]
        FAIL["lib/failures.js<br/>reportFailure"]
    end
    APP[("app process<br/>Headset1 at /io/github/marcelogm/ArctisCompanion")]
    EXT --> CLI & PRE & TOG & IND
    TOG & EXT --> FAIL
    CLI <-- "PropertiesChanged, Properties.Set,<br/>ActivatePreset, Application.Activate" --> APP
```

| Module | Path | Shell imports |
|---|---|---|
| Entry point | `extension/arctis-companion@marcelogm.github.io/extension.js` | yes |
| Presentation | `extension/arctis-companion@marcelogm.github.io/lib/presentation.js` | none, tested |
| D-Bus client | `extension/arctis-companion@marcelogm.github.io/lib/client.js` | none, tested |
| Failure log | `extension/arctis-companion@marcelogm.github.io/lib/failures.js` | none, tested |
| Toggle and menu | `extension/arctis-companion@marcelogm.github.io/lib/toggle.js` | yes |
| Panel icon | `extension/arctis-companion@marcelogm.github.io/lib/indicator.js` | yes |
| Metadata | `extension/arctis-companion@marcelogm.github.io/metadata.json` | – |

## Updates

```mermaid
sequenceDiagram
    participant E as ArctisCompanionExtension
    participant C as HeadsetClient
    participant A as app Headset1
    participant P as present
    participant T as AppPresetsToggle and CompanionIndicator
    E->>C: createProxy (DO_NOT_AUTO_START), start()
    A-->>C: name owner appears, properties loaded
    C-->>E: onChange
    E->>P: present(properties() or null, gettext)
    P-->>E: visible, panelIcon, title, subtitle, sensitive, checked, header, items
    E->>T: render(presentation)
    T->>C: toggle clicked: setAutomaticPresets(!checked)
    C->>A: Properties.Set AutomaticPresets
    A-->>C: PropertiesChanged AutomaticPresets
    T->>C: preset item: activatePreset(id)
    T->>C: Headset Settings: activateApplication(token from the Shell launch context)
    C->>A: Application.Activate with activation-token, so mutter raises the window
```

| `State` | Shown |
|---|---|
| no owner, `absent`, `permission-denied` | nothing (the window explains these) |
| `connecting` | toggle, preset items insensitive |
| `awaiting-headset` | toggle, preset items insensitive unless `Route` is `bluetooth` |
| `bluetooth-only`, `headset-off`, `connected` | toggle, preset items sensitive |

The switch part is insensitive while `AppRuleCount` is 0; the arrow still opens the preset menu. The panel icon
shows only for a battery warning or while charging.

## Tests

```mermaid
flowchart LR
    RUN["tests/run.js<br/>GLib main loop, exit code"] --> S1["presentation.test.js"]
    RUN --> S2["client.test.js"]
    RUN --> S3["metadata.test.js, failures.test.js"]
    S2 --> C["HeadsetClient on the session bus"]
    S2 --> F["FakeHeadsetService<br/>second connection, owns the app name"]
    C <--> BUS[("dbus-run-session")]
    F <--> BUS
```

| Piece | Path |
|---|---|
| Runner and harness | `extension/tests/run.js`, `extension/tests/support/harness.js`, `extension/tests/support/assert.js` |
| Presentation tests | `extension/tests/presentation.test.js` |
| Client tests | `extension/tests/client.test.js` |
| Fake `Headset1` service | `extension/tests/support/fake_headset_service.js` |
| Activatable test bus | `extension/tests/support/session-bus.xml`, `extension/tests/support/activation.js` |
| Nested shells | `extension/tests/nested-shell.sh`, `extension/tests/shell-session.sh`, `extension/tests/fake_app.js` |

## Build and translations

```mermaid
flowchart LR
    SRC["extension/arctis-companion@marcelogm.github.io"] --> PACK["npm run pack<br/>gnome-extensions pack"]
    PO["extension/po: POT + pt_BR.po"] --> PACK
    PACK --> ZIP["shell-extension.zip<br/>lib, dbus, locale"]
    SRC --> X["po/extract.sh<br/>xgettext"] --> PO
    PO --> CHK["po/check.sh: stale POT,<br/>msgfmt --check, untranslated"]
```

Scripts: `extension/package.json`, `extension/po/extract.sh`, `extension/po/check.sh`. CI: job `extension` in
`.github/workflows/ci.yml`.
