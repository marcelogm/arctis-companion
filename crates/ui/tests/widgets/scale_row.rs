use adw::prelude::*;
use arctis_core::companion::Snapshot;
use arctis_core::intent::Intent;
use arctis_core::settings::headset_settings::Setting;
use arctis_core::settings::values::{MicVolume, Sidetone};
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::view::headset_rows::{HeadsetGroupsModel, ScaleModel, headset_groups};
use arctis_ui::ScaleRow;
use arctis_ui::intents::IntentSink;
use gtk::{AccessibleProperty, glib};

use crate::support::{
    Log, assert_close, descendants, first, has_accessible_text, init, named, recording_sink,
};

fn groups(snapshot: &Snapshot) -> HeadsetGroupsModel {
    headset_groups(&snapshot.dongle).expect("a supported model")
}

fn sidetone_model(snapshot: &Snapshot) -> ScaleModel {
    groups(snapshot).sidetone.expect("the Nova 5 has sidetone")
}

fn rendered(model: &ScaleModel) -> (ScaleRow, IntentSink, Log) {
    init();
    let (sink, log) = recording_sink();
    let row: ScaleRow = glib::Object::new();
    row.connect_intents(&sink);
    sink.rendering(|| row.render(model));
    (row, sink, log)
}

#[gtk::test]
fn the_row_shows_the_model() {
    let model = sidetone_model(&snapshot());
    let (row, _, _) = rendered(&model);
    let scale = first::<gtk::Scale>(&row);
    assert_eq!(row.title(), english(&model.title));
    assert_close(scale.value(), 3.0);
    assert_close(scale.adjustment().upper(), f64::from(Sidetone::MAX));
    assert!(has_accessible_text(
        &scale,
        AccessibleProperty::ValueText,
        "Low"
    ));
}

#[gtk::test]
fn a_step_between_marks_reads_as_a_count() {
    let mut s = snapshot();
    s.dongle.desired.sidetone = Sidetone::try_from(4).expect("sidetone");
    let (row, _, _) = rendered(&sidetone_model(&s));
    let scale = first::<gtk::Scale>(&row);
    assert!(has_accessible_text(
        &scale,
        AccessibleProperty::ValueText,
        "4 of 10"
    ));
}

#[gtk::test]
fn moving_the_scale_emits_the_setting_intent() {
    let (row, _, log) = rendered(&sidetone_model(&snapshot()));
    first::<gtk::Scale>(&row).set_value(6.0);
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetSidetone(
            Sidetone::try_from(6).expect("sidetone")
        )]
    );
}

#[gtk::test]
fn rendering_a_new_value_emits_nothing() {
    let (row, sink, log) = rendered(&sidetone_model(&snapshot()));
    let mut s = snapshot();
    s.dongle.desired.sidetone = Sidetone::try_from(10).expect("sidetone");
    sink.rendering(|| row.render(&sidetone_model(&s)));
    assert!(log.borrow().is_empty());
}

#[gtk::test]
fn the_microphone_volume_has_quiet_and_loud_icons() {
    let model = groups(&snapshot()).mic_volume.expect("mic volume");
    let (row, _, log) = rendered(&model);
    let icons: Vec<gtk::Image> = descendants(&row)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Image>().ok())
        .filter(WidgetExt::get_visible)
        .collect();
    assert_eq!(icons.len(), 2);
    first::<gtk::Scale>(&row).set_value(9.0);
    assert_eq!(
        *log.borrow(),
        vec![Intent::SetMicVolume(
            MicVolume::try_from(9).expect("volume")
        )]
    );
}

#[gtk::test]
fn sidetone_has_no_end_icons() {
    let (row, _, _) = rendered(&sidetone_model(&snapshot()));
    assert!(
        descendants(&row)
            .into_iter()
            .filter_map(|widget| widget.downcast::<gtk::Image>().ok())
            .all(|icon| !icon.get_visible())
    );
}

#[gtk::test]
fn a_setting_not_applied_shows_its_subtitle() {
    let mut s = snapshot();
    s.dongle.not_applied = vec![Setting::Sidetone];
    let (row, _, _) = rendered(&sidetone_model(&s));
    let subtitle = named::<gtk::Label>(&row, "subtitle");
    assert!(subtitle.get_visible());
    assert_eq!(subtitle.label(), "Not applied");
}

#[gtk::test]
fn the_scale_has_an_accessible_label_and_value_text() {
    let (row, _, _) = rendered(&sidetone_model(&snapshot()));
    let scale = first::<gtk::Scale>(&row);
    assert!(has_accessible_text(
        &scale,
        AccessibleProperty::Label,
        &english(&sidetone_model(&snapshot()).title)
    ));
    assert!(has_accessible_text(
        &scale,
        AccessibleProperty::ValueText,
        "Low"
    ));
}
