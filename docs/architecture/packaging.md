# Packaging, translations and release

How the app is built into a Flatpak, how its strings reach pt_BR, how a tag becomes a GitHub release, and how a
computer gets the app, the udev rule and the Shell extension. The hand-run hardware list is
`docs/hardware-checklist.md`.

## Build

```mermaid
flowchart LR
    FP["Flatpak manifest<br/>GNOME 50, rust-stable, llvm22"] --> MS["meson.build"]
    MS --> PO["po/meson.build<br/>.mo per language"]
    MS --> DM["data/meson.build"]
    DM --> MF["i18n.merge_file<br/>desktop entry, metainfo"]
    DM --> I["gschema, D-Bus service,<br/>Headset1 XML, icons"]
    MS --> CB["cargo build --release<br/>package arctis-app"]
    PO & MF & I --> T["meson test: msgfmt --check,<br/>desktop-file-validate, appstreamcli,<br/>glib-compile-schemas --strict"]
```

| Piece | Path |
|---|---|
| Flatpak manifest | `build-aux/io.github.marcelogm.ArctisCompanion.json` |
| Top-level meson | `meson.build` |
| Catalogue build and check | `po/meson.build` |
| Data install and validation | `data/meson.build` |
| Desktop entry template | `data/io.github.marcelogm.ArctisCompanion.desktop.in` |
| AppStream template | `data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in` |
| D-Bus activation | `data/io.github.marcelogm.ArctisCompanion.service.in` |
| udev rule | `data/udev/70-arctis-companion.rules` |

| finish-arg | Why |
|---|---|
| `--share=ipc` | shared memory for the X11 fallback |
| `--socket=wayland`, `--socket=fallback-x11` | the window |
| `--device=all` | the dongle's `/dev/hidraw*` nodes; Flatpak has no narrower class for hidraw (`docs/research/flatpak-pipewire.md`) |
| `--filesystem=xdg-run/pipewire-0` | the PipeWire socket, not mounted by default |

Notifications, the Background portal and the file chooser go through portals; the app owns its own app-id name on
the session bus. `crates/app/tests/packaging.rs` pins the finish-args, the version shared by `meson.build`,
`Cargo.toml` and the metainfo, the metainfo's known gaps and permission text, and the release assets.

## Translations

```mermaid
flowchart LR
    R["Rust: msgid(), forms()"] --> X["po/extract.sh"]
    B["Blueprint: _()"] --> X
    D["desktop entry, metainfo"] --> X
    X --> POT["po/arctis-companion.pot"]
    POT --> PT["po/pt_BR.po"]
    PT --> C1["po/check.sh<br/>CI job i18n"]
    PT --> C2["census test<br/>every view msgid translated"]
    PT --> MO["meson: arctis-companion.mo"]
    PT --> AB["crates/ui/build.rs<br/>msgfmt --xml metainfo"]
```

`xgettext --language=Rust` only sees bare calls, so every English literal that reaches `gettext` is wrapped in an
identity `const fn`: `crates/core/src/l10n.rs::msgid` for one string, `crates/core/src/l10n.rs::forms` for a
singular and plural pair.

| Piece | Path |
|---|---|
| Markers | `crates/core/src/l10n.rs` |
| Extraction | `po/extract.sh` |
| Template and catalogue | `po/arctis-companion.pot`, `po/pt_BR.po`, `po/LINGUAS` |
| Catalogue check | `po/check.sh` |
| Census of view msgids | `crates/core/tests/view/catalogue.rs` |
| Merged metainfo for the About dialog | `crates/ui/build.rs`, `crates/ui/src/app_dialogs.rs::about_dialog` |
| Extension catalogue | `extension/po/extract.sh`, `extension/po/pt_BR.po` |

```mermaid
sequenceDiagram
    participant A as application::run
    participant L as locale::init
    participant G as gettextrs
    participant T as text::resolve
    A->>L: init()
    L->>G: setlocale(LC_ALL, "")
    L->>G: bindtextdomain(arctis-companion, prefix/share/locale)
    L->>G: textdomain(arctis-companion)
    A->>A: register_resources(), build().run()
    T->>G: gettext / ngettext per Text
```

| Step | Symbol |
|---|---|
| Startup | `crates/app/src/application.rs::run` |
| Domain binding | `crates/app/src/locale.rs::init`, `crates/app/src/locale.rs::locale_dir_beside` |
| Text resolution | `crates/ui/src/text.rs::resolve` |

## Release

```mermaid
flowchart LR
    TAG["tag v*"] --> V["version<br/>tag = meson version"]
    TAG --> F["flatpak<br/>arctis-companion.flatpak"]
    TAG --> E["extension<br/>gnome-extensions pack"]
    V & F & E --> P["publish<br/>gh release create"]
    PR["pull request touching release.yml"] --> F & E
```

`.github/workflows/release.yml` publishes three assets: the bundle, the extension zip and
`data/udev/70-arctis-companion.rules`. Pull requests that touch the workflow run the build jobs only.

## Install on a computer

```mermaid
sequenceDiagram
    actor U as User
    participant F as flatpak
    participant D as udev
    participant S as gnome-extensions
    participant A as Arctis Companion
    U->>F: flatpak install --user arctis-companion.flatpak
    U->>D: copy 70-arctis-companion.rules, reload, trigger
    U->>S: gnome-extensions install the zip, enable it
    U->>A: launch
    alt dongle denied
        A-->>U: Permission Required page and PermissionDialog with the rule and the command
    else dongle readable
        A-->>U: headset page
    end
```

The dialog is `crates/ui/src/permission_dialog.rs::PermissionDialog`.

## Docs check

`xtask/src/check_doc_paths.rs::run` reads `docs/architecture/`, `CONTEXT.md` and the "- Code:" lines of
`docs/adr/`.
