# SP14 Packaging and Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Arctis Companion as an installable release: a final Flatpak manifest, metainfo that states the known
gaps and the device permission, a complete `pt_BR` catalogue wired into the app, a tag-driven release workflow that
publishes the `.flatpak`, the extension zip and the udev rule, ADR "Code:" lines under `check-doc-paths`, a
hand-run hardware checklist, and a triage of every follow-up left by SP0–SP13.

**Architecture:** No new runtime component. Packaging is data plus tests: `crates/app/tests/packaging.rs` pins the
manifest's sandbox, the version shared by `meson.build`, `Cargo.toml` and the metainfo, the metainfo's known gaps,
and the release assets. i18n has three parts: a `const fn` marker pair `arctis_core::l10n::{msgid, forms}` that
makes every English msgid visible to `xgettext --language=Rust` (which ignores qualified calls such as
`Text::plain("…")`); `po/extract.sh` + `po/check.sh` (the extension's pattern); and a census test in
`crates/core/tests/view/catalogue.rs` that walks the `Debug` dump of every view model and fails on any msgid
missing from `po/pt_BR.po`. `crates/app/src/locale.rs::init` binds the text domain next to the binary's prefix.

**Tech Stack:** Rust 1.98.1 workspace, `gettext-rs` 0.7.7 (`gettext-system`), GNU gettext 0.26 (`xgettext` Rust
support, `msgfmt`, `msgcmp`, `msgattrib`, `msgcomm`), meson `i18n` module (`gettext`, `merge_file`), AppStream
1.1.3 (`appstreamcli`, `metainfo.its`), `desktop-file-validate`, `org.flatpak.Builder` with `org.gnome.Sdk//50`,
`rust-stable` and `llvm22`, GitHub Actions (`flatpak/flatpak-github-actions/flatpak-builder@v6`,
`actions/upload-artifact@v4`, `actions/download-artifact@v4`, `gh release create`).

Spec: `docs/superpowers/specs/2026-10-11-arctis-companion-design.md` §1, §5 (SP14 row) and §6 (risks owned by
SP14); ui part §4 (i18n) and §2 (permission dialog); quality part §2 (CI, release, i18n) and §3 (durable docs:
"ADR 'Code:' lines join the check in SP14"); device part (udev rule); audio part §1 and §3 (known gaps, translated
built-in names); `docs/research/hardware-findings.md` §7; `docs/research/flatpak-pipewire.md` (finish-args);
ADR 0013 and ADR 0014.

---

## Conventions for every task

- Worktree: `/home/marcelo/Desenvolvimento/arctis-wt/sp14`, branch `sp14-packaging-release`. Never touch
  `/home/marcelo/Desenvolvimento/arctis-companion`. All paths below are relative to the worktree.
- **Run (toolbox):** `X` means
  `toolbox run -c arctis bash -lc 'cd /home/marcelo/Desenvolvimento/arctis-wt/sp14 && X'`.
  Cargo, meson, `xgettext`, `msgfmt`, `appstreamcli`, `npm` and `gnome-extensions pack` run there. Missing packages:
  `sudo dnf install -y <pkg>` inside the toolbox.
- **Run (host):** `git`, `gh`, `flatpak`, `flatpak run org.flatpak.Builder`, `gnome-extensions install`, `gdbus`.
- **Run (ui):** `X` means the GTK tests under a private headless weston, always in this worktree (do not use
  `/home/marcelo/Desenvolvimento/arctis-wt/uitest.sh`: it is hard-wired to the gone `sp12` worktree):
  `toolbox run -c arctis bash -lc 'cd /home/marcelo/Desenvolvimento/arctis-wt/sp14 && export XDG_RUNTIME_DIR=$(mktemp -d) LC_ALL=C.UTF-8 && { weston --backend=headless --socket=wl-sp14 --idle-time=0 >/dev/null 2>&1 & W=$!; } && until [ -S "$XDG_RUNTIME_DIR/wl-sp14" ]; do sleep 0.1; done; GDK_BACKEND=wayland GSK_RENDERER=cairo WAYLAND_DISPLAY=wl-sp14 X; S=$?; kill $W; exit $S'`.
- No comments in any file of any language, doc comments included. Shell scripts may keep the `#!/bin/sh` line only.
  Names and small functions carry the meaning. Clippy limits apply to tests too (cognitive complexity 8, 40 lines,
  nesting 3, 4 arguments).
- New text file types must be known to `xtask/src/check_rules/comments/mod.rs::syntax_of`. This plan adds only
  `.sh`, `.po`, `.pot`, `LINGUAS`, `.yml`, `.json`, `.xml.in`, `.desktop.in`, `meson.build` and `.md`, all already
  known (`.in` is stripped before lookup; `.po`, `.pot`, `LINGUAS` are generated and exempt).
- Before each commit, Run (toolbox): `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings`
  for Rust changes, and `cargo xtask check-rules --staged` after `git add`.
- Commit subjects are conventional and short. Never write `Co-Authored-By`, "Generated with", or any AI or tool
  name in a commit, PR title, PR body or issue. Never use `--no-verify`.
- Push after every commit: `git push -u origin sp14-packaging-release` the first time, then `git push`.
- Hardware: SP14 never opens `/dev/hidraw*` from a script or test. The only thing that touches the dongle is the
  installed Flatpak app itself in Task 19, which uses the allow-listed `Command`s through `crates/hid`.
- Never install the udev rule, never `gnome-extensions enable|disable`, never `systemctl`.

## Acceptance criteria → tasks

| SP14 acceptance criterion or scope item (design §5, quality §2–3) | Task |
|---|---|
| final manifest (`--device=all`, PipeWire socket) | 7, 8 |
| udev rule + in-app instructions | 9 (metainfo text), 18 (packaging page), 19 (shown by the installed app when denied); the dialog itself is SP9's `crates/ui/src/permission_dialog.rs::PermissionDialog`, kept |
| metainfo with known gaps: audio-mode switch, phone Bluetooth preset; `--device=all` justification (risk table) | 7, 9 |
| pt_BR 100 % | 3, 4, 5, 6, 11, 11A (About dialog), 12, 13 |
| `appstreamcli validate`, `desktop-file-validate`, `glib-compile-schemas --strict` pass | 9 (meson tests on the merged files), 19 (Flatpak build runs them) |
| CI regenerates the POT, `msgfmt --check`, fails on untranslated `pt_BR` (quality §2) | 4, 14 |
| `check-doc-paths` covers ADR "Code:" lines | 2 |
| release workflow: tag builds `.flatpak`, extension zip and rule | 7, 10, 21 |
| `flatpak-builder --install` works on Fedora 44 with the rule; app launches from the Flatpak | 19 |
| extension installed for the user (not enabled) | 19 |
| end-to-end checklist on hardware (300 ms readback; ruled app 1 s / 10 s; Bluetooth 30 min 48 kHz / 256 no xruns) | 15A (the soak prints the slowest confirmed write, so 300 ms is measurable), 17 (written for a person, never ticked by an agent) |
| hardware-findings §7 consolidated into `docs/hardware-checklist.md` | 17 |
| issue #3: drop the `.githooks/commit-msg` fallback | 1, 20 (`Closes #3`) |
| triage of every follow-up | Triage table below; fixes in 15, issues in 16, checklist rows in 17 |
| `docs/architecture/packaging.md` | 18 |

## File map

| Path | Change | Responsibility |
|---|---|---|
| `xtask/hooks/commit-msg.sh` | modify | runs only `cargo xtask check-commits` |
| `xtask/tests/install_hooks.rs` | modify | legacy guard is no longer run |
| `xtask/src/check_doc_paths.rs` | modify | adds `docs/adr/*.md`, checking only "- Code:" lines |
| `xtask/tests/check_doc_paths.rs` | modify | ADR cases |
| `docs/adr/0014-xtask-enforces-rules.md` | modify | present tense for the ADR check |
| `docs/adr/0009-in-process-pipewire-dsp.md` | modify | "Code:" line cites `crates/audio/src/outputs.rs`, which does not exist; becomes `sinks.rs`, `insertion.rs` |
| `crates/core/src/l10n.rs` | create | `msgid`, `forms` markers |
| `crates/core/src/lib.rs` | modify | `pub mod l10n;` |
| `crates/core/src/view/*.rs`, `crates/core/src/companion/alerts.rs`, `crates/core/src/eq/library.rs`, `crates/ui/src/**/*.rs`, `crates/app/src/platform/{background,portal}.rs` | modify | every msgid literal wrapped in `msgid()` / `forms()` |
| `crates/core/src/view/text.rs` | modify | `Text::plural` keeps `[&str; 2]`, callers pass `forms(…)` |
| `crates/core/tests/view/text.rs` | modify | marker test |
| `crates/core/tests/view/catalogue.rs`, `crates/core/tests/view/catalogue/{po_file,debug_msgids}.rs` | create | census of view msgids against `po/pt_BR.po` |
| `crates/core/tests/view/main.rs`, `crates/core/tests/view/headset_page.rs` | modify | `mod catalogue;`, `pub(crate)` `PHASES`/`ROUTES` |
| `po/extract.sh`, `po/check.sh`, `po/LINGUAS`, `po/meson.build`, `po/arctis-companion.pot`, `po/pt_BR.po` | create | app catalogue |
| `meson.build` | modify | `i18n`, `subdir('po')` |
| `data/io.github.marcelogm.ArctisCompanion.desktop` → `.desktop.in` | rename | translated by `i18n.merge_file` |
| `data/io.github.marcelogm.ArctisCompanion.metainfo.xml` → `.metainfo.xml.in` | rename + rewrite | description, known gaps, permission, release, translation |
| `data/meson.build` | modify | `merge_file` and validation of merged outputs |
| `crates/app/tests/data_files.rs` | modify | `SUFFIXES` name the `.desktop.in` and `.metainfo.xml.in` templates |
| `crates/ui/build.rs`, `crates/ui/resources/resources.gresource.xml` | modify | read the `.in` template (Task 4); bundle the `msgfmt --xml` merged metainfo for the About dialog (Task 11A) |
| `crates/ui/tests/widgets/app_dialogs.rs` | modify | the bundled metainfo carries pt-BR |
| `docs/architecture/ui.md` | modify | metainfo path |
| `crates/hid/tests/hardware/{soak_log,soak_log_rules,soak}.rs` | modify | slowest write-to-readback confirmation, printed by the soak (checklist A1) |
| `build-aux/io.github.marcelogm.ArctisCompanion.json` | modify | finish-args |
| `crates/app/tests/packaging.rs` | create | manifest, versions, metainfo, release assets |
| `crates/app/src/locale.rs`, `crates/app/src/lib.rs`, `crates/app/src/application.rs`, `crates/app/Cargo.toml` | create/modify | `setlocale`, `bindtextdomain`, `textdomain` |
| `crates/app/tests/locale.rs` | create | `locale_dir_beside` |
| `extension/po/pt_BR.po` | modify | reviewed wording, glossary below |
| `.github/workflows/ci.yml` | modify | `i18n` job; `gettext` in the `ui` job (`crates/ui/build.rs` runs `msgfmt`) |
| `.github/workflows/release.yml` | create | tag → three assets |
| `crates/core/tests/companion/presets_flow.rs` | modify | SP10 follow-up: publish in the same turn |
| `docs/hardware-checklist.md` | create | hand-run checklist |
| `docs/architecture/packaging.md`, `docs/architecture/README.md` | create/modify | build, i18n, release, install diagrams |

## Triage of the follow-ups

Source: the follow-up list handed over by SP0–SP13. "Fix" = a task here, test first. "Checklist" = a row in
`docs/hardware-checklist.md` (Task 17). "Issue" = opened in Task 16. "None" = already done or informational, with
the evidence.

| # | From | Follow-up (short) | Decision | Where / evidence |
|---|---|---|---|---|
| 1 | SP2 | messages frozen by `crates/core/tests/protocol/messages.rs` | None | informational |
| 2 | SP2 | use `decode_reply(query, bytes)` | None | informational, SP4/SP5 merged |
| 3 | SP2 | SP5 passes `uniq: None` | None | SP5 merged |
| 4 | SP2 | keep `Curve::from_slots` | None | still used by `eq_wire` |
| 5 | SP2 | local llvm-cov env vars | None | developer note; CI enforces coverage |
| 6 | SP2 | `gh pr merge --delete-branch` vs worktrees | None | applied in Task 20 (delete remote branch, then worktree) |
| 7 | SP3 | SP2/SP3 port order | None | both merged |
| 8 | SP3 | `Coefficients::design` off the RT callback | None | `crates/audio/src/designer.rs` designs on the audio thread |
| 9 | SP3 | `response()` at 48 kHz without preamp | None | SP7 merged with `preamp_db_at` |
| 10 | SP3 | APO clamp toast wording covers both cases | None | `crates/core/src/view/preset_files.rs` "{n} values were changed to fit the headset" + "Filter {filter}: {field}" |
| 11 | SP3 | `PresetFormatError`/`EnvelopeError` Display is English | None | no view or widget shows them (grep of `crates/core/src/view`, `crates/ui/src`) |
| 12 | SP13 | merge PR #4 | None | merged |
| 13 | SP13 | devkit visual table | Checklist | row E1–E8 |
| 14 | SP13 | 100 % pt_BR review of the extension | Fix | Task 12 |
| 15 | SP13 | extension zip as a release asset | Fix | Task 10 |
| 16 | SP13 | upload the nightly shell-smoke log | Issue | optional; #17 |
| 17 | SP10 | publish a snapshot synchronously after Save/Delete | Fix (pin) | `crates/core/src/companion/reducer.rs` already publishes at the end of every `handle`; Task 15 pins it with tests |
| 18 | SP10 | import/export a preset file by hand | Checklist | row U3 |
| 19 | SP10 | Rename and Details dialogs not presented in tests | Checklist | row U4 |
| 20 | SP10 | two Duplicates in one main-loop turn share an id | Issue | #18 |
| 21 | SP10 | Gtk-CRITICAL `gtk_widget_get_parent` lines in the a11y test | Issue | #19 |
| 22 | SP11 | `RulePreset::{Known, Missing}` if `RuleRowModel` changes | None | conditional on a future change |
| 23 | SP11 | create the `po/` catalogue | Fix | Tasks 4–6, 11 |
| 24 | SP11 | trivial rebases | None | merged |
| 25 | SP11 | local branch deleted by hand | None | informational |
| 26 | SP12 | fold SP10 test helpers into shared support | Issue | #20 |
| 27 | SP12 | notification default action `app.show-battery` → battery page | None | `crates/app/src/application.rs` `show-battery` → `crates/app/src/runtime.rs::Runtime::show_battery` |
| 28 | SP12 | extract a page-clock helper if more timed pages appear | None | conditional |
| 29 | SP0 | §7 items 1–5 (persistence, events, echoes, readback, `35`) | Checklist | rows H1–H5 |
| 30 | SP0 | §7 items 6–13 | Checklist | rows H6–H13 |
| 31 | SP0 | move captures into `crates/core/tests/golden/` | None | done by SP2 |
| 32 | SP0 | `ENOENT` → PermissionDenied | None | done by SP5 |
| 33 | SP0 | Manager role in context properties, read-back detection, two `pw_stream`s | None | done by SP7 |
| 34 | SP0 | restore `default.configured.audio.sink` | None | `crates/audio/src/keys.rs` (SP7) |
| 35 | SP1 | issue #3 | Fix | Task 1 |
| 36 | SP1 | unknown text types fail closed | None | convention above |
| 37 | SP1 | main needs PR + rebase + `rules` | None | Task 20 |
| 38 | SP1 | hooks run host cargo | None | informational |
| 39 | SP1 | main checkout behind origin | None | out of scope: never edit the main checkout |
| 40 | SP4 | SP5 reuses testkit | None | done |
| 41 | SP4 | resend `Desired` after `Identity` | None | done; `crates/core/tests/end_to_end.rs` lists both `Enforce` |
| 42 | SP4 | `mutants.toml` excludes testkit | None | matches "test code is not mutated" |
| 43 | SP4 | check the first full nightly `cargo mutants` | Fix (verify) | Task 16 dispatches `nightly.yml`; survivors become issue G |
| 44 | SP5 | sidetone write + 30 min soak with the headset on | Checklist | row H14 |
| 45 | SP5 | plug the dongle in with the rule; no permission dialog flash | Checklist | row F3 |
| 46 | SP5 | `check-doc-paths` learns `macro_rules` names | Issue | #21 |
| 47 | SP5 | nothing catches a scenario missing from `scenario_tests!` | Issue | #22 |
| 48 | SP5 | manifest must grant `--device=all` | Fix | Tasks 7–8 |
| 49 | SP6 | `snapshot.presets.active` | None | done |
| 50 | SP6 | `Intent::RetryWrite` | None | done |
| 51 | SP6 | withdraw of a gone notification is a no-op | None | `gio::Application::withdraw_notification` is a no-op for unknown ids |
| 52 | SP6 | `Phase::is_present`/`is_connecting` | None | informational |
| 53 | SP6 | banner 2 instead of 3 after a restart with the headset off (Deviation 36) | Issue | #23 |
| 54 | SP6 | local coverage needs llvm-tools | None | CI enforces |
| 55 | SP7 | transient `DefaultSink`; re-check insert/remove end to end | Checklist | row B2 (needs Bluetooth) |
| 56 | SP7 | combined main CI after the rebase | None | push-to-main runs are green (`gh run list`) |
| 57 | SP7 | ManagerDenied inside the Flatpak | Checklist | row B3 |
| 58 | SP7 | owned pw timer source | None | waits on upstream |
| 59 | SP8 | `auto-power-off` is an enum | None | the extension reads no GSettings (grep of `extension/`) |
| 60 | SP8 | shared `testkit::companion` helpers in `crates/core/tests/companion/support.rs` | Issue | #24 |
| 61 | SP8 | end-to-end lists are exact | None | informational |
| 62 | SP8 | two outstanding Background portal requests | Issue | #25 |
| 63 | SP8 | SP14 owns `setlocale`/`bindtextdomain`; D-Bus error strings English | Fix / None | Task 13; error strings travel on the wire and the extension only logs them (`lib/failures.js`) |
| 64 | SP10 | header buttons are icon-only; re-measure 360 px if labels are added | None | conditional on a future label |
| 65 | SP5 + SP14 §5 | 300 ms readback has no measurement in the soak output | Fix | Task 15A, then checklist row A1 |

Issue A (#16, offline Flatpak sources and Flathub metadata) comes from Deviation 1, not from the list.

## Decisions (pt_BR)

GNOME's Brazilian Portuguese conventions: sentence case for every string, titles included; typographic quotes
“ ”; ellipsis …; numbers rendered by `crates/ui/src/text.rs` (decimal comma). Glossary, applied to the app and the
extension:

| English | pt_BR |
|---|---|
| Headset | headset |
| Dongle / wireless dongle | dongle / dongle sem fio |
| Preset | predefinição |
| App Presets | Predefinições de apps |
| Automatic App Presets | Predefinições automáticas de apps |
| App rule | regra de app |
| Equalizer | equalizador |
| Sidetone | sidetone |
| Mic volume | volume do microfone |
| Mute light | luz de mudo |
| Volume limiter | limitador de volume |
| Extended range | alcance estendido |
| Automatic power off | desligamento automático |
| Battery / low / critical | bateria / fraca / crítica |
| Run in Background | Executar em segundo plano |
| 2.4 GHz | 2,4 GHz |
| Permission Required | Permissão necessária |

## Deviations

| # | Deviation | Why |
|---|---|---|
| 1 | The manifest keeps `build-args: --share=network` and no `cargo-sources.json`; the release ships a GitHub bundle, not a Flathub submission | the spec asks for a bundle on tag; offline vendoring needs `flatpak-cargo-generator` kept in step with `Cargo.lock` for every change. Issue A (#16) tracks it |
| 2 | No root `README.md`; install steps live in `docs/architecture/packaging.md`, the metainfo and the permission dialog | the engineering rules allow only diagram-rich `docs/architecture/*.md` |
| 3 | New root module `crates/core/src/l10n.rs` (`msgid`, `forms`), not in the spec's module tree | `xgettext --language=Rust` matches only bare calls; `Text::plain("…")` is invisible to it (checked with gettext 0.26). A `const fn` identity marker is the Rust `N_()` |
| 4 | `Text::plural` callers pass `forms(singular, plural)` instead of an array literal | `--keyword=forms:1,2` extracts both forms |
| 5 | Desktop entry and metainfo become `.in` templates merged by `i18n.merge_file` | translating them is part of pt_BR 100 % |
| 6 | GSettings summaries are not translated | no UI shows them; `gschema.xml` is not passed to `xgettext` |
| 7 | `ActivatePreset` D-Bus error strings stay English | they are wire text; the extension logs them only |
| 8 | `release.yml` also runs its build jobs on pull requests that touch it, publishing only on a `v*` tag | tests the workflow before the first tag |
| 9 | A separate `i18n` CI job on `fedora:44` | `ubuntu-latest` has gettext 0.21, without Rust support |
| 10 | `check-doc-paths` reads ADRs only on lines that start with `- Code:` | ADR prose cites history and paths on purpose |
| 11 | `--device=all` replaces `--device=dri` | `all` binds the whole `/dev`, DRI included |
| 12 | The preview instance is found with `pgrep -f` and its `/proc/<pid>/exe`, not `pgrep -x arctis-companion` | `arctis-companion` has 16 characters; `pgrep -x` matches the 15-character `comm` and finds nothing |
| 13 | The hardware criteria of SP14 (300 ms readback, ruled app 1 s / 10 s, Bluetooth xruns) are written as checklist rows, not run | they need a person, a real session with a ruled app, and a Bluetooth-paired computer |
| 14 | No screenshots in the metainfo | `appstreamcli validate` passes without them; Flathub is out of scope (issue A) |
| 15 | The census test reads `po/pt_BR.po` with `include_str!` from `crates/core/tests` | dev-only; `arctis-core` keeps its dependency allow-list |
| 16 | `crates/ui/build.rs` runs `msgfmt --xml -d ../../po` and bundles the merged metainfo in the GResource; `msgfmt` (gettext ≥ 0.19.7 with its built-in `metainfo.its`) becomes a build tool of `arctis-ui`, like `blueprint-compiler` | `AdwAboutDialog::from_appdata` shows the metainfo summary and release notes; the English template alone would leave them English under pt_BR |
| 17 | SP14 adds a latency figure to SP5's hardware test support (`crates/hid/tests/hardware/soak_log.rs`) | the acceptance "verified readback within 300 ms" needs a number a person can read; the soak's log had only counts. No hidraw is opened by SP14: only the person runs the ignored test |
| 18 | The installed app is started with `gapplication launch` (D-Bus activation through the exported service file), falling back to `flatpak run` | it is how the app grid starts a `DBusActivatable` app, and it does not depend on the agent shell's `WAYLAND_DISPLAY` |
| 19 | `po/extract.sh` passes `--force-po` | the first `xgettext` call writes no file when it finds no message (before Task 5), and the later `--join-existing` calls then fail |
| 20 | `msgcmp` is the first line of `po/check.sh` to fail on the untranslated catalogue in Task 4, not `msgattrib` | `msgcmp` treats an empty `msgstr` as missing; the failure is still "nothing is translated yet" |
| 21 | The adopted preset name "Headset Curve" (`crates/core/src/companion/catalog.rs::HEADSET_CURVE_NAME`) stays English | it is saved as a custom preset's name in the user's data and shown verbatim; translating it would need gettext in the pure core |
| 22 | The census APO input is built with `collect::<Vec<String>>().concat()` | clippy's `format_collect` rejects collecting `format!` into a `String` |
| 23 | The extension is installed but GNOME Shell lists it only after the next login | on Wayland the Shell does not rescan `~/.local/share/gnome-shell/extensions`; the extension stays not enabled |
| 24 | The `version` step of `release.yml` uses a block scalar, and the workflow was checked with the `actionlint` 1.7.7 release binary | the plain `run:` scalar held `: ` and GitHub rejected the file; Fedora ships no `actionlint` package and the workspace has no YAML parser for a packaging test |

---

## Task 1: Drop the `.githooks` fallback (issue #3)

**Files:**
- Modify: `xtask/tests/install_hooks.rs:119-124`
- Modify: `xtask/hooks/commit-msg.sh`

- [ ] **Step 1: Write the failing test**

Replace `commit_msg_hook_runs_the_legacy_guard_in_checkouts_without_xtask` with:

```rust
#[test]
fn commit_msg_hook_ignores_a_legacy_githooks_guard() {
    let fixture = Fixture::init();
    fixture.write(".githooks/commit-msg", "#!/bin/sh\nexit 7\n");
    fixture.xtask(&["install-hooks"]);
    assert_eq!(run_hook(&fixture, "commit-msg", &["message"]), Some(0));
}
```

- [ ] **Step 2: Run it to see it fail**

Run (toolbox): `cargo test -p xtask --test install_hooks commit_msg_hook_ignores_a_legacy_githooks_guard`
Expected: FAIL, `left: Some(7)`, `right: Some(0)`.

- [ ] **Step 3: Remove the fallback**

`xtask/hooks/commit-msg.sh` becomes:

```sh
#!/bin/sh
[ -d xtask ] || exit 0
exec cargo xtask check-commits --message "$1"
```

- [ ] **Step 4: Run the hook tests**

Run (toolbox): `cargo test -p xtask --test install_hooks`
Expected: PASS, including `commit_msg_hook_passes_checkouts_without_any_guard`.

- [ ] **Step 5: Reinstall the shared hooks and commit**

Run (host): `cargo xtask install-hooks` (writes the shared `.git/hooks`, which is how the fallback leaves every
worktree), then:

```bash
git add xtask/hooks/commit-msg.sh xtask/tests/install_hooks.rs
git commit -m "chore(xtask): drop the .githooks fallback from the commit-msg hook"
git push -u origin sp14-packaging-release
```

## Task 2: ADR "Code:" lines under `check-doc-paths`

**Files:**
- Modify: `xtask/tests/check_doc_paths.rs`
- Modify: `xtask/src/check_doc_paths.rs`
- Modify: `docs/adr/0014-xtask-enforces-rules.md:19`

- [ ] **Step 1: Write the failing tests**

Append to `xtask/tests/check_doc_paths.rs`:

```rust
#[test]
fn adr_code_line_with_a_missing_path_fails() {
    let outcome = outcome_for(
        "docs/adr/0001-example.md",
        "# Example\n\n- Code: `crates/core/src/gone.rs`.\n",
    );
    assert!(!outcome.succeeded());
    assert!(
        outcome.stderr.contains("docs/adr/0001-example.md:3"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn adr_code_line_with_an_existing_symbol_passes() {
    let outcome = outcome_for(
        "docs/adr/0001-example.md",
        "- Code: `crates/core/src/time.rs::Millis`.\n",
    );
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn adr_prose_is_not_checked() {
    let outcome = outcome_for(
        "docs/adr/0001-example.md",
        "Superseded `scripts/check-line-limit.sh`.\n",
    );
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn review_log_is_not_an_adr() {
    let outcome = outcome_for(
        "docs/adr/review-log.md",
        "- Code: `crates/core/src/gone.rs`\n",
    );
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}
```

- [ ] **Step 2: Run them to see the first two fail**

Run (toolbox): `cargo test -p xtask --test check_doc_paths adr_`
Expected: `adr_code_line_with_a_missing_path_fails` FAILS (ADRs are not read); the others pass.

- [ ] **Step 3: Read ADRs, keeping only "- Code:" lines**

In `xtask/src/check_doc_paths.rs`, replace `documents` and `check_document` and add the helpers below
(`Scope` decides which references count):

```rust
const CODE_LINE: &str = "- Code:";

#[derive(Clone, Copy)]
enum Scope {
    Everything,
    CodeLines,
}

pub fn run(root: &Path) -> Report {
    documents(root)
        .iter()
        .map(|(document, scope)| check_document(root, document, *scope))
        .fold(Report::default(), Report::merged)
}

fn documents(root: &Path) -> Vec<(PathBuf, Scope)> {
    let mut documents: Vec<(PathBuf, Scope)> = markdown_under(&root.join("docs/architecture"))
        .into_iter()
        .map(|path| (path, Scope::Everything))
        .collect();
    documents.push((root.join("CONTEXT.md"), Scope::Everything));
    documents.extend(adrs(root).into_iter().map(|path| (path, Scope::CodeLines)));
    documents.retain(|(document, _)| document.is_file());
    documents
}

fn adrs(root: &Path) -> Vec<PathBuf> {
    markdown_under(&root.join("docs/adr"))
        .into_iter()
        .filter(|path| is_numbered(path))
        .collect()
}

fn is_numbered(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(|first: char| first.is_ascii_digit()))
}

fn check_document(root: &Path, document: &Path, scope: Scope) -> Report {
    let label = document
        .strip_prefix(root)
        .unwrap_or(document)
        .display()
        .to_string();
    let Ok(text) = fs::read_to_string(document) else {
        return Report::failed(format!("{label}: unreadable"));
    };
    let lines: Vec<&str> = text.lines().collect();
    let problems = doc_refs(&text)
        .into_iter()
        .filter(|doc_ref| in_scope(scope, &lines, doc_ref.line))
        .filter_map(|doc_ref| {
            problem(root, &doc_ref).map(|problem| format!("{label}:{}: {problem}", doc_ref.line))
        });
    Report::from_errors(problems)
}

fn in_scope(scope: Scope, lines: &[&str], line: usize) -> bool {
    match scope {
        Scope::Everything => true,
        Scope::CodeLines => line
            .checked_sub(1)
            .and_then(|index| lines.get(index))
            .is_some_and(|text| text.trim_start().starts_with(CODE_LINE)),
    }
}
```

`DocRef::line` is 1-based (the existing `missing_path_fails_with_its_line` test asserts `core.md:2` for the
second line); keep that assumption and let the new test confirm it.

- [ ] **Step 4: Run the xtask tests and the repository check**

Run (toolbox): `cargo test -p xtask --test check_doc_paths && cargo xtask check-doc-paths`
Expected: the tests PASS; the repository check FAILS exactly once, on
`docs/adr/0009-in-process-pipewire-dsp.md:21: \`crates/audio/src/outputs.rs\` does not exist` (checked on
`origin/main`: every other ADR "Code:" path exists). In that line replace `` `crates/audio/src/outputs.rs` `` with
`` `crates/audio/src/sinks.rs`, `crates/audio/src/insertion.rs` `` (sink classification and EQ insertion live there),
then rerun: PASS.

- [ ] **Step 5: Update ADR 0014 and commit**

`docs/adr/0014-xtask-enforces-rules.md` line 19 already says the ADR "Code:" lines are checked; delete its last
sentence, "ADR "Code:" lines join the check in SP14, once every cited path exists.".

```bash
git add xtask/src/check_doc_paths.rs xtask/tests/check_doc_paths.rs docs/adr/0014-xtask-enforces-rules.md docs/adr/0009-in-process-pipewire-dsp.md
git commit -m "feat(xtask): check the Code lines of ADRs"
git push
```

## Task 3: The `msgid` and `forms` markers

**Files:**
- Create: `crates/core/src/l10n.rs`
- Modify: `crates/core/src/lib.rs`
- Modify: `crates/core/tests/view/text.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/core/tests/view/text.rs` (add `use arctis_core::l10n::{forms, msgid};`):

```rust
#[test]
fn markers_return_their_english_text() {
    assert_eq!(msgid("Low"), "Low");
    assert_eq!(
        forms("{n} minute", "{n} minutes"),
        ["{n} minute", "{n} minutes"]
    );
}

#[test]
fn markers_work_in_constants() {
    const MARKS: [(u8, &str); 1] = [(0, msgid("Off"))];
    assert_eq!(MARKS[0].1, "Off");
}
```

- [ ] **Step 2: Run it to see it fail**

Run (toolbox): `cargo test -p arctis-core --test view markers_`
Expected: FAIL to compile, "unresolved import `arctis_core::l10n`".

- [ ] **Step 3: Add the module**

`crates/core/src/l10n.rs`:

```rust
#[must_use]
pub const fn msgid(english: &'static str) -> &'static str {
    english
}

#[must_use]
pub const fn forms(singular: &'static str, plural: &'static str) -> [&'static str; 2] {
    [singular, plural]
}
```

Add `pub mod l10n;` to `crates/core/src/lib.rs` next to `pub mod intent;`.

- [ ] **Step 4: Run it to see it pass**

Run (toolbox): `cargo test -p arctis-core --test view markers_ && cargo xtask check-pure`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/l10n.rs crates/core/src/lib.rs crates/core/tests/view/text.rs
git commit -m "feat(core): mark msgids for xgettext"
git push
```

## Task 4: The `po/` skeleton, extraction and check scripts

**Files:**
- Create: `po/extract.sh`, `po/check.sh`, `po/LINGUAS`, `po/meson.build`
- Create (generated): `po/arctis-companion.pot`, `po/pt_BR.po`
- Modify: `meson.build`, `data/meson.build`, `crates/app/tests/data_files.rs`, `crates/ui/build.rs`,
  `crates/ui/resources/resources.gresource.xml`, `docs/architecture/README.md`, `docs/architecture/ui.md`

- [ ] **Step 1: Write the checks first**

In `crates/app/tests/data_files.rs` change `SUFFIXES` to
`[".desktop.in", ".metainfo.xml.in", ".gschema.xml", ".service.in"]`. Run (ui):
`cargo test -p arctis-app --test data_files`. Expected: FAIL naming
`data/io.github.marcelogm.ArctisCompanion.desktop.in`.

`po/check.sh` (same shape as `extension/po/check.sh`):

```sh
#!/bin/sh
set -eu
cd "$(dirname "$0")"
fresh=$(mktemp -d)
sh extract.sh "$fresh/fresh.pot"
grep -v POT-Creation-Date arctis-companion.pot > "$fresh/tracked.body"
grep -v POT-Creation-Date "$fresh/fresh.pot" > "$fresh/fresh.body"
diff -u "$fresh/tracked.body" "$fresh/fresh.body"
grep -qx pt_BR LINGUAS
msgfmt --check --output-file=/dev/null pt_BR.po
msgcmp pt_BR.po arctis-companion.pot
test -z "$(msgattrib --untranslated --no-obsolete pt_BR.po)"
test -z "$(msgattrib --only-fuzzy --no-obsolete pt_BR.po)"
echo "catalogue up to date"
```

- [ ] **Step 2: Run it to see it fail**

Run (toolbox): `sh po/check.sh`
Expected: FAIL, `extract.sh: No such file or directory`.

- [ ] **Step 3: Write the extractor**

`po/extract.sh`:

```sh
#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
output="${1:-po/arctis-companion.pot}"
lists=$(mktemp -d)
common="--from-code=UTF-8 --add-location=file --sort-by-file --package-name=arctis-companion"
find crates -path '*/src/*' -name '*.rs' -not -path '*/testkit/*' | LC_ALL=C sort > "$lists/rust"
find crates/ui/resources -name '*.blp' | LC_ALL=C sort > "$lists/blueprint"
xgettext $common --language=Rust --keyword=msgid --keyword=forms:1,2 \
    --files-from="$lists/rust" --output="$output"
xgettext $common --join-existing --language=C --keyword=_ --keyword=C_:1c,2 \
    --files-from="$lists/blueprint" --output="$output"
xgettext $common --join-existing --language=Desktop \
    --output="$output" data/io.github.marcelogm.ArctisCompanion.desktop.in
xgettext $common --join-existing \
    --output="$output" data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in
rm -r "$lists"
```

Turn the desktop entry and the metainfo into templates (content unchanged here; Task 9 rewrites the metainfo):

```bash
git mv data/io.github.marcelogm.ArctisCompanion.desktop data/io.github.marcelogm.ArctisCompanion.desktop.in
git mv data/io.github.marcelogm.ArctisCompanion.metainfo.xml data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in
```

In `data/meson.build` replace the two `install_data` calls for the desktop entry and the metainfo, and the two
validation tests, with:

```meson
desktop_file = i18n.merge_file(
  input: app_id + '.desktop.in',
  output: app_id + '.desktop',
  type: 'desktop',
  po_dir: meson.project_source_root() / 'po',
  install: true,
  install_dir: datadir / 'applications',
)

metainfo_file = i18n.merge_file(
  input: app_id + '.metainfo.xml.in',
  output: app_id + '.metainfo.xml',
  po_dir: meson.project_source_root() / 'po',
  install: true,
  install_dir: datadir / 'metainfo',
)

desktop_file_validate = find_program('desktop-file-validate', required: false)
if desktop_file_validate.found()
  test('validate-desktop', desktop_file_validate, args: [desktop_file])
endif

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('validate-metainfo', appstreamcli, args: ['validate', '--no-net', '--explain', metainfo_file])
endif
```

The rename breaks two readers of the old names on `origin/main`; point them at the template:

- `crates/ui/build.rs`: `const METAINFO: &str = "../../data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in";`
- `crates/ui/resources/resources.gresource.xml`: the entry becomes
  `<file alias="metainfo.xml">io.github.marcelogm.ArctisCompanion.metainfo.xml.in</file>` (Task 11A swaps it for
  the merged file).

Update the metainfo and desktop paths in `docs/architecture/README.md` and `docs/architecture/ui.md` (line 40) to
the `.in` names so `check-doc-paths` stays green.

Run (ui): `cargo test -p arctis-app --test data_files && cargo test -p arctis-ui --test widgets about_comes_from_the_metainfo`
Expected: PASS.

- [ ] **Step 4: Catalogue files and meson**

`po/LINGUAS`:

```
pt_BR
```

`po/meson.build`:

```meson
i18n.gettext(gettext_package, preset: 'glib')

msgfmt = find_program('msgfmt')
test('check-pt_BR', msgfmt, args: ['--check', '--output-file=/dev/null', files('pt_BR.po')])
```

In `meson.build`, after `gnome = import('gnome')`:

```meson
i18n = import('i18n')
gettext_package = meson.project_name()
```

and `subdir('po')` before `subdir('data')`.

Generate the template and the empty pt_BR file:

Run (toolbox): `sh po/extract.sh && msginit --no-translator --locale=pt_BR.UTF-8 --input=po/arctis-companion.pot --output-file=po/pt_BR.po`

Confirm that the Blueprint and data strings are in: `grep -c '^msgid' po/arctis-companion.pot` is above 50 (Blueprint and data only at this point) and
`grep -n 'Permission Required\|Control your SteelSeries' po/arctis-companion.pot` prints both.

- [ ] **Step 5: Run the check to see the remaining, expected failure**

Run (toolbox): `sh po/check.sh`
Expected: FAIL only at the `msgattrib --untranslated` line (nothing is translated yet). Run (toolbox):
`meson setup /tmp/arctis-sp14-meson --wipe && meson test -C /tmp/arctis-sp14-meson check-pt_BR validate-schema validate-desktop validate-metainfo`
(`meson test` builds first, cargo release build included, a few minutes).
Expected: PASS (an untranslated catalogue is still valid, and the merged files equal the English ones).

- [ ] **Step 6: Commit**

```bash
git add po meson.build data docs/architecture/README.md docs/architecture/ui.md crates/app/tests/data_files.rs crates/ui/build.rs crates/ui/resources/resources.gresource.xml
git commit -m "build(i18n): add the po catalogue with extract and check scripts"
git push
```

## Task 5: Mark every msgid in the Rust sources

**Files (each English msgid literal):**
- Modify: `crates/core/src/view/{app_presets,battery_chart,eq_editor,equalizer_rows,headset_page,headset_rows,preferences,preset_edits,preset_files,presets,status_page,text}.rs`
- Modify: `crates/core/src/companion/alerts.rs`, `crates/core/src/eq/library.rs`
- Modify: `crates/ui/src/{services,toasts,preset_files}.rs`, `crates/ui/src/eq_editor_page/header.rs`
- Modify: `crates/app/src/platform/{background,portal}.rs`

The rule: every English literal that reaches `gettext` is written `msgid("…")`; every singular/plural pair is
`forms("…", "…")`. Dynamic arguments (`preset.name()`, identifiers) stay as they are. Examples of each shape:

```rust
use crate::l10n::{forms, msgid};

Text::plain(msgid("Preset missing"))

Text::format(
    msgid("{preset}, chosen for {app}"),
    vec![name, ("app", Arg::Str(app.clone()))],
)

Text::plural(forms("{n} minute", "{n} minutes"), minutes, Vec::new())

pub const SIDETONE_MARKS: [(u8, &str); 4] = [
    (0, msgid("Off")),
    (3, msgid("Low")),
    (6, msgid("Medium")),
    (10, msgid("High")),
];

MuteLight::Off => msgid("Off"),

Arg::Msgid(msgid("Unknown").to_owned())

StatusPageModel::new(
    "applications-multimedia-symbolic",
    msgid("No App Presets"),
    msgid("Choose a preset that applies while an app plays sound"),
    Some(msgid("Add Rule")),
)

built_in("bass-boost", msgid("Bass Boost"), BASS_BOOST),

const REASON: &str =
    msgid("Keep battery warnings, battery history and app presets working with the window closed");
```

`xgettext` 0.26 extracts `msgid("…")` inside macro bodies (`vec![…]`, `assert_eq!(…)`) and in `const` items
(checked in the toolbox), so no literal has to move out of a macro.

In `crates/ui` and `crates/app` the import is `use arctis_core::l10n::msgid;`.

- [ ] **Step 1: Baseline**

Run (toolbox): `cargo test -p arctis-core && sh po/extract.sh && grep -c '^msgid' po/arctis-companion.pot`
Expected: core tests PASS; note the msgid count (Blueprint and data strings only).

- [ ] **Step 2: Wrap the literals, one file at a time**

For each file above, wrap every msgid literal as shown. Run (toolbox) `cargo test -p arctis-core --test view`
after each core file; the English rendering (`testkit::english`) must not change, so every existing view test
stays green.

- [ ] **Step 3: Review what extraction still misses**

Run (toolbox):

```bash
sh po/extract.sh
find crates -path '*/src/*' -name '*.rs' -not -path '*/testkit/*' | LC_ALL=C sort > /tmp/arctis-rust-files
xgettext --from-code=UTF-8 --language=Rust --extract-all --no-location --files-from=/tmp/arctis-rust-files --output=/tmp/arctis-all.pot
msgcomm --unique --no-location /tmp/arctis-all.pot po/arctis-companion.pot | grep '^msgid'
```

Expected: only non-messages remain (icon names ending `-symbolic`, ids such as `"flat"`, GSettings keys, D-Bus
names, file extensions, `format!` templates, log and panic text, `ActivatePreset` wire errors). Any English
sentence or label shown to a user in that list gets `msgid()`; repeat until none is left.

- [ ] **Step 4: Full test run**

Run (toolbox): `cargo clippy --workspace --all-targets -- -D warnings && cargo test -p arctis-core`, then
Run (ui): `cargo test -p arctis-ui -p arctis-app`.
Expected: PASS.

- [ ] **Step 5: Regenerate the template and commit**

Run (toolbox): `sh po/extract.sh && msgmerge --update --backup=none po/pt_BR.po po/arctis-companion.pot`

```bash
git add crates po
git commit -m "refactor(i18n): mark every msgid for extraction"
git push
```

## Task 6: Census test of view msgids against `pt_BR`

**Files:**
- Create: `crates/core/tests/view/catalogue.rs`
- Create: `crates/core/tests/view/catalogue/po_file.rs`
- Create: `crates/core/tests/view/catalogue/debug_msgids.rs`
- Modify: `crates/core/tests/view/main.rs` (`mod catalogue;`)
- Modify: `crates/core/tests/view/headset_page.rs` (`pub(crate) const PHASES`, `pub(crate) const ROUTES`)

- [ ] **Step 1: Write the helper tests first**

`crates/core/tests/view/catalogue/debug_msgids.rs`:

```rust
const MARKERS: [&str; 5] = [
    "Plain(\"",
    "msgid: \"",
    "singular: \"",
    "plural: \"",
    "Msgid(\"",
];

pub fn msgids_in(dump: &str) -> Vec<String> {
    MARKERS
        .iter()
        .flat_map(|marker| {
            dump.match_indices(marker)
                .map(move |(at, _)| debug_string(&dump[at + marker.len()..]))
        })
        .collect()
}

fn debug_string(rest: &str) -> String {
    let mut text = String::new();
    let mut chars = rest.chars();
    while let Some(next) = chars.next() {
        match next {
            '"' => break,
            '\\' => text.push(unescaped(chars.next())),
            other => text.push(other),
        }
    }
    text
}

const fn unescaped(escaped: Option<char>) -> char {
    match escaped {
        Some('n') => '\n',
        Some('t') => '\t',
        Some(other) => other,
        None => '\\',
    }
}
```

`crates/core/tests/view/catalogue/po_file.rs`:

```rust
use std::collections::HashMap;

pub type Group = (Vec<String>, Vec<String>);

pub struct Catalogue {
    translations: HashMap<String, Vec<String>>,
    groups: Vec<Group>,
}

impl Catalogue {
    pub fn parse(text: &str) -> Self {
        let entries: Vec<Entry> = text
            .split("\n\n")
            .map(Entry::parse)
            .filter(|entry| !entry.msgids.is_empty() && !entry.fuzzy)
            .collect();
        let groups = entries
            .iter()
            .map(|entry| (entry.msgids.clone(), entry.msgstrs.clone()))
            .collect();
        let translations = entries.into_iter().flat_map(Entry::keyed).collect();
        Self {
            translations,
            groups,
        }
    }

    pub fn is_translated(&self, msgid: &str) -> bool {
        self.translations.get(msgid).is_some_and(|forms| all_filled(forms))
    }

    pub fn translated_groups(&self) -> impl Iterator<Item = &Group> {
        self.groups.iter().filter(|(_, forms)| all_filled(forms))
    }
}

fn all_filled(forms: &[String]) -> bool {
    !forms.is_empty() && forms.iter().all(|form| !form.is_empty())
}

#[derive(Default)]
struct Entry {
    msgids: Vec<String>,
    msgstrs: Vec<String>,
    fuzzy: bool,
}

impl Entry {
    fn parse(block: &str) -> Self {
        let mut entry = Self::default();
        let mut target: Option<bool> = None;
        for line in block.lines() {
            target = entry.take(line, target);
        }
        entry
    }

    fn take(&mut self, line: &str, target: Option<bool>) -> Option<bool> {
        if line.starts_with("#,") {
            self.fuzzy |= line.contains("fuzzy");
            return target;
        }
        if let Some(rest) = line.strip_prefix('"') {
            self.append(target, &unquoted(rest));
            return target;
        }
        let (keyword, value) = line.split_once(' ')?;
        if keyword == "msgctxt" {
            return None;
        }
        let is_msgstr = keyword.starts_with("msgstr");
        let field = if is_msgstr { &mut self.msgstrs } else { &mut self.msgids };
        keyword.starts_with("msg").then(|| {
            field.push(unquoted(value.trim_start_matches('"')));
            is_msgstr
        })
    }

    fn append(&mut self, target: Option<bool>, text: &str) {
        let field = match target {
            Some(true) => &mut self.msgstrs,
            Some(false) => &mut self.msgids,
            None => return,
        };
        if let Some(last) = field.last_mut() {
            last.push_str(text);
        }
    }

    fn keyed(self) -> Vec<(String, Vec<String>)> {
        self.msgids
            .into_iter()
            .map(|msgid| (msgid, self.msgstrs.clone()))
            .collect()
    }
}

fn unquoted(quoted: &str) -> String {
    let body = quoted.strip_suffix('"').unwrap_or(quoted);
    body.replace("\\n", "\n").replace("\\\"", "\"").replace("\\\\", "\\")
}
```

`Entry::parse` treats the header (`msgid ""`) as an entry with an empty key; it is never looked up.

`crates/core/tests/view/catalogue.rs`, first only the helper tests:

```rust
mod debug_msgids;
mod po_file;

use arctis_core::view::text::{Arg, Text};

use debug_msgids::msgids_in;
use po_file::Catalogue;

const SAMPLE_PO: &str = "msgid \"\"\nmsgstr \"\"\n\"Language: pt_BR\\n\"\n\n\
msgid \"Low\"\nmsgstr \"Baixo\"\n\n\
#, fuzzy\nmsgid \"High\"\nmsgstr \"Alto\"\n\n\
msgid \"{n} minute\"\nmsgid_plural \"{n} minutes\"\nmsgstr[0] \"{n} minuto\"\nmsgstr[1] \"{n} minutos\"\n\n\
msgid \"\"\n\"Long \"\n\"line\"\nmsgstr \"\"\n\"Linha \"\n\"longa\"\n\n\
msgid \"Missing\"\nmsgstr \"\"\n";

#[test]
fn debug_dumps_yield_every_msgid_shape() {
    let texts = vec![
        Text::plain("A \"quoted\" word"),
        Text::format("{x} left", vec![("x", Arg::Msgid("Flat".to_owned()))]),
        Text::plural(["{n} rule", "{n} rules"], 2, Vec::new()),
        Text::verbatim("Firefox"),
    ];
    let mut found = msgids_in(&format!("{texts:?}"));
    found.sort();
    assert_eq!(
        found,
        ["A \"quoted\" word", "Flat", "{n} rule", "{n} rules", "{x} left"]
    );
}

#[test]
fn the_po_reader_knows_translated_entries() {
    let catalogue = Catalogue::parse(SAMPLE_PO);
    assert!(catalogue.is_translated("Low"));
    assert!(catalogue.is_translated("{n} minute"));
    assert!(catalogue.is_translated("{n} minutes"));
    assert!(catalogue.is_translated("Long line"));
    assert!(!catalogue.is_translated("High"));
    assert!(!catalogue.is_translated("Missing"));
    assert!(!catalogue.is_translated("Absent"));
}
```

- [ ] **Step 2: Run the helper tests**

Run (toolbox): `cargo test -p arctis-core --test view catalogue`
Expected: FAIL to compile until `mod catalogue;` is added to `crates/core/tests/view/main.rs`; then PASS. If
`the_po_reader_knows_translated_entries` fails, fix `po_file.rs` (keep functions within the clippy limits).

- [ ] **Step 3: Write the census tests**

Make `PHASES` and `ROUTES` in `crates/core/tests/view/headset_page.rs` `pub(crate)`. Append to
`crates/core/tests/view/catalogue.rs`:

```rust
use std::collections::BTreeSet;
use std::fmt::Debug;

use arctis_core::battery::sample::{Reading, UnixSeconds};
use arctis_core::battery::series::Window;
use arctis_core::companion::alerts::Alerts;
use arctis_core::eq::library::library;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::values::AutoPowerOff;
use arctis_core::view::app_presets::{add_rule_dialog, app_presets};
use arctis_core::view::battery_chart::battery_page;
use arctis_core::view::eq_editor::{eq_editor_for, kind_items};
use arctis_core::view::equalizer_rows::equalizer_group;
use arctis_core::view::headset_page::{banner, headset_page};
use arctis_core::view::headset_rows::{auto_power_off_text, sidetone_text};
use arctis_core::view::preferences::{Background, preferences};
use arctis_core::view::preset_edits::{copy_name, saved_toast};
use arctis_core::view::preset_files::{export_failed, import_preset};
use arctis_core::view::presets::{deleted_toast, presets_page};
use arctis_core::view::text::preset_title;

use crate::headset_page::{PHASES, ROUTES};
use crate::support::{percent, rules, snapshot, with};

const PT_BR: &str = include_str!("../../../../po/pt_BR.po");

fn dump(value: &impl Debug) -> String {
    format!("{value:?}")
}

fn headset_dumps() -> Vec<String> {
    PHASES
        .iter()
        .flat_map(|phase| ROUTES.iter().map(move |route| with(*phase, *route)))
        .flat_map(|s| [dump(&headset_page(&s)), dump(&banner(&s)), dump(&equalizer_group(&s))])
        .collect()
}

fn page_dumps() -> Vec<String> {
    let s = snapshot();
    let mut ruled = snapshot();
    ruled.presets.rules = rules(&[("firefox", "flat"), ("steam", "gone")]);
    vec![
        dump(&presets_page(&s)),
        dump(&app_presets(&s)),
        dump(&app_presets(&ruled)),
        dump(&add_rule_dialog(&s)),
        dump(&battery_page(&s, Window::Day, UnixSeconds(0))),
        dump(&battery_page(&s, Window::Week, UnixSeconds(0))),
        dump(&preferences(&s, Background::Off)),
        dump(&preferences(&s, Background::On)),
        dump(&preferences(&s, Background::Denied)),
    ]
}

fn preset_dumps() -> Vec<String> {
    let mut ruled = snapshot();
    ruled.presets.rules = rules(&[("firefox", "flat"), ("steam", "flat")]);
    library()
        .iter()
        .flat_map(|preset| {
            [
                dump(&preset_title(preset)),
                dump(&eq_editor_for(&ruled, preset)),
                dump(&deleted_toast(&ruled, preset)),
                dump(&saved_toast(preset)),
                dump(&copy_name(preset)),
                dump(&export_failed(preset)),
            ]
        })
        .collect()
}

fn value_dumps() -> Vec<String> {
    let mut dumps: Vec<String> = (0..=10).map(|value| dump(&sidetone_text(value))).collect();
    dumps.extend(AutoPowerOff::CHOICES.iter().map(|choice| dump(&auto_power_off_text(*choice))));
    dumps.push(dump(&kind_items()));
    dumps
}

fn file_dumps() -> Vec<String> {
    let s = snapshot();
    let eleven: String = (1..=11)
        .map(|n| format!("Filter {n}: ON PK Fc 100 Hz Gain 1 dB Q 1\n"))
        .collect();
    let inputs = [
        "nonsense\n",
        "Filter 1: ON NO Fc 100 Hz\n",
        eleven.as_str(),
        "Filter 1: ON PK Fc 100 Hz Gain 30 dB Q 1\n",
    ];
    inputs
        .iter()
        .map(|text| dump(&import_preset(&s, "file.txt", text)))
        .collect()
}

fn alert_dumps() -> Vec<String> {
    let mut alerts = Alerts::default();
    [15, 3]
        .iter()
        .map(|value| dump(&alerts.update(discharging(*value), percent(15))))
        .collect()
}

fn discharging(value: u8) -> Reading {
    Reading {
        percent: percent(value),
        charging: Charging::Discharging,
        link: Link::Connected,
    }
}

fn census() -> BTreeSet<String> {
    [headset_dumps(), page_dumps(), preset_dumps(), value_dumps(), file_dumps(), alert_dumps()]
        .concat()
        .iter()
        .flat_map(|text| msgids_in(text))
        .collect()
}

#[test]
fn the_census_reaches_every_family_of_views() {
    let census = census();
    for msgid in [
        "Permission Required",
        "No App Presets",
        "Bass Boost",
        "Off",
        "Headset Battery Low",
    ] {
        assert!(census.contains(msgid), "{msgid} not reached: {census:#?}");
    }
}

#[test]
fn every_view_message_is_translated_to_pt_br() {
    let catalogue = Catalogue::parse(PT_BR);
    let missing: Vec<String> = census()
        .into_iter()
        .filter(|msgid| !catalogue.is_translated(msgid))
        .collect();
    assert!(missing.is_empty(), "untranslated: {missing:#?}");
}

#[test]
fn translations_keep_every_placeholder() {
    let catalogue = Catalogue::parse(PT_BR);
    let broken: Vec<&Vec<String>> = catalogue
        .translated_groups()
        .filter(|(msgids, forms)| changes_placeholders(msgids, forms))
        .map(|(msgids, _)| msgids)
        .collect();
    assert!(broken.is_empty(), "placeholders changed: {broken:#?}");
}

fn changes_placeholders(msgids: &[String], forms: &[String]) -> bool {
    let expected: BTreeSet<&str> = msgids.iter().flat_map(|msgid| placeholders(msgid)).collect();
    forms.iter().any(|form| placeholders(form) != expected)
}

fn placeholders(text: &str) -> BTreeSet<&str> {
    text.split('{')
        .skip(1)
        .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
        .collect()
}
```

Notes for the implementer: names and paths above match `origin/main` after SP8; if one moved, follow the compiler
to the current path (the module list is in `crates/core/src/view/mod.rs`). Untranslated entries are skipped by
`translated_groups` and reported by `po/check.sh` instead. Each pt_BR form must carry the union of the placeholders
of `msgid` and `msgid_plural`, so keep `{n}` in both pt_BR plural forms.

- [ ] **Step 4: Run the census to see it fail**

Run (toolbox): `cargo test -p arctis-core --test view catalogue`
Expected: `the_census_reaches_every_family_of_views` PASS; `every_view_message_is_translated_to_pt_br` FAIL listing the
msgids (pt_BR is empty). Any listed msgid that is absent from `po/arctis-companion.pot` is an unmarked literal:
go back to Task 5 Step 2 for it.

- [ ] **Step 5: Commit the red census**

The commit hook runs only `check-rules`, not tests, so a red test may be committed on the branch.

```bash
git add crates/core/tests/view
git commit -m "test(core): census view msgids against the pt_BR catalogue"
git push
```

## Task 7: Packaging tests

**Files:**
- Create: `crates/app/tests/packaging.rs`

- [ ] **Step 1: Write the failing tests**

```rust
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const MANIFEST: &str = "build-aux/io.github.marcelogm.ArctisCompanion.json";
const METAINFO: &str = "data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in";
const RELEASE: &str = ".github/workflows/release.yml";

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn meson_version() -> String {
    let meson = repo_file("meson.build");
    let marker = "\n  version: '";
    let start = meson.find(marker).expect("project version") + marker.len();
    meson[start..].split('\'').next().expect("closing quote").to_owned()
}

#[test]
fn the_sandbox_grants_hidraw_pipewire_and_a_display() {
    let manifest: serde_json::Value =
        serde_json::from_str(&repo_file(MANIFEST)).expect("manifest is JSON");
    let granted: BTreeSet<&str> = manifest["finish-args"]
        .as_array()
        .expect("finish-args")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    let expected = BTreeSet::from([
        "--share=ipc",
        "--socket=wayland",
        "--socket=fallback-x11",
        "--device=all",
        "--filesystem=xdg-run/pipewire-0",
    ]);
    assert_eq!(granted, expected);
}

#[test]
fn cargo_and_the_metainfo_carry_the_meson_version() {
    let version = meson_version();
    assert!(repo_file("Cargo.toml").contains(&format!("\nversion = \"{version}\"")));
    assert!(repo_file(METAINFO).contains(&format!("<release version=\"{version}\"")));
}

#[test]
fn the_metainfo_names_the_known_gaps() {
    let metainfo = repo_file(METAINFO);
    for gap in [
        "Switching the headset between 2.4 GHz and Bluetooth audio",
        "The Bluetooth equalizer preset for audio from a phone",
    ] {
        assert!(metainfo.contains(gap), "{gap}");
    }
}

#[test]
fn the_metainfo_explains_the_device_permission() {
    let metainfo = repo_file(METAINFO);
    assert!(metainfo.contains("access to all devices"));
    assert!(metainfo.contains("70-arctis-companion.rules"));
    assert!(metainfo.contains("<translation type=\"gettext\">arctis-companion</translation>"));
}

#[test]
fn a_tag_publishes_the_bundle_the_extension_and_the_rule() {
    let workflow = repo_file(RELEASE);
    assert!(workflow.contains("tags: [\"v*\"]"));
    for asset in [
        "arctis-companion.flatpak",
        "arctis-companion@marcelogm.github.io.shell-extension.zip",
        "data/udev/70-arctis-companion.rules",
    ] {
        assert!(workflow.contains(asset), "{asset}");
    }
}
```

- [ ] **Step 2: Run them to see them fail**

Run (toolbox): `cargo test -p arctis-app --test packaging`
Expected: FAIL: `the_sandbox_grants_…` (finish-args differ), `the_metainfo_…` (texts absent), `a_tag_…`
(`release.yml` missing); `cargo_and_the_metainfo_carry_the_meson_version` may already PASS (0.1.0 everywhere).

- [ ] **Step 3: Commit**

```bash
git add crates/app/tests/packaging.rs
git commit -m "test(app): pin the sandbox, versions, metainfo and release assets"
git push
```

## Task 8: Final finish-args

**Files:**
- Modify: `build-aux/io.github.marcelogm.ArctisCompanion.json`

- [ ] **Step 1: Replace `finish-args`**

```json
  "finish-args": [
    "--share=ipc",
    "--socket=wayland",
    "--socket=fallback-x11",
    "--device=all",
    "--filesystem=xdg-run/pipewire-0"
  ],
```

`--device=all` because hidraw has no narrower Flatpak device class (`docs/research/flatpak-pipewire.md`, SP0 §6.1);
`xdg-run/pipewire-0` because Flatpak 1.18 does not mount the PipeWire socket by default. Notifications, the
Background portal and the file chooser go through portals; the app owns its own app-id name on the session bus.

- [ ] **Step 2: Run the test**

Run (toolbox): `cargo test -p arctis-app --test packaging the_sandbox`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add build-aux/io.github.marcelogm.ArctisCompanion.json
git commit -m "build(flatpak): grant hidraw and the PipeWire socket"
git push
```

## Task 9: Translated desktop entry and metainfo

**Files:**
- Modify: `data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in`
- Modify: `data/io.github.marcelogm.ArctisCompanion.desktop.in`
- Modify: `data/meson.build`

- [ ] **Step 1: Rewrite the metainfo**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>io.github.marcelogm.ArctisCompanion</id>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>GPL-3.0-or-later</project_license>
  <name>Arctis Companion</name>
  <summary>Control your SteelSeries Arctis Nova 5 headset</summary>
  <description>
    <p>Change the headset’s settings and equalizer, let presets follow the app that plays sound, and keep an eye on the battery.</p>
    <ul>
      <li>Sidetone, microphone volume, mute light, volume limiter, wireless mode and automatic power off</li>
      <li>Built-in and custom equalizer presets with a parametric editor and Equalizer APO import</li>
      <li>App presets that switch the equalizer while a chosen app plays</li>
      <li>Battery history and low battery notifications</li>
      <li>A Quick Settings toggle through the companion GNOME Shell extension</li>
    </ul>
    <p>The wireless dongle is reached through hidraw, so the app needs access to all devices and a udev rule on the computer, 70-arctis-companion.rules. When permission is missing, the app shows the rule and the command to load it.</p>
    <p>Not supported yet:</p>
    <ul>
      <li>Switching the headset between 2.4 GHz and Bluetooth audio</li>
      <li>The Bluetooth equalizer preset for audio from a phone</li>
    </ul>
  </description>
  <developer id="io.github.marcelogm">
    <name translate="no">marcelogm</name>
  </developer>
  <launchable type="desktop-id">io.github.marcelogm.ArctisCompanion.desktop</launchable>
  <translation type="gettext">arctis-companion</translation>
  <url type="homepage">https://github.com/marcelogm/arctis-companion</url>
  <url type="bugtracker">https://github.com/marcelogm/arctis-companion/issues</url>
  <content_rating type="oars-1.1"/>
  <requires>
    <display_length compare="ge">360</display_length>
  </requires>
  <supports>
    <control>pointing</control>
    <control>keyboard</control>
    <control>touch</control>
  </supports>
  <releases>
    <release version="0.1.0" date="2026-10-12">
      <description>
        <p>First release.</p>
      </description>
    </release>
  </releases>
</component>
```

Keep the `.desktop.in` content as on main (`Name`, `Comment`, `Keywords` are extracted by `--language=Desktop`);
the meson `merge_file` wiring is already in place from Task 4.

- [ ] **Step 2: Run the validators and the packaging tests**

Run (toolbox):
`meson setup /tmp/arctis-sp14-meson --wipe && meson test -C /tmp/arctis-sp14-meson validate-desktop validate-metainfo validate-schema && cargo test -p arctis-app --test packaging the_metainfo`
Expected: PASS; `cargo test -p arctis-app --test packaging cargo_and` PASS too. Read
`/tmp/arctis-sp14-meson/meson-logs/testlog.txt`: fix every `W:` or `E:` line from `appstreamcli`; `I:`/`P:` lines
may stay. Run (ui): `cargo test -p arctis-ui --test widgets about_comes_from_the_metainfo` still PASS (the About
dialog reads this file).

- [ ] **Step 3: Regenerate the template, commit**

Run (toolbox): `sh po/extract.sh && msgmerge --update --backup=none po/pt_BR.po po/arctis-companion.pot`

```bash
git add data po
git commit -m "feat(data): translated desktop entry and metainfo with known gaps"
git push
```

## Task 10: Release workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Write the workflow**

```yaml
name: release

on:
  push:
    tags: ["v*"]
  pull_request:
    paths:
      - .github/workflows/release.yml

permissions:
  contents: read

jobs:
  version:
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4
      - name: tag matches the meson version
        run: grep -q "^  version: '${GITHUB_REF_NAME#v}'," meson.build

  flatpak:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/flathub-infra/flatpak-github-actions:gnome-50
      options: --privileged -v /:/host
    steps:
      - name: Free runner disk space
        run: rm -rf /host/usr/share/dotnet /host/usr/local/lib/android /host/opt/ghc /host/opt/hostedtoolcache/CodeQL /host/usr/local/share/boost
      - uses: actions/checkout@v4
      - uses: flatpak/flatpak-github-actions/flatpak-builder@v6
        with:
          bundle: arctis-companion.flatpak
          manifest-path: build-aux/io.github.marcelogm.ArctisCompanion.json
          cache-key: flatpak-release-${{ github.sha }}
          upload-artifact: false
      - uses: actions/upload-artifact@v4
        with:
          name: release-flatpak
          path: arctis-companion.flatpak

  extension:
    runs-on: ubuntu-latest
    container: fedora:44
    defaults:
      run:
        working-directory: extension
    steps:
      - run: dnf install -y --setopt=install_weak_deps=False nodejs npm gettext gnome-shell
        working-directory: /
      - uses: actions/checkout@v4
      - run: npm ci
      - run: npm run pack
      - uses: actions/upload-artifact@v4
        with:
          name: release-extension
          path: extension/dist/arctis-companion@marcelogm.github.io.shell-extension.zip

  publish:
    needs: [version, flatpak, extension]
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          pattern: release-*
          merge-multiple: true
          path: assets
      - env:
          GH_TOKEN: ${{ github.token }}
        run: >-
          gh release create "$GITHUB_REF_NAME"
          assets/arctis-companion.flatpak
          assets/arctis-companion@marcelogm.github.io.shell-extension.zip
          data/udev/70-arctis-companion.rules
          --title "$GITHUB_REF_NAME"
          --generate-notes
```

`version` is skipped on pull requests, so `publish` (which needs it and also checks the ref) never runs there.

- [ ] **Step 2: Run the packaging test and lint the YAML**

Run (toolbox): `cargo test -p arctis-app --test packaging && cargo xtask check-rules`
Expected: PASS (all five packaging tests). If `actionlint` is available (`sudo dnf install -y actionlint` may not
exist on Fedora; skip if not), run `actionlint .github/workflows/release.yml`.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: publish the bundle, the extension and the udev rule on a tag"
git push
```

The PR run (Task 20) exercises `flatpak` and `extension`; check both upload their artifact.

## Task 11: Complete pt_BR catalogue for the app

**Files:**
- Modify: `po/pt_BR.po`

- [ ] **Step 1: Translate every entry**

Header: `Language: pt_BR`, `Plural-Forms: nplurals=2; plural=(n > 1);`, `Content-Type: text/plain;
charset=UTF-8`, `Language-Team: Brazilian Portuguese`, `Last-Translator` and `Project-Id-Version:
arctis-companion 0.1.0` (no tool names). Translate with the glossary above, sentence case, “ ”, …, every
`{placeholder}` kept verbatim, mnemonics kept (`_Cancel` → `_Cancelar`, `_Rename` → `_Renomear`), `2.4 GHz`
written `2,4 GHz`. Built-in preset names are translated (audio §3): Flat → Plano, Bass Boost → Reforço de graves,
Bass Reduction → Redução de graves, Treble Soften → Agudos suaves, Vocal Clarity → Clareza vocal, Footsteps →
Passos, V-Shape → Formato em V, Movie Night → Noite de cinema, Podcast → Podcast, Late Night → Madrugada. Desktop
`Keywords` keep the trailing `;` and list Portuguese words (`headset;fone;equalizador;bateria;sidetone;`).

- [ ] **Step 2: Run the checks**

Run (toolbox): `sh po/check.sh && cargo test -p arctis-core --test view catalogue`
Expected: "catalogue up to date"; the three census tests PASS.

- [ ] **Step 3: Look at it**

Run (toolbox): `msgfmt -o /tmp/arctis-pt.mo po/pt_BR.po && msgunfmt /tmp/arctis-pt.mo | grep -c '^msgid'` equals
the POT's count. Read the file once top to bottom for consistency with the glossary.

- [ ] **Step 4: Commit**

```bash
git add po/pt_BR.po
git commit -m "feat(i18n): complete the pt_BR translation"
git push
```

## Task 11A: Translated About dialog

**Files:**
- Modify: `crates/ui/tests/widgets/app_dialogs.rs`
- Modify: `crates/ui/build.rs`, `crates/ui/resources/resources.gresource.xml`
- Modify: `.github/workflows/ci.yml` (`ui` job installs `gettext`)

`crates/ui/src/app_dialogs.rs::about_dialog` builds `AdwAboutDialog::from_appdata` from the GResource copy of the
metainfo. Since Task 4 that copy is the English template, so the About dialog's summary and release notes would stay
English under pt_BR.

- [ ] **Step 1: Write the failing test**

Append to `crates/ui/tests/widgets/app_dialogs.rs`:

```rust
#[gtk::test]
fn the_bundled_metainfo_carries_pt_br() {
    init();
    let bytes = gtk::gio::resources_lookup_data(
        "/io/github/marcelogm/ArctisCompanion/ui/metainfo.xml",
        gtk::gio::ResourceLookupFlags::NONE,
    )
    .expect("the metainfo resource");
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("xml:lang=\"pt-BR\""), "{text}");
}
```

Run (ui): `cargo test -p arctis-ui --test widgets the_bundled_metainfo_carries_pt_br`
Expected: FAIL (the template has no `xml:lang`).

- [ ] **Step 2: Merge the metainfo at build time**

`crates/ui/build.rs`: keep `METAINFO` (the template), add `const PO_DIR: &str = "../../po";`, print
`cargo::rerun-if-changed={PO_DIR}`, and before `compile_resources` call a new `merge_metainfo(&out_dir)?` that runs

```rust
fn merge_metainfo(out_dir: &Path) -> Result<(), Box<dyn Error>> {
    let merged = out_dir.join("io.github.marcelogm.ArctisCompanion.metainfo.xml");
    let status = Command::new("msgfmt")
        .arg("--xml")
        .arg(format!("--template={METAINFO}"))
        .args(["-d", PO_DIR, "-o"])
        .arg(merged)
        .status()
        .map_err(|error| format!("msgfmt: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("msgfmt --xml failed on the metainfo".into())
    }
}
```

In `crates/ui/resources/resources.gresource.xml` the entry goes back to
`<file alias="metainfo.xml">io.github.marcelogm.ArctisCompanion.metainfo.xml</file>`; `compile_resources` searches
`OUT_DIR` first, so it picks the merged file. `msgfmt` writes `xml:lang="pt-BR"` (checked with gettext 0.26 in the
toolbox, whose own `metainfo.its` needs no `appstream` package). In `.github/workflows/ci.yml` add `gettext` to the
`ui` job's `dnf install` list (`test` already has it; the GNOME 50 SDK ships `msgfmt`).

- [ ] **Step 3: Run the tests**

Run (ui): `cargo test -p arctis-ui --test widgets app_dialogs`
Expected: PASS, `about_comes_from_the_metainfo` included.

- [ ] **Step 4: Commit**

```bash
git add crates/ui/build.rs crates/ui/resources/resources.gresource.xml crates/ui/tests/widgets/app_dialogs.rs .github/workflows/ci.yml
git commit -m "feat(ui): translate the About dialog from the merged metainfo"
git push
```

## Task 12: Review the extension's pt_BR

**Files:**
- Modify: `extension/po/pt_BR.po`

- [ ] **Step 1: Apply the glossary**

Sentence case and the glossary: "Predefinições de Apps" → "Predefinições de apps", "Configurações do Headset" →
"Configurações do headset", and every other entry checked the same way. Keep placeholders and `·` separators.

- [ ] **Step 2: Run the extension catalogue check and tests**

Run (toolbox): `cd extension && npm run check-po && npm test`
Expected: "catalogue up to date"; tests PASS (they assert English strings, not pt_BR).

- [ ] **Step 3: Commit**

```bash
git add extension/po/pt_BR.po
git commit -m "fix(extension): align the pt_BR wording with the app"
git push
```

## Task 13: Bind the text domain at startup

**Files:**
- Create: `crates/app/src/locale.rs`
- Create: `crates/app/tests/locale.rs`
- Modify: `crates/app/src/lib.rs`, `crates/app/src/application.rs:168`, `crates/app/Cargo.toml`

- [ ] **Step 1: Write the failing test**

`crates/app/tests/locale.rs`:

```rust
use std::path::Path;

use arctis_app::locale::{GETTEXT_PACKAGE, locale_dir_beside};

#[test]
fn the_catalogue_sits_under_the_prefix_share_dir() {
    assert_eq!(
        locale_dir_beside(Path::new("/app/bin/arctis-companion")).as_deref(),
        Some(Path::new("/app/share/locale"))
    );
}

#[test]
fn a_bare_name_has_no_prefix() {
    assert_eq!(locale_dir_beside(Path::new("arctis-companion")), None);
}

#[test]
fn the_domain_is_the_meson_project_name() {
    let meson = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../meson.build"))
        .expect("meson.build");
    assert!(meson.starts_with(&format!("project(\n  '{GETTEXT_PACKAGE}',")));
}
```

- [ ] **Step 2: Run it to see it fail**

Run (toolbox): `cargo test -p arctis-app --test locale`
Expected: FAIL to compile, "could not find `locale` in `arctis_app`".

- [ ] **Step 3: Implement**

`crates/app/Cargo.toml` `[dependencies]`: `gettext-rs.workspace = true`.

`crates/app/src/locale.rs`:

```rust
use std::path::{Path, PathBuf};

use gettextrs::{LocaleCategory, bind_textdomain_codeset, bindtextdomain, setlocale, textdomain};

use crate::logging::warn;

pub const GETTEXT_PACKAGE: &str = "arctis-companion";

pub fn init() {
    if setlocale(LocaleCategory::LcAll, "").is_none() {
        warn("the locale is not supported; staying in English");
    }
    if let Some(dir) = std::env::current_exe().ok().and_then(|exe| locale_dir_beside(&exe)) {
        report(bindtextdomain(GETTEXT_PACKAGE, dir).map(drop));
    }
    report(bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8").map(drop));
    report(textdomain(GETTEXT_PACKAGE).map(drop));
}

#[must_use]
pub fn locale_dir_beside(executable: &Path) -> Option<PathBuf> {
    let prefix = executable.parent()?.parent()?;
    Some(prefix.join("share").join("locale"))
}

fn report(result: std::io::Result<()>) {
    if let Err(error) = result {
        warn(&format!("translations unavailable: {error}"));
    }
}
```

`/app/bin/arctis-companion` → `/app/share/locale` in the Flatpak; `/usr/bin` → `/usr/share/locale` for a meson
install; a `target/debug` run finds no catalogue and stays English.

Add `pub mod locale;` to `crates/app/src/lib.rs`. In `crates/app/src/application.rs::run`, call
`crate::locale::init();` as the first statement, before `arctis_ui::register_resources()`, so templates and the
first `Text` resolve in the user's language.

- [ ] **Step 4: Run the tests**

Run (ui): `cargo test -p arctis-app --test locale --test application`, then Run (toolbox):
`cargo clippy -p arctis-app --all-targets -- -D warnings`.
Expected: PASS (`crates/app/tests/application.rs` still starts the application with `init` in place).

- [ ] **Step 5: Commit**

```bash
git add crates/app Cargo.lock
git commit -m "feat(app): bind the gettext domain at startup"
git push
```

## Task 14: CI job for the catalogue

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add the job after `extension`**

```yaml
  i18n:
    needs: rules
    runs-on: ubuntu-latest
    container: fedora:44
    steps:
      - run: dnf install -y --setopt=install_weak_deps=False gettext appstream findutils diffutils
      - uses: actions/checkout@v4
      - run: sh po/check.sh
```

- [ ] **Step 2: Check locally what the job runs**

Run (toolbox): `sh po/check.sh && cargo xtask check-rules`
Expected: "catalogue up to date"; rules PASS.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: check the app catalogue on every pull request"
git push
```

## Task 15: Pin the SP10 publish-in-the-same-turn contract

**Files:**
- Modify: `crates/core/tests/companion/presets_flow.rs`

- [ ] **Step 1: Write the tests**

Reuse `crates/core/tests/companion/support.rs::published` (it returns the last `Action::Publish` of a turn as
`Option<Rc<Snapshot>>`); add `published` to the existing `use crate::support::{…}` list instead of writing a second
helper.

```rust
#[test]
fn saving_a_preset_publishes_it_in_the_same_turn() {
    let mut h = Harness::new(persisted());
    let actions = h.intent(Intent::SavePreset(mine(unique_curve())));
    assert!(published(&actions).is_some_and(|s| s.presets.find(&id("mine")).is_some()));
}

#[test]
fn deleting_a_preset_publishes_its_absence_in_the_same_turn() {
    let mut h = using_mine();
    let actions = h.intent(Intent::DeletePreset(id("mine")));
    assert!(published(&actions).is_some_and(|s| s.presets.find(&id("mine")).is_none()));
}
```

- [ ] **Step 2: Run them**

Run (toolbox): `cargo test -p arctis-core --test companion same_turn`
Expected: PASS (characterization of `crates/core/src/companion/reducer.rs`, which publishes after every `handle`).
If either FAILS, that is the real gap SP10 feared: make the reducer push `Action::Publish` in the same `handle`
call for `SavePreset`/`DeletePreset`, then rerun.

- [ ] **Step 3: Commit**

```bash
git add crates/core/tests/companion/presets_flow.rs
git commit -m "test(core): pin the snapshot published after saving or deleting a preset"
git push
```

## Task 15A: The soak reports its slowest confirmed write

**Files:**
- Modify: `crates/hid/tests/hardware/soak_log_rules.rs`
- Modify: `crates/hid/tests/hardware/soak_log.rs`, `crates/hid/tests/hardware/soak.rs`

Checklist row A1 needs "edit → verified readback within 300 ms when not Rebinding" as a number. `SoakLog` already
drops a write that a phase change interrupts (`enter` moves it to `interrupted`), so every confirmed write is one that
did not cross a Rebinding. Only the request time is missing. These are pure tests of the log; nothing here opens
hidraw.

- [ ] **Step 1: Write the failing test**

Append to `crates/hid/tests/hardware/soak_log_rules.rs`:

```rust
#[test]
fn the_slowest_confirmation_is_measured_from_the_request() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    let first = log.toggle_request(Duration::from_millis(1_000)).expect("a toggle");
    log.record(Duration::from_millis(1_120), DeviceUpdate::Observed(first));
    let second = log.toggle_request(Duration::from_millis(2_000)).expect("a toggle");
    log.record(Duration::from_millis(2_250), DeviceUpdate::Observed(second));
    assert_eq!(log.slowest_confirmation(), Some(Duration::from_millis(250)));
}

#[test]
fn an_interrupted_write_has_no_latency() {
    let mut log = log_after(vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Observed(original()),
    ]);
    let _ = log.toggle_request(Duration::from_millis(1_000));
    log.record(Duration::from_millis(1_050), DeviceUpdate::Phase(Phase::Rebinding));
    assert_eq!(log.slowest_confirmation(), None);
}
```

Existing callers of `toggle_request()` / `restore_request()` in this file pass `Duration::ZERO`.

Run (toolbox): `cargo test -p arctis-hid --test hardware soak_log`
Expected: FAIL to compile (`toggle_request` takes no argument; no `slowest_confirmation`).

- [ ] **Step 2: Implement**

In `crates/hid/tests/hardware/soak_log.rs`: `toggle_request(&mut self, at: Duration)` and
`restore_request(&mut self, at: Duration)` pass `at` to `request`, which stores `waiting_since: Option<Duration>`
next to `waiting_for`; `observe` takes `at` (from `record`) and, when it confirms, pushes
`at.saturating_sub(since)` into `pub confirmations: Vec<Duration>`; `enter` clears `waiting_since` with
`waiting_for`; `pub fn slowest_confirmation(&self) -> Option<Duration>` is `self.confirmations.iter().max().copied()`.
In `crates/hid/tests/hardware/soak.rs` pass `self.started.elapsed()` to both requests and add
`slowest confirmation {:?}` (`log.slowest_confirmation()`) to the `soak:` summary line. Keep each function within
the clippy limits.

- [ ] **Step 3: Run the tests**

Run (toolbox): `cargo test -p arctis-hid --test hardware && cargo clippy -p arctis-hid --all-targets -- -D warnings`
Expected: PASS; the ignored hardware test is not run (no `ARCTIS_HW`, no `--ignored`).

- [ ] **Step 4: Commit**

```bash
git add crates/hid/tests/hardware
git commit -m "test(hid): report the slowest confirmed write of the soak"
git push
```

## Task 16: Issues for deferred items and the nightly check

- [ ] **Step 1: Dispatch the nightly run**

Run (host): `gh workflow run nightly.yml --ref main`, then poll `gh run list --workflow nightly.yml -L 1` until it
completes (about an hour for mutants; continue with other tasks meanwhile). Read the `mutants` log:
`gh run view <id> --log --job <mutants job id> | grep -E 'MISSED|caught|missed'`.

- [ ] **Step 2: Open the issues**

Each body is one or two plain sentences, no attribution. Run (host) one `gh issue create --title … --body …` per
row:

| Id | Title | Body |
|---|---|---|
| A | `build(flatpak): offline cargo sources and Flathub metadata` | The manifest builds with network access. Flathub needs `cargo-sources.json` kept in step with `Cargo.lock`, plus screenshots in the metainfo. |
| C | `ci: keep the nightly shell-smoke log as an artifact` | Upload the nested shell output of the shell-smoke job so a failure can be read later. |
| D | `fix(ui): two Duplicates in one main-loop turn get the same id` | Duplicating the same built-in twice before the snapshot returns yields one fresh id for both (SP10 plan, Deviation 22). |
| E | `test(ui): find the source of the Gtk-CRITICAL gtk_widget_get_parent lines` | The accessibility walk prints Gtk-CRITICAL lines about `gtk_widget_get_parent`; they predate SP10. |
| F | `test(ui): fold the SP10 page helpers into the shared support` | `open_presets` and `shown_scrolled_windows` duplicate `support::open_page` and `assert_never_scrolls_horizontally`. |
| H | `feat(xtask): let check-doc-paths name macro_rules items` | Docs cannot point at `crates/core/src/testkit/scenarios/mod.rs::scenario_tests` because only fn, struct, enum and trait count. |
| I | `test(core): catch a shared scenario missing from scenario_tests!` | A `pub fn` scenario left out of the macro list runs nowhere and nothing reports it. |
| J | `fix(core): show the adopted headset-off banner after a restart` | With the headset off at startup, `Identity` arrives only on Syncing, so the page shows banner 2 instead of 3 (SP6 plan, Deviation 36). |
| K | `test(core): use testkit::companion in the companion test support` | `crates/core/tests/companion/support.rs` keeps its own `persisted`, `curve` and `NOVA_5`. |
| L | `fix(app): ignore stale Background portal answers` | Turning Run in Background on, off and on leaves two requests; a late denial of the first still applies. A request counter would drop it. |
| G | `test(core): kill the mutants left by the nightly run` | Only if Step 1 shows survivors: list them in the body (file:line and the mutation). |

Record the issue numbers in the Triage table of this plan (replace "Task 16 issue X" with `#<n>`), and commit:

```bash
git add docs/superpowers/plans/2026-10-12-sp14-packaging-release.md
git commit -m "docs: record the SP14 follow-up issues"
git push
```

## Task 17: Hardware checklist

**Files:**
- Create: `docs/hardware-checklist.md`

- [ ] **Step 1: Write the checklist**

Structure: a short intro (who runs it, that every box starts unticked, that scripts must keep the `Command`
allow-list guard of SP0, the sidetone warning from hardware-findings §11), then one table per area with columns
`#`, `Check`, `How`, `Passes when`, `Done`. The `Done` column is `[ ]` in every row; nobody but the person running
it ticks a box. Rows:

- **H — device (hardware-findings §7, items 1–14):** copy each item's procedure and designed default verbatim,
  "Passes when" = the measured value is recorded in `docs/research/hardware-findings.md` §7 and, if it differs from
  the default, an issue is opened against the named constant (`REBIND_SETTLE`, `VERIFY_BACKOFF`, `ECHO_WINDOW`,
  `WIRELESS_MODE_GRACE`, `STATUS_SAFETY_POLL`, `AUTO_SWITCH_MIN_INTERVAL`, `curve.rs` tolerances, sidetone marks).
  H14 is the SP5 soak with the headset on: `ARCTIS_HW=1 cargo test -p arctis-hid --test hardware -- --ignored
  --nocapture`, passes when the `soak:` line shows writes requested = confirmed and failed 0.
- **A — SP14 acceptance on hardware:**
  A1 2.4 GHz edit latency: headset on: run H14's `hardware` test, which drives the real
  `crates/hid/src/worker.rs::Worker` and confirms each sidetone write by an `Observed` readback (writes that cross a
  Rebinding are counted as interrupted, not timed); passes when the `soak:` line's "slowest confirmation" (Task 15A)
  is at most 300 ms.
  A2 ruled app: add a rule for a player (e.g. `org.gnome.Totem` or `firefox`) with a non-current preset, start
  playback: the preset row changes after 1 s; stop: it returns 10 s later (stopwatch or the debug log timestamps).
  A3 Bluetooth system engine: on a Bluetooth-paired computer, play audio through "Arctis Nova 5 (Equalizer)" for 30
  minutes at 48 kHz with quantum 256 (`pw-metadata -n settings 0 clock.force-quantum 256`), `pw-top` shows ERR 0
  for the EQ node at the end.
- **F — Flatpak and desktop:**
  F1 install the release bundle (`flatpak install --user arctis-companion.flatpak`) and the rule (`sudo cp
  70-arctis-companion.rules /etc/udev/rules.d/ && sudo udevadm control --reload-rules && sudo udevadm trigger`),
  launch from the app grid: the headset page appears.
  F2 without the rule (another computer or the rule moved away as root): the "Permission Required" page and its
  dialog show the rule and the command; both copy buttons work.
  F3 with the rule installed, unplug and plug the dongle by hand: the permission page never flashes.
  F4 close the window with Run in Background on: the process stays, GNOME's background apps list shows "Headset
  connected, NN %"; log out and in: the app starts in the background.
  F5 low battery: below the threshold a notification appears; clicking it opens the battery page; charging
  withdraws it.
  F6 pt_BR: if `flatpak info --user io.github.marcelogm.ArctisCompanion.Locale` does not list `pt`, run
  `flatpak config --user --set extra-languages pt_BR && flatpak update --user`; then quit the running instance and
  start `LANG=pt_BR.UTF-8 LANGUAGE=pt_BR flatpak run io.github.marcelogm.ArctisCompanion`: every page, dialog, toast
  and the About dialog in Portuguese, numbers with a decimal comma, nothing truncated at 360 px wide.
- **U — UI by hand (SP10):** U3 export a preset as JSON and as APO text, import both back and a published AutoEQ
  file; U4 open the Rename and the Details dialogs and use them with the keyboard only.
- **B — Bluetooth audio (SP7):** B1 the Nova 5 bluez node properties (`pw-dump`) match the matcher in
  `crates/audio/src/sinks.rs::sink_kind` (`device.api` `bluez5`, description "Arctis Nova 5"); B2 with the Flatpak app running, switch the default output between the Nova 5
  Bluetooth sink and speakers several times: the EQ sink is inserted and removed each time and the previous default
  comes back; B3 inside the Flatpak, if WirePlumber denies the Manager role, the warning row "Choose “Arctis Nova 5
  (Equalizer)” as output in Sound settings" appears.
- **E — Shell extension (SP13 devkit table):** in `extension/`, `npm run devkit -- <scenario>` for each scenario,
  one row each: connected, low-battery, charging, app-rule, manual-pick, no-rules, connecting, absent, with the
  "Expect" column copied verbatim from PR #4's comment (Run (host): `gh pr view 4 --comments`; scenario names as
  `extension/tests/nested-shell.sh` accepts them). E9: enable the installed extension yourself in the
  Extensions app; the toggle appears while the Flatpak app runs and hides when it quits.

- [ ] **Step 2: Check it renders and passes the rules**

Run (toolbox): `cargo xtask check-rules` (Markdown comments are forbidden; no `<!--`).
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add docs/hardware-checklist.md
git commit -m "docs: add the hardware checklist for a person to run"
git push
```

## Task 18: Architecture page for packaging, i18n and release

**Files:**
- Create: `docs/architecture/packaging.md`
- Modify: `docs/architecture/README.md` (move "Build and packaging" there, add the table row)

- [ ] **Step 1: Write the page**

Sections, each a mermaid diagram plus a table of backticked paths and symbols (all checked by
`check-doc-paths`):

1. **Build** — flowchart: `build-aux/io.github.marcelogm.ArctisCompanion.json` (GNOME 50, rust-stable, llvm22,
   finish-args) → `meson.build` → `po/meson.build` (`.mo`), `data/meson.build` (`merge_file` of
   `data/io.github.marcelogm.ArctisCompanion.desktop.in` and `data/io.github.marcelogm.ArctisCompanion.metainfo.xml.in`,
   gschema, D-Bus service, Headset1 XML, icons) → meson tests (desktop-file-validate, appstreamcli,
   glib-compile-schemas `--strict`, msgfmt `--check`) and cargo `--package arctis-app`.
   A finish-args table: each argument and why (`--device=all` for hidraw, `--filesystem=xdg-run/pipewire-0`, …).
2. **Translations** — flowchart: sources (`crates/core/src/l10n.rs::msgid`, `crates/core/src/l10n.rs::forms`,
   Blueprint `_()`, desktop, metainfo) → `po/extract.sh` → `po/arctis-companion.pot` → `po/pt_BR.po` →
   `po/check.sh` (CI `i18n` job) and `crates/core/tests/view/catalogue.rs` (census); runtime sequence:
   `crates/app/src/locale.rs::init` → `gettextrs::textdomain` → `crates/ui/src/text.rs::resolve`. The About dialog's
   metainfo is merged by `crates/ui/build.rs` (`msgfmt --xml`) into the GResource read by
   `crates/ui/src/app_dialogs.rs::about_dialog`. The extension's own domain: `extension/po/extract.sh`,
   `extension/po/pt_BR.po`.
3. **Release** — flowchart of `.github/workflows/release.yml`: tag `v*` → `version`, `flatpak`, `extension` →
   `publish` (`gh release create` with the bundle, the zip and `data/udev/70-arctis-companion.rules`); pull
   requests touching the workflow run the build jobs only. Pinned by `crates/app/tests/packaging.rs`.
4. **Install on a computer** — sequence: user installs the bundle, copies the rule, reloads udev, installs the
   extension zip with `gnome-extensions install`; the app's `crates/ui/src/permission_dialog.rs::PermissionDialog`
   repeats the rule and the command when the dongle is denied. Link to `docs/hardware-checklist.md`.
5. **Docs check** — one line: `xtask/src/check_doc_paths.rs::run` reads `docs/architecture/`, `CONTEXT.md` and the
   "- Code:" lines of `docs/adr/`.

In `docs/architecture/README.md`, replace the "Build and packaging" section with one sentence linking
`packaging.md`, add the row `| [packaging.md](packaging.md) | Flatpak build, translations, release, install |`,
add `release.yml` to the CI section's workflow list and an `i18n` box to the CI diagram.

- [ ] **Step 2: Run the doc check**

Run (toolbox): `cargo xtask check-doc-paths && cargo xtask check-rules`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add docs/architecture
git commit -m "docs(architecture): add the packaging, translations and release page"
git push
```

## Task 19: Build, install and launch the Flatpak; install the extension

- [ ] **Step 1: Full local test run**

Run (toolbox): `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --exclude arctis-ui --exclude arctis-app && sh po/check.sh && cd extension && npx eslint . && npm test && npm run check-po`
plus `cargo xtask check-rules && cargo xtask check-doc-paths && cargo xtask check-pure`, and Run (ui):
`cargo test -p arctis-ui -p arctis-app`.
Expected: PASS.

- [ ] **Step 2: Build and install**

Run (host):

```bash
mkdir -p /home/marcelo/.cache/arctis-flatpak
flatpak run org.flatpak.Builder --user --install --force-clean \
  --state-dir=/home/marcelo/.cache/arctis-flatpak/state \
  /home/marcelo/.cache/arctis-flatpak/build \
  /home/marcelo/Desenvolvimento/arctis-wt/sp14/build-aux/io.github.marcelogm.ArctisCompanion.json
```

Expected: ends with the app installed; the meson tests inside the build (`run-tests: true`) pass, which runs
desktop-file-validate, appstreamcli, `glib-compile-schemas --strict` and `msgfmt --check` in the GNOME 50 SDK.
Then: `flatpak info --user --show-permissions io.github.marcelogm.ArctisCompanion` lists `devices=all;` and
`filesystems=xdg-run/pipewire-0;`, and `flatpak list --user | grep ArctisCompanion` shows the app (and its
`.Locale` extension when the user's languages include pt). The catalogue was built and split into the Locale
extension: `find /home/marcelo/.cache/arctis-flatpak/build/files/share/runtime/locale -name arctis-companion.mo`
prints a `pt/share/pt_BR/LC_MESSAGES/arctis-companion.mo` path.

- [ ] **Step 3: Stop a running preview instance by PID**

Run (host):

```bash
for pid in $(pgrep -f arctis-companion); do
  case "$(readlink /proc/$pid/exe)" in
    /home/marcelo/.cache/arctis-preview-target/*) echo "stopping preview $pid"; kill "$pid" ;;
  esac
done
```

Never kill any other PID. Wait until `gdbus call --session --dest org.freedesktop.DBus --object-path
/org/freedesktop/DBus --method org.freedesktop.DBus.NameHasOwner io.github.marcelogm.ArctisCompanion` prints
`(false,)`.

- [ ] **Step 4: Launch from the Flatpak and verify**

First make sure nothing outside the Flatpak can win the D-Bus activation: `ls
~/.local/share/dbus-1/services/io.github.marcelogm.ArctisCompanion.service
~/.local/share/applications/io.github.marcelogm.ArctisCompanion.desktop` must report both missing (none existed on
2026-10-12). If one exists, do not delete it: record it in the "Local install" section and in the final report,
since it would start the preview instead of the Flatpak.

Run (host): `date --iso-8601=seconds > /home/marcelo/.cache/arctis-flatpak/launched-at && gapplication launch io.github.marcelogm.ArctisCompanion`
(D-Bus activation through the exported service file, as the app grid does). If it fails with "not found" or
`ServiceUnknown`, fall back to Run (host, in the background):
`flatpak run io.github.marcelogm.ArctisCompanion > /home/marcelo/.cache/arctis-flatpak/run.log 2>&1`.
Then, after a few seconds:
- `flatpak ps --columns=application,pid` lists the app;
- `gdbus introspect --session --dest io.github.marcelogm.ArctisCompanion --object-path /io/github/marcelogm/ArctisCompanion | grep io.github.marcelogm.ArctisCompanion.Headset1` prints the interface;
- `gdbus call --session --dest io.github.marcelogm.ArctisCompanion --object-path /io/github/marcelogm/ArctisCompanion --method org.freedesktop.DBus.Properties.Get io.github.marcelogm.ArctisCompanion.Headset1 State` returns a state other than `absent` and `permission-denied` (the host's existing rule grants hidraw; `connecting`
may show for a few seconds, so poll up to 30 s);
- the same call for `Route` returns `dongle` while the dongle's USB sink is the default output (PipeWire reached
  through `xdg-run/pipewire-0`); any other value is recorded with `pactl get-default-sink`, not treated as a failure;
- no `CRITICAL` in `run.log`, or, after `gapplication launch`, in
  `journalctl --user --since "$(cat /home/marcelo/.cache/arctis-flatpak/launched-at)" | grep -i arctis`.
Leave the app running; it is the user's real instance now.

- [ ] **Step 5: Install the extension (do not enable)**

Run (toolbox): `cd extension && npm ci && npm run pack` (`gnome-extensions` and `msgfmt` are in the toolbox).
Run (host): `gnome-extensions install --force /home/marcelo/Desenvolvimento/arctis-wt/sp14/extension/dist/arctis-companion@marcelogm.github.io.shell-extension.zip`
Expected: `gnome-extensions info arctis-companion@marcelogm.github.io` shows the extension; its enabled state is
whatever it was before.

- [ ] **Step 6: Record**

Add the outcome (date, Flatpak commit `flatpak info --user io.github.marcelogm.ArctisCompanion | grep Commit`,
state returned) as a short "Local install" section at the end of this plan, and commit:

```bash
git add docs/superpowers/plans/2026-10-12-sp14-packaging-release.md
git commit -m "docs: record the SP14 local install"
git push
```

## Task 20: Pull request and merge

- [ ] **Step 1: Open the PR**

Run (host):

```bash
gh pr create --base main --head sp14-packaging-release \
  --title "build: release packaging, pt_BR catalogue and hardware checklist" \
  --body "Final Flatpak manifest, translated metainfo and desktop entry, complete pt_BR catalogue, tag-driven release workflow and the hand-run hardware checklist.

Closes #3"
```

- [ ] **Step 2: Wait for green CI**

`gh pr checks --watch`. Expected: `rules`, `test`, `ui`, `extension`, `i18n`, `audio-integration`, `flatpak`,
and the release workflow's `flatpak` and `extension` jobs pass; `gh run download` of the release run shows both
artifacts. Fix any failure test-first on the branch.

- [ ] **Step 3: Merge**

```bash
gh pr merge --rebase
git push origin --delete sp14-packaging-release
```

Confirm issue #3 closed (`gh issue view 3 --json state`).

## Task 21: Tag the first release

- [ ] **Step 1: Tag main**

Run (host): `git -C /home/marcelo/Desenvolvimento/arctis-wt/sp14 fetch origin && git -C /home/marcelo/Desenvolvimento/arctis-wt/sp14 tag v0.1.0 origin/main && git -C /home/marcelo/Desenvolvimento/arctis-wt/sp14 push origin v0.1.0`

- [ ] **Step 2: Verify the release**

`gh run watch` on the `release` run, then `gh release view v0.1.0 --json assets --jq '.assets[].name'`.
Expected: `arctis-companion.flatpak`, `arctis-companion@marcelogm.github.io.shell-extension.zip`,
`70-arctis-companion.rules`.

- [ ] **Step 3: Clean up the worktree**

`git -C /home/marcelo/Desenvolvimento/arctis-companion worktree remove /home/marcelo/Desenvolvimento/arctis-wt/sp14`
then `git -C /home/marcelo/Desenvolvimento/arctis-companion branch -D sp14-packaging-release`.

## Self-review

- Every SP14 acceptance criterion maps to a task (table at the top); the hardware criteria are checklist rows by
  design (Deviation 13).
- Tests precede code in Tasks 1, 2, 3, 4 (`data_files.rs`), 6, 7, 11A, 13 and 15A; Task 15 is a characterization
  pin (it passes on arrival unless the SP10 gap is real); Tasks 4, 9, 10, 11, 14 are driven by `po/check.sh`, the
  census and `crates/app/tests/packaging.rs`, which are written red first.
- Renaming the desktop entry and metainfo to `.in` templates is followed into every reader on `origin/main`:
  `crates/ui/build.rs`, `crates/ui/resources/resources.gresource.xml`, `crates/app/tests/data_files.rs`,
  `docs/architecture/README.md`, `docs/architecture/ui.md`.
- Names used across tasks: `arctis_core::l10n::{msgid, forms}` (Task 3, used in 5, 18), `po/extract.sh` /
  `po/check.sh` (4, 11, 14, 18), `crates/app/src/locale.rs::{init, locale_dir_beside, GETTEXT_PACKAGE}` (13, 18),
  `crates/app/tests/packaging.rs` (7–10, 18), `.github/workflows/release.yml` (7, 10, 18, 21).

## Local install

2026-10-12: `flatpak run org.flatpak.Builder --user --install --force-clean` built and installed
`io.github.marcelogm.ArctisCompanion` 0.1.0 (commit `bd418139de05`) with its `.Locale` extension; the four meson tests
inside the build passed (check-pt_BR, validate-desktop, validate-schema, validate-metainfo). Permissions:
`devices=all;`, `filesystems=xdg-run/pipewire-0;`. No preview instance was running. `gapplication launch` failed
with `ServiceUnknown` (the session bus had not reloaded the exported service file), so the app was started with
`flatpak run`. Over D-Bus, `Headset1.State` returned `connected` and `Route` returned `dongle`; the run log has no
CRITICAL line. The extension zip was installed with `gnome-extensions install --force` and is not enabled.
