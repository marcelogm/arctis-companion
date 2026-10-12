use std::rc::Rc;

use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::eq::FilterKind;
use arctis_core::testkit::autoeq::{SENNHEISER_HD_600, SENNHEISER_HD_600_FILE};
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::view::presets::presets_page;
use arctis_ui::PresetsPage;
use arctis_ui::preset_files::PickedText;

use crate::support::{
    FakeFiles, Mounted, descendants, filter, has_button, mount, named, one_peak, saved, shows_text,
    with_preset,
};

fn shown(s: Snapshot) -> Mounted<PresetsPage> {
    let mounted = mount(|page: &PresetsPage, services| {
        page.set_services(services);
    });
    mounted.page.render(&Rc::new(s));
    mounted
}

fn rows(page: &PresetsPage, group: &str) -> Vec<adw::ActionRow> {
    descendants(&named::<adw::PreferencesGroup>(page, group))
        .into_iter()
        .filter(|widget| widget.widget_name() == "preset-row")
        .filter_map(|widget| widget.downcast::<adw::ActionRow>().ok())
        .collect()
}

fn titles(page: &PresetsPage, group: &str) -> Vec<String> {
    rows(page, group)
        .iter()
        .map(|row| row.title().to_string())
        .collect()
}

fn pick(files: &FakeFiles, name: &str, text: &str) {
    files.picked.replace(Some(PickedText {
        name: name.to_owned(),
        text: text.to_owned(),
    }));
}

#[gtk::test]
fn built_in_and_custom_presets_are_listed_apart() {
    let s = with_preset(&one_peak());
    let model = presets_page(&s);
    let shown = shown(s);
    let built_in: Vec<String> = model
        .built_in
        .iter()
        .map(|item| english(&item.title))
        .collect();
    assert_eq!(titles(&shown.page, "built-in-presets"), built_in);
    assert_eq!(titles(&shown.page, "custom-presets"), ["Mine"]);
    let custom = named::<adw::PreferencesGroup>(&shown.page, "custom-presets");
    assert!(custom.description().is_none_or(|text| text.is_empty()));
}

#[gtk::test]
fn an_empty_custom_group_explains_itself() {
    let shown = shown(snapshot());
    let custom = named::<adw::PreferencesGroup>(&shown.page, "custom-presets");
    assert_eq!(
        custom.description().as_deref(),
        Some("Create, duplicate or import a preset to customize it")
    );
    assert!(titles(&shown.page, "custom-presets").is_empty());
}

#[gtk::test]
fn each_row_opens_its_preset_in_the_editor() {
    let shown = shown(snapshot());
    let first = rows(&shown.page, "built-in-presets").remove(0);
    assert_eq!(first.action_name().as_deref(), Some("win.show-preset"));
    assert_eq!(first.action_target_value(), Some("flat".to_variant()));
    assert!(first.is_activatable());
}

#[gtk::test]
fn a_new_snapshot_refreshes_the_rows() {
    let shown = shown(snapshot());
    shown.page.render(&Rc::new(with_preset(&one_peak())));
    assert_eq!(titles(&shown.page, "custom-presets"), ["Mine"]);
    assert_eq!(titles(&shown.page, "built-in-presets").len(), 11);
}

#[gtk::test]
fn importing_a_published_autoeq_file_saves_a_custom_preset() {
    let shown = shown(snapshot());
    pick(&shown.files, SENNHEISER_HD_600_FILE, SENNHEISER_HD_600);
    named::<gtk::Button>(&shown.page, "import").emit_clicked();
    let preset = saved(&shown.log);
    assert_eq!(preset.name(), "Sennheiser HD 600 ParametricEQ");
    assert_eq!(preset.curve().filters().len(), 10);
    assert_eq!(shown.log.borrow().len(), 1);
    assert!(shows_text(
        &shown.toasts,
        "Imported “Sennheiser HD 600 ParametricEQ”"
    ));
    assert!(!has_button(&shown.toasts, "Details"));
}

#[gtk::test]
fn clamped_values_are_announced_with_their_details() {
    let shown = shown(snapshot());
    pick(
        &shown.files,
        "Hot.txt",
        "Filter 1: ON PK Fc 100 Hz Gain 14 dB Q 1\n",
    );
    named::<gtk::Button>(&shown.page, "import").emit_clicked();
    assert_eq!(
        saved(&shown.log).curve().filters()[0],
        filter(FilterKind::Peaking, 100, 20, 1_000)
    );
    assert!(shows_text(
        &shown.toasts,
        "Imported “Hot”. 1 value was changed to fit the headset"
    ));
    assert!(has_button(&shown.toasts, "Details"));
}

#[gtk::test]
fn a_file_without_filters_raises_a_toast_and_saves_nothing() {
    let shown = shown(snapshot());
    pick(&shown.files, "notes.txt", "hello");
    named::<gtk::Button>(&shown.page, "import").emit_clicked();
    assert!(shown.log.borrow().is_empty());
    assert!(shows_text(&shown.toasts, "“notes.txt” has no filters"));
}

#[gtk::test]
fn a_cancelled_import_does_nothing() {
    let shown = shown(snapshot());
    named::<gtk::Button>(&shown.page, "import").emit_clicked();
    assert!(shown.log.borrow().is_empty());
}
