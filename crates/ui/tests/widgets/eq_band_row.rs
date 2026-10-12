use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use arctis_core::eq::{Filter, FilterKind};
use arctis_core::testkit::english::english;
use arctis_core::testkit::snapshots::snapshot;
use arctis_core::view::eq_editor::{BandModel, eq_editor_for, kind_position};
use arctis_core::view::preset_edits::BandEdit;
use arctis_ui::EqBandRow;

use crate::support::{assert_close, combo_text, custom_preset, filter, init, named};

const CONTROLS: [&str; 5] = [
    "band-kind",
    "band-frequency",
    "band-gain",
    "band-q",
    "band-enabled",
];

fn model_of(filter: Filter) -> BandModel {
    eq_editor_for(&snapshot(), &custom_preset(&[filter]))
        .bands
        .remove(0)
}

fn row_showing(filter: Filter, editable: bool) -> EqBandRow {
    init();
    let row = EqBandRow::default();
    row.render(&model_of(filter), editable);
    row
}

fn recorded_edits(row: &EqBandRow) -> Rc<RefCell<Vec<BandEdit>>> {
    let edits = Rc::new(RefCell::new(Vec::new()));
    let writer = Rc::clone(&edits);
    row.connect_edit(move |edit| writer.borrow_mut().push(edit));
    edits
}

#[gtk::test]
fn a_band_shows_its_values() {
    let band = filter(FilterKind::LowShelf, 105, 13, 700);
    let row = row_showing(band, true);
    assert_eq!(row.title(), english(&model_of(band).title));
    assert_eq!(combo_text(&named(&row, "band-kind")), "Low Shelf");
    assert_close(named::<adw::SpinRow>(&row, "band-frequency").value(), 105.0);
    assert_close(named::<adw::SpinRow>(&row, "band-gain").value(), 6.5);
    assert_close(named::<adw::SpinRow>(&row, "band-q").value(), 0.7);
    assert!(named::<adw::SwitchRow>(&row, "band-enabled").is_active());
    assert!(named::<adw::ButtonRow>(&row, "band-remove").is_visible());
}

#[gtk::test]
fn low_pass_and_high_pass_make_the_gain_insensitive() {
    let kinds = [
        (FilterKind::Peaking, true),
        (FilterKind::LowShelf, true),
        (FilterKind::HighShelf, true),
        (FilterKind::LowPass, false),
        (FilterKind::HighPass, false),
    ];
    for (kind, sensitive) in kinds {
        let row = row_showing(filter(kind, 1_000, 0, 707), true);
        let gain = named::<adw::SpinRow>(&row, "band-gain");
        assert_eq!(gain.is_sensitive(), sensitive, "{kind:?}");
    }
}

#[gtk::test]
fn a_read_only_band_cannot_change() {
    let row = row_showing(filter(FilterKind::Peaking, 250, 4, 1_000), false);
    for name in CONTROLS {
        assert!(!named::<gtk::Widget>(&row, name).is_sensitive(), "{name}");
    }
    assert!(!named::<adw::ButtonRow>(&row, "band-remove").is_visible());
}

#[gtk::test]
fn each_control_reports_its_edit() {
    let row = row_showing(filter(FilterKind::Peaking, 250, 4, 1_000), true);
    let edits = recorded_edits(&row);
    let low_pass = u32::try_from(kind_position(FilterKind::LowPass)).expect("small position");
    named::<adw::ComboRow>(&row, "band-kind").set_selected(low_pass);
    named::<adw::SpinRow>(&row, "band-frequency").set_value(300.0);
    named::<adw::SpinRow>(&row, "band-gain").set_value(-1.5);
    named::<adw::SpinRow>(&row, "band-q").set_value(2.0);
    named::<adw::SwitchRow>(&row, "band-enabled").set_active(false);
    assert_eq!(
        *edits.borrow(),
        vec![
            BandEdit::Kind(FilterKind::LowPass),
            BandEdit::Frequency(300.0),
            BandEdit::Gain(-1.5),
            BandEdit::Q(2.0),
            BandEdit::Enabled(false),
        ]
    );
}

#[gtk::test]
fn remove_band_reports_itself() {
    let row = row_showing(filter(FilterKind::Peaking, 250, 4, 1_000), true);
    let removed = Rc::new(Cell::new(false));
    let flag = Rc::clone(&removed);
    row.connect_remove(move || flag.set(true));
    named::<adw::ButtonRow>(&row, "band-remove").emit_by_name::<()>("activated", &[]);
    assert!(removed.get());
}

#[gtk::test]
fn rendering_reports_no_edit() {
    let row = row_showing(filter(FilterKind::Peaking, 250, 4, 1_000), true);
    let edits = recorded_edits(&row);
    row.render(
        &model_of(filter(FilterKind::HighPass, 9_000, 0, 1_410)),
        true,
    );
    row.render(&model_of(filter(FilterKind::Peaking, 40, -20, 200)), false);
    assert!(edits.borrow().is_empty());
}
