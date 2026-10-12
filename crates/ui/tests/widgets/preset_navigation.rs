use std::rc::Rc;

use adw::prelude::*;
use arctis_core::testkit::autoeq::{SENNHEISER_HD_600, SENNHEISER_HD_600_FILE};
use arctis_core::testkit::snapshots::snapshot;
use arctis_ui::preset_files::{PickedText, PresetFiles};

use crate::support::{
    FakeFiles, descendants, named, one_peak, open_editor, open_presets, rendered_window, saved,
    shows_text, visible_tag, with_preset,
};

#[gtk::test]
fn show_page_opens_only_pages_that_exist() {
    let (window, _) = rendered_window(snapshot());
    let navigation = named::<adw::NavigationView>(&window, "navigation");
    let show = || {
        WidgetExt::activate_action(&window, "win.show-page", Some(&"test-page".to_variant()))
            .expect("the action exists");
    };
    show();
    assert_eq!(visible_tag(&window).as_deref(), Some("main"));
    navigation.add(&adw::NavigationPage::with_tag(
        &gtk::Label::new(None),
        "Test",
        "test-page",
    ));
    show();
    assert_eq!(visible_tag(&window).as_deref(), Some("test-page"));
}

#[gtk::test]
fn the_presets_row_opens_the_presets_page() {
    let (window, _) = rendered_window(snapshot());
    open_presets(&window);
    assert_eq!(visible_tag(&window).as_deref(), Some("presets"));
}

#[gtk::test]
fn a_preset_opens_in_the_editor() {
    let (window, log) = rendered_window(snapshot());
    open_presets(&window);
    let page = open_editor(&window, "bass-boost");
    assert_eq!(visible_tag(&window).as_deref(), Some("eq-editor"));
    assert_eq!(page.title(), "Bass Boost");
    open_editor(&window, "flat");
    assert_eq!(page.title(), "Flat");
    assert_eq!(visible_tag(&window).as_deref(), Some("eq-editor"));
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn an_unknown_preset_opens_nothing() {
    let (window, log) = rendered_window(snapshot());
    open_presets(&window);
    WidgetExt::activate_action(&window, "win.show-preset", Some(&"gone".to_variant()))
        .expect("the action exists");
    assert_eq!(visible_tag(&window).as_deref(), Some("presets"));
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn the_window_renders_the_preset_pages() {
    let (window, _) = rendered_window(snapshot());
    window.render(&Rc::new(with_preset(&one_peak())));
    open_presets(&window);
    let custom = named::<adw::PreferencesGroup>(&window, "custom-presets");
    assert!(custom.description().is_none_or(|text| text.is_empty()));
    let titles: Vec<String> = descendants(&custom)
        .into_iter()
        .filter(|widget| widget.widget_name() == "preset-row")
        .filter_map(|widget| widget.downcast::<adw::ActionRow>().ok())
        .map(|row| row.title().to_string())
        .collect();
    assert_eq!(titles, ["Mine"]);
}

#[gtk::test]
fn the_window_hands_the_file_chooser_to_its_pages() {
    let (window, log) = rendered_window(snapshot());
    let files = Rc::new(FakeFiles::default());
    files.picked.replace(Some(PickedText {
        name: SENNHEISER_HD_600_FILE.to_owned(),
        text: SENNHEISER_HD_600.to_owned(),
    }));
    let shared: Rc<dyn PresetFiles> = files.clone();
    window.set_preset_files(shared);
    open_presets(&window);
    named::<gtk::Button>(&window, "import").emit_clicked();
    assert_eq!(saved(&log).name(), "Sennheiser HD 600 ParametricEQ");
    assert!(shows_text(
        &window,
        "Imported “Sennheiser HD 600 ParametricEQ”"
    ));
}
