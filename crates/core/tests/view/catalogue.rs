mod debug_msgids;
mod po_file;

use std::collections::BTreeSet;
use std::fmt::Debug;

use arctis_core::battery::sample::{Reading, UnixSeconds};
use arctis_core::battery::series::Window;
use arctis_core::companion::alerts::Alerts;
use arctis_core::eq::Curve;
use arctis_core::eq::library::library;
use arctis_core::eq::reference::plot;
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
use arctis_core::view::text::{Arg, Text, preset_title};

use crate::headset_page::{PHASES, ROUTES};
use crate::support::{percent, rules, snapshot, with};
use debug_msgids::msgids_in;
use po_file::Catalogue;

const PT_BR: &str = include_str!("../../../../po/pt_BR.po");

const SAMPLE_PO: &str = "msgid \"\"\nmsgstr \"\"\n\"Language: pt_BR\\n\"\n\n\
msgid \"Low\"\nmsgstr \"Baixo\"\n\n\
#, fuzzy\nmsgid \"High\"\nmsgstr \"Alto\"\n\n\
msgid \"{n} minute\"\nmsgid_plural \"{n} minutes\"\nmsgstr[0] \"{n} minuto\"\nmsgstr[1] \"{n} minutos\"\n\n\
msgid \"\"\n\"Long \"\n\"line\"\nmsgstr \"\"\n\"Linha \"\n\"longa\"\n\n\
msgid \"Back\\\\nslash\"\nmsgstr \"Barra\\\\ninvertida\"\n\n\
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
        [
            "A \"quoted\" word",
            "Flat",
            "{n} rule",
            "{n} rules",
            "{x} left"
        ]
    );
}

#[test]
fn the_po_reader_knows_translated_entries() {
    let catalogue = Catalogue::parse(SAMPLE_PO);
    let translated = [
        "Low",
        "{n} minute",
        "{n} minutes",
        "Long line",
        "Back\\nslash",
    ];
    let untranslated = ["High", "Missing", "Absent"];
    assert!(
        translated
            .iter()
            .all(|msgid| catalogue.is_translated(msgid))
    );
    assert!(
        !untranslated
            .iter()
            .any(|msgid| catalogue.is_translated(msgid))
    );
}

fn unescape(quoted: &str) -> String {
    let mut text = String::new();
    let mut chars = quoted.chars();
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

fn dump(value: &impl Debug) -> String {
    format!("{value:?}")
}

fn headset_dumps() -> Vec<String> {
    PHASES
        .iter()
        .flat_map(|phase| ROUTES.iter().map(move |route| with(*phase, *route)))
        .flat_map(|s| {
            [
                dump(&headset_page(&s)),
                dump(&banner(&s)),
                dump(&equalizer_group(&s)),
            ]
        })
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
    dumps.extend(
        AutoPowerOff::CHOICES
            .iter()
            .map(|choice| dump(&auto_power_off_text(*choice))),
    );
    dumps.push(dump(&kind_items()));
    dumps
}

fn file_dumps() -> Vec<String> {
    let s = snapshot();
    let eleven = (1..=11)
        .map(|n| format!("Filter {n}: ON PK Fc 100 Hz Gain 1 dB Q 1\n"))
        .collect::<Vec<String>>()
        .concat();
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
    [
        headset_dumps(),
        page_dumps(),
        preset_dumps(),
        value_dumps(),
        file_dumps(),
        alert_dumps(),
    ]
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
fn every_plotted_series_label_is_translated_to_pt_br() {
    let catalogue = Catalogue::parse(PT_BR);
    for series in plot(&Curve::default()) {
        let label = series.id.label();
        assert!(catalogue.is_translated(label), "untranslated: {label}");
    }
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
    let expected: BTreeSet<&str> = msgids
        .iter()
        .flat_map(|msgid| placeholders(msgid))
        .collect();
    forms.iter().any(|form| placeholders(form) != expected)
}

fn placeholders(text: &str) -> BTreeSet<&str> {
    text.split('{')
        .skip(1)
        .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
        .collect()
}
